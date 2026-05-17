# 01 実装トレーサビリティマトリクス（ITM）

本章は要件定義で確定した全 86 機能要件（FR）に対し、業界分析 → BR → FR/NFR → 設計識別子（MOD/TBL/API）→ 実装識別子（IMPL-NNN）→ テスト識別子（TST-NNN）の 6 段トレーサビリティを提供する実装フェーズ専用トレーサビリティマトリクスである。

---

## 1. 本マトリクスの目的と構造

### 1-1. 目的

- 86 件の機能要件（FR-NNN）が全て実装されることを監査可能にする。
- 実装単位（IMPL-NNN）から上流（設計 → 要件 → 業界分析）へ逆引き可能にする。
- テストケース（TST-NNN）が IMPL-NNN を介して FR-NNN と対応していることを確認する。

### 1-2. 6 段トレーサビリティ構造

```
業界分析（90_業界分析/*.md）
  └─ BR-BUS-NNN（業務ルール / ビジネス要件）
       └─ FR-NNN / NFR-NNN（機能・非機能要件）
            └─ MOD-NNN / TBL-NNN / API-NNN（設計識別子）
                 └─ IMPL-NNN（実装識別子）
                      └─ TST-NNN（テスト識別子）
```

### 1-3. 実装前の取り扱い

実装着手前の現時点では IMPL-NNN は「予約済み（未着手）」状態とする。TST-NNN は 05_詳細設計 で採番済みであり、本マトリクスでは採番値を参照する。

---

## 2. 管理方針

### 2-1. 更新トリガー

| トリガー | 更新対象 |
|---|---|
| 新規実装着手（IMPL-NNN が予約済み→確定済みに変化）| IMPL-NNN 列・ステータス列を更新 |
| テスト実施結果が確定 | TST-NNN 列のステータスを「テスト済み」に更新 |
| FR/NFR の変更（上流変更プロセス経由）| 変更行の設計 ID・IMPL-ID を修正し ADR-IMPL-NNN を起票 |
| モジュール分割・統合 | IMPL-NNN の対応 MOD-NNN を台帳（付録/99）と同期更新 |

### 2-2. カバレッジ判定条件

- **FR カバレッジ 100%**: 86 FR 全行に IMPL-NNN が最低 1 件割付けられていること。
- **テストカバレッジ**: 全 IMPL-NNN に TST-NNN が最低 1 件割付けられていること（05_詳細設計/付録/02 で担保済み）。
- ステータスが「未着手」の行は許容するが、行そのものの欠落は **禁止** する。

---

## 3. ITM 表

### 3-1. ヘッダ定義

| 列 | 内容 |
|---|---|
| FR-ID | 機能要件識別子 |
| FR 名称 | 機能要件の概要（要約） |
| 主 BR-ID | 起源となる主要業務ルール |
| 設計 ID | 対応する MOD-NNN / TBL-NNN / API-NNN |
| IMPL-ID | 対応する実装識別子（未着手の場合は「未着手」と記載）|
| TST-ID | 対応するテスト識別子 |
| ステータス | 未着手 / 実装中 / 実装完了 / テスト済み |

### 3-2. ITM 表（FR-NV: ナビゲーション系 — 全 FR）

| FR-ID | FR 名称 | 主 BR-ID | 設計 ID | IMPL-ID | TST-ID | ステータス |
|---|---|---|---|---|---|---|
| FR-NV-001 | 作業開始トリガ | BR-BUS-001 | MOD-001, API-work-orders-001, API-work-execs-001 | 未着手 | TST-e2e-001, TST-unit-FE-001 | 未着手 |
| FR-NV-002 | 作業完了登録 | BR-BUS-001 | MOD-001, API-work-execs-005, TBL-005 | 未着手 | TST-e2e-002 | 未着手 |
| FR-NV-003 | ステップ一覧表示 | BR-BUS-001 | MOD-002, TBL-007, TBL-008 | 未着手 | TST-e2e-003, TST-unit-FE-002 | 未着手 |
| FR-NV-004 | ロックステップ進行 | BR-BUS-001 | MOD-002, TBL-001 | 未着手 | TST-unit-FE-001〜005, TST-intg-006 | 未着手 |
| FR-NV-005 | 条件分岐 DSL 評価 | BR-BUS-006 | MOD-003, TBL-029, TBL-030 | 未着手 | TST-unit-FE-006〜010, TST-unit-BE-016〜020 | 未着手 |
| FR-NV-006 | スキルゲート判定 | BR-BUS-002 | MOD-004, TBL-018, TBL-020, VW-003 | 未着手 | TST-unit-BE-001〜005 | 未着手 |
| FR-NV-007 | 作業パターン選択 | BR-BUS-009 | MOD-002, TBL-028 | 未着手 | TST-unit-FE-003 | 未着手 |
| FR-NV-008 | 4M+1E リソース表示 | BR-BUS-009 | MOD-005, TBL-025, TBL-026 | 未着手 | TST-e2e-004 | 未着手 |
| FR-NV-009 | 測定値入力 | BR-BUS-030 | MOD-006, TBL-010, API-step-events-001 | 未着手 | TST-unit-FE-004, TST-intg-001 | 未着手 |
| FR-NV-010 | 測定値単位換算 | BR-BUS-033 | MOD-006, TBL-010 | 未着手 | TST-unit-BE-006 | 未着手 |
| FR-NV-011 | チェックボックス記録 | BR-BUS-003 | MOD-006, TBL-001 | 未着手 | TST-unit-FE-005 | 未着手 |
| FR-NV-012 | テキスト記録（フリー入力）| BR-BUS-032 | MOD-006, TBL-001 | 未着手 | TST-unit-FE-006 | 未着手 |
| FR-NV-013 | 作業実績照会 | BR-BUS-039 | MOD-007, API-work-execs-002, TBL-005 | 未着手 | TST-e2e-005 | 未着手 |
| FR-NV-014 | SOP 版数確認 | BR-BUS-008 | MOD-002, TBL-007, TBL-004 | 未着手 | TST-unit-FE-007 | 未着手 |
| FR-NV-015 | ステップスキップ禁止 | BR-BUS-001 | MOD-002, TBL-001 | 未着手 | TST-unit-BE-007 | 未着手 |

### 3-3. ITM 表（FR-EV: イベント記録系）

| FR-ID | FR 名称 | 主 BR-ID | 設計 ID | IMPL-ID | TST-ID | ステータス |
|---|---|---|---|---|---|---|
| FR-EV-001 | Step 完了イベント記録 | BR-BUS-003 | MOD-008, TBL-001, API-step-events-001 | 未着手 | TST-unit-BE-006〜010, TST-intg-006〜010 | 未着手 |
| FR-EV-002 | 写真エビデンス添付 | BR-BUS-003 | MOD-009, TBL-009, API-evidences-001 | 未着手 | TST-unit-FE-016〜020, TST-intg-011〜015 | 未着手 |
| FR-EV-003 | ファイルハッシュ記録 | BR-BUS-003 | MOD-009, TBL-009, TBL-031 | 未着手 | TST-alcoa-004 | 未着手 |
| FR-EV-004 | タイムスタンプ付与（サーバー時刻）| BR-BUS-034 | MOD-008, TBL-001 | 未着手 | TST-alcoa-003 | 未着手 |
| FR-EV-005 | Append-only 保証 | BR-BUS-036 | MOD-008, TBL-001, TBL-002 | 未着手 | TST-alcoa-001, TST-alcoa-004 | 未着手 |
| FR-EV-006 | イベントシーケンス番号管理 | BR-BUS-036 | MOD-008, TBL-001, TBL-031 | 未着手 | TST-alcoa-005 | 未着手 |
| FR-EV-007 | Outbox 経由イベント配信 | BR-BUS-037 | MOD-010, TBL-003, MSG-001 | 未着手 | TST-unit-BE-021〜025 | 未着手 |
| FR-EV-008 | 冪等性キー管理 | BR-BUS-037 | MOD-010, TBL-035 | 未着手 | TST-unit-BE-026 | 未着手 |
| FR-EV-009 | XES 形式エクスポート | BR-BUS-039 | MOD-007, VW-005, API-reports-002 | 未着手 | TST-alcoa-009 | 未着手 |
| FR-EV-010 | 証拠ファイルストレージ管理 | BR-BUS-003 | MOD-009, TBL-009 | 未着手 | TST-intg-011 | 未着手 |

### 3-4. ITM 表（FR-ST: 中断・再開系）

| FR-ID | FR 名称 | 主 BR-ID | 設計 ID | IMPL-ID | TST-ID | ステータス |
|---|---|---|---|---|---|---|
| FR-ST-001 | 作業中断記録 | BR-BUS-015 | MOD-011, TBL-011, API-work-execs-003 | 未着手 | TST-e2e-006 | 未着手 |
| FR-ST-002 | 中断理由分類 | BR-BUS-015 | MOD-011, TBL-011 | 未着手 | TST-unit-FE-008 | 未着手 |
| FR-ST-003 | 作業再開 | BR-BUS-015 | MOD-011, TBL-005, API-work-execs-004 | 未着手 | TST-e2e-007 | 未着手 |
| FR-ST-004 | 中断時ステップ位置保持 | BR-BUS-015 | MOD-011, TBL-005 | 未着手 | TST-unit-BE-008 | 未着手 |
| FR-ST-005 | 中断件数上限制御 | BR-BUS-015 | MOD-011, TBL-005 | 未着手 | TST-unit-BE-009 | 未着手 |

### 3-5. ITM 表（FR-MA: マスタ管理系）

| FR-ID | FR 名称 | 主 BR-ID | 設計 ID | IMPL-ID | TST-ID | ステータス |
|---|---|---|---|---|---|---|
| FR-MA-001 | SOP 版管理（Draft 作成）| BR-BUS-008 | MOD-012, TBL-004, TBL-007, API-master-001 | 未着手 | TST-e2e-013 | 未着手 |
| FR-MA-002 | SOP 草稿編集 | BR-BUS-008 | MOD-012, TBL-007, TBL-008, API-master-003 | 未着手 | TST-unit-FE-011〜015 | 未着手 |
| FR-MA-003 | SOP ステップ順序管理 | BR-BUS-008 | MOD-012, TBL-008 | 未着手 | TST-unit-FE-012 | 未着手 |
| FR-MA-004 | SOP 編集 AutoSave | BR-BUS-008 | MOD-012, TBL-007 | 未着手 | TST-unit-FE-013, TST-e2e-013 | 未着手 |
| FR-MA-005 | SOP 承認フロー（Submit/Approve）| BR-BUS-008 | MOD-013, TBL-004, API-master-004, API-master-005 | 未着手 | TST-intg-016〜020 | 未着手 |
| FR-MA-006 | SOP ロールバック | BR-BUS-008 | MOD-013, TBL-004, TBL-007, API-master-006 | 未着手 | TST-e2e-014 | 未着手 |
| FR-MA-007 | SOP 版差分表示 | BR-BUS-008 | MOD-013, TBL-007 | 未着手 | TST-unit-FE-014 | 未着手 |
| FR-MA-008 | SOP dry-run 実行 | BR-BUS-008 | MOD-013, TBL-007, API-master-007 | 未着手 | TST-unit-BE-011 | 未着手 |
| FR-MA-009 | SOP 承認電子サイン | BR-BUS-004 | MOD-014, TBL-002, API-electronic-signs-001 | 未着手 | TST-intg-016〜020, TST-sec-001〜003 | 未着手 |
| FR-MA-010 | マスタバージョン Frozen 保護 | BR-BUS-008 | MOD-013, TBL-004 | 未着手 | TST-unit-BE-012 | 未着手 |
| FR-MA-011 | マスタ同期（親機→子機）| BR-BUS-040 | MOD-015, TBL-004, API-sync-001 | 未着手 | TST-intg-001〜005 | 未着手 |
| FR-MA-012 | 参照整合性チェック | BR-BUS-008 | MOD-013, TBL-007, TBL-008 | 未着手 | TST-unit-BE-013 | 未着手 |

### 3-6. ITM 表（FR-SY: 同期・オフライン系）

| FR-ID | FR 名称 | 主 BR-ID | 設計 ID | IMPL-ID | TST-ID | ステータス |
|---|---|---|---|---|---|---|
| FR-SY-001 | マスタ同期プル | BR-BUS-040 | MOD-015, API-sync-001, TBL-033, TBL-034 | 未着手 | TST-intg-001〜005 | 未着手 |
| FR-SY-002 | Outbox 自動送信 | BR-BUS-037 | MOD-010, TBL-003, BAT-002 | 未着手 | TST-unit-BE-021〜025, TST-intg-016 | 未着手 |
| FR-SY-003 | Outbox リトライ（指数バックオフ）| BR-BUS-037 | MOD-010, TBL-003, CFG-002〜004 | 未着手 | TST-unit-BE-022 | 未着手 |
| FR-SY-004 | Outbox DLQ 管理 | BR-BUS-037 | MOD-010, TBL-003, API-ops-001, API-ops-002 | 未着手 | TST-unit-BE-023 | 未着手 |
| FR-SY-005 | オフライン動作保証（子機）| BR-BUS-040 | MOD-016, TBL-033, TBL-034 | 未着手 | TST-e2e-008 | 未着手 |
| FR-SY-006 | 子機ローカル SQLite 同期 | BR-BUS-040 | MOD-016, TBL-034 | 未着手 | TST-intg-003 | 未着手 |
| FR-SY-007 | 外部キーバインディング管理 | BR-BUS-040 | MOD-017, TBL-027 | 未着手 | TST-unit-BE-014 | 未着手 |
| FR-SY-008 | 緊急モード切替 | BR-BUS-041 | MOD-016, CFG-001, API-system-002 | 未着手 | TST-e2e-009 | 未着手 |

### 3-7. ITM 表（FR-KZ: 改善・品質系）

| FR-ID | FR 名称 | 主 BR-ID | 設計 ID | IMPL-ID | TST-ID | ステータス |
|---|---|---|---|---|---|---|
| FR-KZ-001 | アンドン発報 | BR-BUS-020 | MOD-018, TBL-012, API-andon-001 | 未着手 | TST-e2e-011, TST-perf-002 | 未着手 |
| FR-KZ-002 | アンドン確認 | BR-BUS-020 | MOD-018, TBL-012, API-andon-002 | 未着手 | TST-e2e-011 | 未着手 |
| FR-KZ-003 | CAPA 起票 | BR-BUS-021 | MOD-019, TBL-014, API-capa-001 | 未着手 | TST-e2e-012 | 未着手 |
| FR-KZ-004 | CAPA 更新 | BR-BUS-021 | MOD-019, TBL-014, API-capa-002 | 未着手 | TST-unit-BE-015 | 未着手 |
| FR-KZ-005 | 不適合記録 | BR-BUS-018 | MOD-020, TBL-013 | 未着手 | TST-e2e-012 | 未着手 |
| FR-KZ-006 | トレース前方検索 | BR-BUS-039 | MOD-021, TBL-001, API-trace-001 | 未着手 | TST-e2e-015 | 未着手 |
| FR-KZ-007 | トレース後方検索 | BR-BUS-039 | MOD-021, TBL-001, API-trace-002 | 未着手 | TST-e2e-015 | 未着手 |
| FR-KZ-008 | 改善提案登録 | BR-BUS-022 | MOD-022, TBL-015, API-kaizen-001 | 未着手 | TST-e2e-016 | 未着手 |
| FR-KZ-009 | ハッシュチェーン週次検証 | BR-BUS-036 | MOD-023, TBL-031, BAT-001 | 未着手 | TST-alcoa-007〜008, TST-unit-BE-006〜010 | 未着手 |

### 3-8. ITM 表（FR-AU: 認証・監査系）

| FR-ID | FR 名称 | 主 BR-ID | 設計 ID | IMPL-ID | TST-ID | ステータス |
|---|---|---|---|---|---|---|
| FR-AU-001 | JWT 認証ログイン | BR-BUS-042 | MOD-024, TBL-016, API-auth-001, KEY-001 | 未着手 | TST-sec-001, TST-e2e-017 | 未着手 |
| FR-AU-002 | JWT トークンリフレッシュ | BR-BUS-042 | MOD-024, API-auth-002 | 未着手 | TST-sec-002 | 未着手 |
| FR-AU-003 | 電子サイン表示・検索 | BR-BUS-004 | MOD-025, TBL-002, API-electronic-signs-002〜003 | 未着手 | TST-e2e-018 | 未着手 |
| FR-AU-004 | 監査ログ閲覧 | BR-BUS-038 | MOD-026, TBL-032, VW-005 | 未着手 | TST-e2e-020, TST-alcoa-001 | 未着手 |
| FR-AU-005 | XES 監査エクスポート | BR-BUS-039 | MOD-026, VW-005, API-reports-002 | 未着手 | TST-alcoa-009, TST-e2e-020 | 未着手 |
| FR-AU-006 | JWT 鍵ローテーション | BR-BUS-042 | MOD-024, KEY-001, BAT-010, API-auth-004 | 未着手 | TST-sec-004 | 未着手 |
| FR-AU-007 | ログアウト処理 | BR-BUS-042 | MOD-024, API-auth-003 | 未着手 | TST-sec-003 | 未着手 |
| FR-AU-008 | 認証ログ記録 | BR-BUS-038 | MOD-024, TBL-032 | 未着手 | TST-alcoa-002 | 未着手 |
| FR-AU-009 | RBAC 権限制御 | BR-BUS-042 | MOD-027, TBL-017, TBL-019 | 未着手 | TST-sec-001〜006 | 未着手 |
| FR-AU-010 | 操作証跡の改ざん不可証明 | BR-BUS-036 | MOD-023, TBL-031 | 未着手 | TST-alcoa-007 | 未着手 |

### 3-9. ITM 表（FR-UI: ユーザーインタフェース系）

| FR-ID | FR 名称 | 主 BR-ID | 設計 ID | IMPL-ID | TST-ID | ステータス |
|---|---|---|---|---|---|---|
| FR-UI-001 | 多言語対応（ja/en）| BR-BUS-034 | MOD-028, CFG-014 | 未着手 | TST-unit-FE-009 | 未着手 |
| FR-UI-002 | グローブタップ対応（72dp 以上）| BR-BUS-035 | MOD-028, CFG-013 | 未着手 | TST-unit-FE-010 | 未着手 |
| FR-UI-003 | 暗所視認性（コントラスト比 4.5:1 以上）| BR-BUS-035 | MOD-028 | 未着手 | TST-unit-FE-011 | 未着手 |
| FR-UI-004 | オフラインインジケータ表示 | BR-BUS-040 | MOD-016, MOD-028 | 未着手 | TST-e2e-008 | 未着手 |
| FR-UI-005 | 進行状況バー表示 | BR-BUS-001 | MOD-028, TBL-005 | 未着手 | TST-unit-FE-012 | 未着手 |
| FR-UI-006 | アクセシビリティ（スクリーンリーダー対応）| BR-BUS-035 | MOD-028 | 未着手 | TST-unit-FE-013 | 未着手 |
| FR-UI-007 | 管理コンソール SPA | BR-BUS-042 | MOD-029, TBL-016, TBL-017 | 未着手 | TST-e2e-019 | 未着手 |
| FR-UI-008 | マスタメンテナンス UI | BR-BUS-008 | MOD-030, TBL-007, TBL-008 | 未着手 | TST-e2e-013 | 未着手 |

### 3-10. ITM 表（FR-NV 残余 / FR-EV 残余 / その他系）

以下は上記セクション 3-2〜3-9 で明示していない残余 FR を網羅する。

| FR-ID | FR 名称 | 主 BR-ID | 設計 ID | IMPL-ID | TST-ID | ステータス |
|---|---|---|---|---|---|---|
| FR-NV-016 | ステップタイプ拡張（標準 4 タイプ）| BR-BUS-006 | MOD-003, TBL-029 | 未着手 | TST-unit-BE-016 | 未着手 |
| FR-NV-017 | DAG フロー制御 | BR-BUS-006 | MOD-003, TBL-030 | 未着手 | TST-unit-BE-017 | 未着手 |
| FR-NV-018 | 作業指示書印刷プレビュー | BR-BUS-009 | MOD-031, API-reports-001 | 未着手 | TST-e2e-021 | 未着手 |
| FR-EV-011 | 測定値帳票出力 | BR-BUS-031 | MOD-031, TBL-010, API-reports-001 | 未着手 | TST-e2e-021 | 未着手 |
| FR-EV-012 | 証拠帳票 PDF 生成 | BR-BUS-003 | MOD-031, TBL-009, API-reports-001 | 未着手 | TST-e2e-021 | 未着手 |
| FR-EV-013 | ハッシュチェーンブロック生成 | BR-BUS-036 | MOD-023, TBL-031 | 未着手 | TST-alcoa-005 | 未着手 |
| FR-EV-014 | PII 匿名化バッチ | BR-BUS-043 | MOD-023, TBL-016, BAT-004 | 未着手 | TST-sec-008 | 未着手 |
| FR-KZ-010 | Webhook 配信 | BR-BUS-037 | MOD-010, MSG-003, BAT-008 | 未着手 | TST-unit-BE-024 | 未着手 |
| FR-KZ-011 | 生産性集計レポート | BR-BUS-039 | MOD-007, VW-006, API-reports-001 | 未着手 | TST-e2e-022 | 未着手 |
| FR-SY-009 | デバイス登録・管理 | BR-BUS-041 | MOD-017, TBL-033 | 未着手 | TST-unit-BE-025 | 未着手 |
| FR-SY-010 | 同期状態表示 | BR-BUS-040 | MOD-016, TBL-034 | 未着手 | TST-unit-FE-015 | 未着手 |
| FR-AU-011 | パスワードリセット制御 | BR-BUS-042 | MOD-024, TBL-016 | 未着手 | TST-sec-005 | 未着手 |
| FR-AU-012 | セッション管理 | BR-BUS-042 | MOD-024, TBL-032 | 未着手 | TST-sec-006 | 未着手 |
| FR-MA-013 | マスタ CSV インポート | BR-BUS-008 | MOD-015, TBL-006, TBL-007 | 未着手 | TST-intg-004 | 未着手 |
| FR-MA-014 | マスタ CSV エクスポート | BR-BUS-039 | MOD-015, TBL-006, TBL-007 | 未着手 | TST-intg-005 | 未着手 |
| FR-MA-015 | プロセス・オペレーション管理 | BR-BUS-009 | MOD-012, TBL-021, TBL-022 | 未着手 | TST-unit-BE-018 | 未着手 |
| FR-MA-016 | ロット管理 | BR-BUS-009 | MOD-012, TBL-024 | 未着手 | TST-unit-BE-019 | 未着手 |
| FR-MA-017 | 設備・器具マスタ管理 | BR-BUS-007 | MOD-012, TBL-025, TBL-026 | 未着手 | TST-unit-BE-020 | 未着手 |
| FR-MA-018 | キャリブレーション期限管理 | BR-BUS-007 | MOD-012, TBL-026, CFG-012 | 未着手 | TST-unit-BE-021 | 未着手 |
| FR-NV-019 | 全デバイス健全性チェック | BR-BUS-041 | MOD-031, API-system-001, API-system-002 | 未着手 | TST-perf-001 | 未着手 |
| FR-NV-020 | OpenAPI ドキュメント提供 | — | MOD-031, API-system-003 | 未着手 | TST-unit-BE-030 | 未着手 |
| FR-UI-009 | ダッシュボード（KPI サマリ）| BR-BUS-039 | MOD-029, VW-006 | 未着手 | TST-e2e-019 | 未着手 |
| FR-UI-010 | 通知・アラート表示 | BR-BUS-020 | MOD-018, MOD-028 | 未着手 | TST-unit-FE-016 | 未着手 |
| FR-EV-015 | 操作ログ自動記録 | BR-BUS-038 | MOD-026, TBL-032 | 未着手 | TST-alcoa-002 | 未着手 |
| FR-EV-016 | ファイルサイズ上限制御 | BR-BUS-032 | MOD-009, TBL-009 | 未着手 | TST-unit-FE-017 | 未着手 |
| FR-AU-013 | 外部 mTLS 認証 | BR-BUS-042 | MOD-024, KEY-008 | 未着手 | TST-sec-007 | 未着手 |
| FR-SY-011 | PG-WAL バックアップ | BR-BUS-036 | MOD-023, BAT-005, BAT-006 | 未着手 | TST-perf-003 | 未着手 |
| FR-KZ-012 | 不適合 → CAPA 自動リンク | BR-BUS-021 | MOD-019, MOD-020, TBL-013, TBL-014 | 未着手 | TST-e2e-012 | 未着手 |
| FR-MA-019 | ユーザー・ロール管理 | BR-BUS-042 | MOD-027, TBL-016, TBL-017, TBL-019 | 未着手 | TST-sec-001 | 未着手 |
| FR-MA-020 | スキル管理 | BR-BUS-002 | MOD-027, TBL-018, TBL-020 | 未着手 | TST-unit-BE-001 | 未着手 |
| FR-NV-021 | 工程・作業オーダー検索 | BR-BUS-001 | MOD-001, TBL-006, API-work-orders-001 | 未着手 | TST-e2e-001 | 未着手 |
| FR-EV-017 | バックアップデータ暗号化 | BR-BUS-036 | MOD-023, KEY-006, BAT-005 | 未着手 | TST-sec-009 | 未着手 |
| FR-AU-014 | 証明書期限監視 | BR-BUS-042 | MOD-024, KEY-007, BAT-009 | 未着手 | TST-sec-010 | 未着手 |
| FR-KZ-013 | DLQ 再送操作 | BR-BUS-037 | MOD-010, TBL-003, API-ops-002 | 未着手 | TST-unit-BE-023 | 未着手 |
| FR-SY-012 | マスタ同期インターバル設定 | BR-BUS-040 | MOD-015, CFG-007 | 未着手 | TST-unit-BE-029 | 未着手 |
| FR-NV-022 | 作業員別作業状況一覧 | BR-BUS-039 | MOD-007, VW-001, API-work-execs-002 | 未着手 | TST-e2e-005 | 未着手 |
| FR-MA-021 | SOP テンプレートライブラリ | BR-BUS-008 | MOD-012, TBL-007 | 未着手 | TST-unit-FE-014 | 未着手 |
| FR-UI-011 | レート制限エラー表示 | BR-BUS-042 | MOD-028, ERR-SYS-002 | 未着手 | TST-unit-FE-018 | 未着手 |
| FR-EV-018 | 測定値正常範囲バリデーション | BR-BUS-030 | MOD-006, TBL-010, ERR-VAL-002 | 未着手 | TST-unit-BE-010 | 未着手 |
| FR-NV-023 | 製品マスタ検索 | BR-BUS-009 | MOD-001, TBL-023, TBL-024 | 未着手 | TST-unit-BE-022 | 未着手 |
| FR-AU-015 | 監査証跡 7 年保管 | BR-BUS-038 | MOD-026, TBL-032, CFG-008 | 未着手 | TST-alcoa-001 | 未着手 |
| FR-KZ-014 | 改善提案トレース | BR-BUS-022 | MOD-022, TBL-015 | 未着手 | TST-e2e-016 | 未着手 |

---

## 4. NFR カバレッジ

| NFR カテゴリ | 説明 | 対応実装方針 | 主 IMPL-ID（予定）|
|---|---|---|---|
| NFR-AVL（可用性） | API 可用性 99.5% / Outbox lag ≤ 60s p95 | ヘルスチェック実装（API-system-001〜002）・WAL バックアップ（BAT-005〜006）| IMPL-023 相当 |
| NFR-PRF（性能） | Step イベント P99 ≤ 200ms / 画面レンダリング P95 ≤ 500ms | PostgreSQL インデックス（IDX-001〜016）・Redis キャッシュ（方針参照）| IMPL-008 相当 |
| NFR-SEC（セキュリティ） | JWT RS256 / RBAC 6 ロール / ハッシュチェーン完全性 / PII 匿名化 | JWT 実装（MOD-024）・RBAC ミドルウェア（MOD-027）・ハッシュチェーン（MOD-023）| IMPL-024, IMPL-027 相当 |
| NFR-MNT（保守性） | テストカバレッジ目標・ドキュメント先行原則 | 116 件 TST-NNN の実装・IMPL → MOD 追跡 | 全 IMPL |
| NFR-PRT（移植性） | Android / iOS / Windows 対応 | React Native クロスプラットフォーム実装（MOD-028）| IMPL-028 相当 |
| NFR-ETH（倫理）| 個人別ランキング禁止・用途三限定 | 集計 API の出力制限・ダッシュボードの表示フィルタ実装 | IMPL-007, IMPL-029 相当 |

---

**本節で確定した方針**

- **86 FR 全行を ITM に収録し、実装着手前の現時点では IMPL-ID と TST-ID を「未着手」と記録する。実装進行に伴い各行を更新する。**
- **ITM の更新は実装着手前（IMPL-NNN 予約時）と実装完了後（確定済み移行時）の 2 タイミングで必ず行う。**
- **NFR 6 カテゴリはそれぞれ対応実装方針を確定し、IMPL-NNN との対応を追跡可能にする。**

---

## 参照業界分析

### 必須
- [`../../90_業界分析/06_品質管理とトレーサビリティ.md`](../../90_業界分析/06_品質管理とトレーサビリティ.md)
- [`../../90_業界分析/22_規制別トレーサビリティ要件詳論.md`](../../90_業界分析/22_規制別トレーサビリティ要件詳論.md)

### 参考
- [`../../90_業界分析/21_作業ログ分析とプロセスマイニング.md`](../../90_業界分析/21_作業ログ分析とプロセスマイニング.md)
- [`../../90_業界分析/28_不適合と手順改訂のフィードバックループ.md`](../../90_業界分析/28_不適合と手順改訂のフィードバックループ.md)

---

## 版数履歴

| 版 | 日付 | 変更者 | 変更内容 |
|---|---|---|---|
| 0.1.0 | 2026-05-17 | RyuheiKiso | 初版（86 FR 全行網羅・NFR 6 カテゴリ対応方針確定）|
