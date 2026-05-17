# 05 監査証跡（AuditTrail）方式

本章の責務は、システム上で発生するすべての業務事象・操作事象を改ざん不能・追跡可能な形式で記録・保持するための設計を確定することである。FR-AU-001〜006・NFR-ALC（7 年保全）・NFR-SEC-008（ハッシュチェーン整合性）・BR-BUS-040（操作ログ全件記録）を受けて、イベント分類・ストレージ設計・ハッシュチェーン方式・保全ポリシー・XES 形式エクスポートの設計命題を固定する。

図: `img/fig_des_sec_audit_trail.drawio` / `img/fig_des_sec_audit_trail.svg` を参照。

---

## 1. 監査ログの格納先とアクセス制御

### 1-1. TBL-032 audit_logs 設計方針

すべての監査イベントは TBL-032（`audit_logs`）に記録する。本テーブルは以下の制約によって Append-only を技術的に強制する。

| 制約 | 実装 | 根拠 |
|---|---|---|
| INSERT のみ許可 | DB ロール `audit_role` に対して `GRANT INSERT ON audit_logs` のみを付与。UPDATE / DELETE は `REVOKE` | FR-AU-003 改ざん防止 |
| テーブル所有者の変更禁止 | `audit_role` は DDL 実行権限を持たない | 管理者による意図的な削除防止 |
| ハッシュチェーン列 | `prev_hash`（前レコードの SHA-256 ハッシュ）を必須カラムとして保持 | FR-AU-006 改ざん検知 |
| パーティショニング | RANGE パーティション（月次）により 7 年分のデータを効率的に管理 | NFR-ALC 7 年保全 |
| バックアップ | pg_basebackup + WAL アーカイブ（KEY-004 で暗号化済み）に含まれる | NFR-ALC |

### 1-2. 監査ロール権限マトリクス

| DB ロール | INSERT | SELECT | UPDATE | DELETE | DDL |
|---|---|---|---|---|---|
| `audit_role`（バックエンド専用） | 可 | 可 | 不可 | 不可 | 不可 |
| `readonly_role`（レポート生成） | 不可 | 可（制限あり）| 不可 | 不可 | 不可 |
| `system_admin`（人間 DBA） | 不可 | 可 | 不可 | 不可 | 不可 |
| `postgres`（緊急用スーパーユーザー） | 可 | 可 | 可 | 可 | 可 |

> 注意: `postgres` スーパーユーザーによる監査ログ操作はログ自体に記録されないため、OS レベルの PostgreSQL アクセスログ（`log_connections = on`・`log_disconnections = on`）を別途保持し相互検証する。

---

## 2. 監査イベント分類と LOG 識別子

すべての監査イベントはイベントタイププレフィックスによって分類する。

### 2-1. AUTH_* 系（認証・セッション）

| LOG 識別子 | イベントタイプ | トリガー条件 | 記録要素 |
|---|---|---|---|
| LOG-001 | `AUTH_LOGIN_SUCCESS` | PIN 認証成功 | user_id, terminal_id, timestamp, IP |
| LOG-002 | `AUTH_LOGIN_FAIL` | PIN 認証失敗（1 回ごと） | user_id（試行値）, terminal_id, 失敗回数, IP |
| LOG-003 | `AUTH_LOGOUT` | 明示的ログアウト / セッション期限切れ | user_id, terminal_id, reason |
| LOG-004 | `AUTH_LOCK` | 連続 3 回失敗による自動ロック（BR-BUS-035） | user_id, terminal_id, locked_at |

### 2-2. WORK_* 系（作業実行）

| LOG 識別子 | イベントタイプ | トリガー条件 | 記録要素 |
|---|---|---|---|
| LOG-005 | `WORK_START` | 作業開始（FR-WN-001） | work_execution_id, sop_version_id, operator_id |
| LOG-005 | `WORK_STEP_COMPLETE` | 各ステップ完了（FR-WN-002） | work_execution_id, step_id, timestamp_client, timestamp_server |
| LOG-005 | `WORK_COMPLETE` | 作業完了 | work_execution_id, completed_at, duration_ms |
| LOG-005 | `WORK_SUSPEND` | 作業中断（FR-KZ-001 アンドン含む） | work_execution_id, reason_code, suspended_at |

### 2-3. MASTER_* 系（マスタ操作）

| LOG 識別子 | イベントタイプ | トリガー条件 | 記録要素 |
|---|---|---|---|
| LOG-006 | `MASTER_CREATE` | SOP 新規作成 | sop_id, version, creator_id |
| LOG-006 | `MASTER_APPROVE` | SOP 承認（FR-MA-003 二重承認） | sop_version_id, approver_id, approver_role |
| LOG-006 | `MASTER_PUBLISH` | SOP 公開 | sop_version_id, published_at |
| LOG-006 | `MASTER_DEPRECATE` | SOP 廃止 | sop_version_id, deprecated_at, reason |

### 2-4. REPORT_* 系（帳票操作）

| LOG 識別子 | イベントタイプ | トリガー条件 | 記録要素 |
|---|---|---|---|
| LOG-007 | `REPORT_GENERATE` | PDF 生成要求（FR-QR-001〜005） | report_type, work_execution_id, generator_id |
| LOG-007 | `REPORT_DOWNLOAD` | PDF ダウンロード | report_id, downloader_id, IP |
| LOG-007 | `REPORT_PRINT` | 印刷指示 | report_id, printer_id, print_count |
| LOG-007 | `REPORT_DISPOSE` | 廃棄記録 | report_id, disposer_id, reason |

### 2-5. SYSTEM_* 系（システム設定）

| LOG 識別子 | イベントタイプ | トリガー条件 | 記録要素 |
|---|---|---|---|
| LOG-008 | `SYSTEM_CONFIG_CHANGE` | 設定値変更（CFG-NNN） | config_key, old_value, new_value, operator_id |
| LOG-009 | `SYSTEM_DLQ_ACTION` | DLQ メッセージ操作（FR-SY-007） | dlq_event_id, action, operator_id |
| LOG-010 | `SYSTEM_KEY_ROTATE` | 鍵ローテーション実施 | key_id, old_fingerprint, new_fingerprint, operator_id |

---

## 3. ハッシュチェーン設計

### 3-1. prev_hash によるチェーン構造

作業イベント（work_events テーブル）の各レコードは `prev_hash` カラムに直前レコードのハッシュを保持し、単方向連結リストを形成する。

```
work_event[n].prev_hash = SHA-256(
    work_event[n-1].event_id
    || work_event[n-1].timestamp_server
    || work_event[n-1].content_hash
    || daily_salt(KEY-009)
)
```

| 要素 | 内容 |
|---|---|
| ハッシュアルゴリズム | SHA-256 |
| チェーン単位 | work_execution_id ごと（各作業フローが独立したチェーン） |
| チェーン先頭 | `prev_hash = SHA-256("GENESIS" || daily_salt(KEY-009))` |
| KEY-009 の役割 | 日次更新の salt。知らない攻撃者が外部からチェーンを再構成できなくする |

### 3-2. hash_chain_blocks テーブルとの連携

`hash_chain_blocks` テーブルは各 work_execution のチェーン末尾ハッシュ（アンカー）を記録する。週次バッチ（BAT-003）がこのアンカーを起点に全チェーンの整合性を検証する。

| カラム | 型 | 内容 |
|---|---|---|
| `block_date` | DATE | チェーン生成日 |
| `work_execution_id` | UUID | 対象作業実行 ID |
| `chain_tip_hash` | CHAR(64) | チェーン末尾レコードの SHA-256 |
| `event_count` | INTEGER | チェーン内レコード数 |
| `recorded_at` | TIMESTAMPTZ | アンカー記録日時 |

### 3-3. 整合性検証（BAT-003）

- 実行タイミング: 毎週月曜 02:00（CFG-012）
- 検出条件: `prev_hash` の再計算値と格納値の不一致
- 検出時アクション: ERR-SEC-002 を発行 → TBL-032 に `SYSTEM_HASH_BREAK` イベントとして記録 → SCR-MC-008（セキュリティアラート画面）に表示 → system_admin へプッシュ通知

---

## 4. 相関トレーシング（trace_id）

すべての監査イベントは、HTTP リクエストヘッダー `X-Trace-Id`（UUID v7）を `trace_id` として全イベントに記録する。これにより、単一の業務操作に起因する複数イベント（例: ログイン → 作業開始 → ステップ完了 → Outbox 送信）を横断的に追跡できる。

| 規則 | 内容 |
|---|---|
| 生成者 | ハンディ APP がリクエスト発行時に UUID v7 を生成 |
| 伝播 | バックエンド内の全非同期処理（Outbox 処理・バッチ等）も同 trace_id を引き継ぐ |
| ログ記録 | TBL-032 の `trace_id` カラム（CHAR(36)）に必ず記録 |
| ヘッダ検証 | バックエンドは UUID v7 形式でない `X-Trace-Id` を ERR-SYS-002（バリデーションエラー）として拒否 |

---

## 5. 7 年保全ポリシー（NFR-ALC）

| 項目 | 内容 |
|---|---|
| 保全期間 | 7 年（NFR-ALC: 電子記録法規制に準拠） |
| 保全範囲 | TBL-032（audit_logs）・TBL-031（work_events）・hash_chain_blocks・帳票 PDF（暗号化アーカイブ） |
| アーカイブ方式 | 保全対象の月次パーティションを PostgreSQL DETACH PARTITION → pg_dump → AES-256-GCM 暗号化（KEY-004）→ 長期ストレージ（テープ / NAS）に転送 |
| 保全メディア | 書き込み後変更不可（WORM）の NAS または LTO テープを推奨（詳細設計フェーズで確定） |
| 定期確認 | 年 1 回、保全データの復元テストを実施し結果を LOG-008 イベントとして記録 |
| 期限到達時 | 7 年到達分をアーカイブから削除。削除操作は system_admin の二重承認を必須とし LOG-008 に記録 |

---

## 6. XES 形式エクスポート（FR-AU-005）

プロセスマイニング用途のために、監査イベントを XES（eXtensible Event Stream）形式でエクスポートする機能を提供する。

| 項目 | 内容 |
|---|---|
| XES バージョン | XES 2.0（IEEE 1849-2016） |
| Case 概念 | `work_execution_id` = Case ID |
| Activity 概念 | `event_type`（WORK_START / WORK_STEP_COMPLETE 等） |
| Timestamp | `timestamp_server`（UTC ISO 8601） |
| Resource | `operator_id`（UUID、名前は非出力） |
| エクスポート API | `GET /api/v1/audit/export/xes?from={date}&to={date}` |
| アクセス権限 | quality_admin / system_admin のみ |
| 出力形式 | `.xes.gz`（gzip 圧縮） |

XES エクスポートには個人名・worker 名を含めない。`resource` フィールドには UUID のみを出力し、プロセス分析ツール側での名前名寄せを可能にしない（BR-BUS-029 準拠）。

---

**本節で確定した方針**
- **全監査イベントは TBL-032（audit_logs）へ Append-only（INSERT のみ）で記録し、`audit_role` への UPDATE/DELETE 権限を REVOKE することで改ざん防止を技術的に強制する。イベント分類は AUTH_* / WORK_* / MASTER_* / REPORT_* / SYSTEM_* の 5 系統・LOG-001〜010 の識別子体系で管理する。**
- **ハッシュチェーンは SHA-256 + KEY-009（日次 salt）で work_execution 単位に構成し、BAT-003 週次バッチが整合性を検証する。チェーン破断を検出した場合は ERR-SEC-002 → SCR-MC-008 アラート → system_admin 通知の経路でエスカレーションする。**
- **全イベントは `X-Trace-Id`（UUID v7）で相関付け、7 年保全要件（NFR-ALC）に対応するため月次パーティションのアーカイブ → AES-256-GCM 暗号化転送体制を設ける。XES エクスポート（FR-AU-005）は個人名を除外した UUID のみで出力し BR-BUS-029（個人別ランキング禁止）に準拠する。**

---

## 参照業界分析

### 必須
- [`90_業界分析/09_セキュリティとアクセス制御.md`](../../90_業界分析/09_セキュリティとアクセス制御.md)
- [`90_業界分析/21_電子記録の法規制とALCOA+.md`](../../90_業界分析/21_電子記録の法規制とALCOA+.md)

### 関連
- [`90_業界分析/11_電子署名と本人確認.md`](../../90_業界分析/11_電子署名と本人確認.md)
