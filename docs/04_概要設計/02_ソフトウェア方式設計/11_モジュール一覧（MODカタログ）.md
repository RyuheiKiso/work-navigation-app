# 11 モジュール一覧（MOD カタログ）

本章の責務は、全モジュール（MOD-NNN）の責務・依存関係・担当 FR-ID・担当 TBL-ID を一覧化する MOD カタログを確定することである。本章が DTM の MOD 列への入力源となる。

---

## 1. バックエンド（MOD-BE）

| MOD-ID | 物理名（crate/module）| ファイルパス | 責務 | 依存 MOD | 関連 FR |
|---|---|---|---|---|---|
| MOD-BE-001 | wnav_api | `crates/wnav_api/` | axum ルータ・ミドルウェア（認証・Idempotency・レート制限）集約 | MOD-BE-002/005 | 全 API |
| MOD-BE-002 | wnav_domain | `crates/wnav_domain/` | ドメインモデル・サービス・リポジトリ Trait | MOD-BE-004 | 全 FR |
| MOD-BE-003 | wnav_hash_chain | `crates/wnav_hash_chain/` | SHA-256 ハッシュチェーン計算・週次検証 | MOD-BE-002 | FR-EV-001/002 |
| MOD-BE-004 | wnav_db | `crates/wnav_db/` | sqlx クエリ・コネクションプール・リポジトリ実装 | — | 全 TBL |
| MOD-BE-005 | wnav_auth | `crates/wnav_auth/` | JWT RS256 検証・RBAC ミドルウェア | MOD-BE-004 | FR-AU-001〜006 |
| MOD-BE-006 | wnav_outbox | `crates/wnav_outbox/` | Outbox Consumer（BAT-002）・WebSocket/Webhook | MOD-BE-002/004 | FR-SY-002/005 |
| MOD-BE-007 | wnav_webhook | `crates/wnav_webhook/` | Webhook 配信・HMAC-SHA256 署名（IF-002）| MOD-BE-004 | IF-002 |
| MOD-IN-001 | pg_role_init | `src/infra/database/roles/` | DB ロール DDL（app_event_writer 等）| — | NFR-SEC-040 |
| MOD-IN-002 | docker_compose_config | `docker/` | Docker Compose 定義（バックエンド・PostgreSQL）| — | NFR-ENV |
| MOD-BE-008 | wnav_iqc | `crates/wnav_iqc/` | IQC BC（入荷ロット管理・AQL 判定・特採/選別・後工程ゲート）| MOD-BE-002/004 | FR-IQ-001〜019 |
| MOD-BE-009 | wnav_rework | `crates/wnav_rework/` | Rework BC（ディスポジション・リワーク作業・再検査・廃却/返却）| MOD-BE-002/004 | FR-ST-013/014, FR-EV-014/015, FR-MA-017/018 |

---

## 2. ハンディ APP（MOD-FE-HA）

| MOD-ID | 物理名 | ファイルパス | 責務 | 関連 FR | 関連 SCR |
|---|---|---|---|---|---|
| MOD-FE-HA-001 | NetworkProvider | `src/features/network/` | ネットワーク 4 段階管理・Offline-First コンテキスト | FR-SY-008/009 | SCR-HA-015 |
| MOD-FE-HA-002 | OutboxWorker | `src/features/network/outbox/` | 端末 Outbox Consumer（BAT-002 相当）| FR-SY-005/006 | — |
| MOD-FE-HA-003 | StepEngine | `src/features/navigation/step-engine/` | Step 実行・JSON Logic 評価・ロックステップ強制 | FR-NV-001〜013 | SCR-HA-004/005/006 |
| MOD-FE-HA-004 | EvidenceCapture | `src/features/evidence/` | 写真撮影・測定値入力・SHA-256 計算 | FR-EV-001〜012 | SCR-HA-008/009 |
| MOD-FE-HA-005 | SuspensionFlow | `src/features/suspension/` | 中断・再開・プレースキーパー | FR-ST-001〜012 | SCR-HA-011/012 |
| MOD-FE-HA-006 | AndonKaizenFlow | `src/features/kaizen/` | アンドン発報・不適合登録 | FR-KZ-001〜007 | SCR-HA-013/014 |
| MOD-FE-HA-007 | ElectronicSignPad | `src/shared/ui/ElectronicSignPad/` | 電子サイン入力 UI（FR-AU-001）| FR-AU-001 | SCR-HA-010 |
| MOD-FE-HA-008 | LocalDbService | `src/shared/db/` | SQLite + TypeORM 接続・マスタキャッシュ | FR-SY-002〜004 | — |
| MOD-FE-HA-009 | IqcInspectionFlow | `src/features/iqc-inspection/` | IQC 入荷受入・サンプル測定入力・AQL 結果表示 | FR-IQ-001〜008 | SCR-HA-016/017/018 |
| MOD-FE-HA-010 | ReworkFlow | `src/features/rework-flow/` | リワーク作業実施・再検査・廃却/返却確認 | FR-ST-014, FR-EV-015, FR-MA-017 | SCR-HA-019〜022 |

---

## 3. マスタメンテ APP（MOD-FE-MA）

MOD-FE-MA-001（SopEditor）は Step 単体属性の編集を、MOD-FE-MA-002（DslConditionBuilder）は条件式の JSON Logic 編集を、MOD-FE-MA-003（SopFlowEditor）は Step 間の DAG フロー編集を、それぞれ責務分離して担当する。3 モジュールは SCR-MA-004 の SopEditorShell が統合する。

| MOD-ID | 物理名 | ファイルパス | 責務 | 関連 FR | 関連 SCR |
|---|---|---|---|---|---|
| MOD-FE-MA-001 | SopEditor | `src/features/sop-editor/` | SOP/Step 編集・Auto-Save | FR-MA-001〜007 | SCR-MA-004/005 |
| MOD-FE-MA-002 | DslConditionBuilder | `src/features/sop-editor/dsl/` | ビジュアル DSL エディタ・DAG 検証 | FR-MA-004/007 | SCR-MA-006 |
| MOD-FE-MA-003 | SopFlowEditor | `src/features/sop-editor/flow/` | Step ノード/エッジの DAG 編集・循環参照プレフライト・フローシミュレーション | FR-MA-016 | SCR-MA-004 (DAG フローモード) |
| MOD-FE-MA-004 | ApprovalWorkflow | `src/features/approval/` | レビュー・承認・公開フロー | FR-MA-008〜010 | SCR-MA-007/008/009 |
| MOD-FE-MA-005 | MasterVersionDiff | `src/features/sop-editor/diff/` | バージョン差分表示 | FR-MA-013 | SCR-MA-010 |
| MOD-FE-MA-006 | UserRoleAdmin | `src/features/user-mgmt/` | ユーザー/ロール CRUD | FR-MA-014/015 | — |
| MOD-FE-MA-007 | MaterialMasterEditor | `src/features/material-master/` | 材料マスタ CRUD・版管理 | FR-MA-017, FR-IQ-001 | SCR-MA-012 |
| MOD-FE-MA-008 | SupplierMasterEditor | `src/features/supplier-master/` | 仕入先マスタ CRUD・品質実績リンク | FR-MA-018, FR-IQ-014 | SCR-MA-013 |
| MOD-FE-MA-009 | SamplingPlanEditor | `src/features/sampling-plan/` | AQL 値・検査水準設定・JSONB スナップショット確認 | FR-IQ-003, FR-IQ-004 | SCR-MA-014 |
| MOD-FE-MA-010 | ReworkSopEditor | `src/features/rework-sop-editor/` | sop_type=REWORK の SOP 管理・不適合カテゴリマッピング | FR-MA-017, FR-ST-014 | SCR-MA-015/016 |

---

## 4. 管理コンソール（MOD-FE-MC）

| MOD-ID | 物理名 | ファイルパス | 責務 | 関連 FR | 関連 SCR |
|---|---|---|---|---|---|
| MOD-FE-MC-001 | OperationDashboard | `src/features/dashboard/` | 運用ダッシュボード・SLI 表示 | OPS-036〜053 | SCR-MC-001 |
| MOD-FE-MC-002 | AuditLogViewer | `src/features/audit/` | 監査ログ閲覧・XES エクスポート | FR-AU-004/005 | SCR-MC-004/005 |
| MOD-FE-MC-003 | HashChainVerifier | `src/features/hash-verify/` | ハッシュチェーン検証結果表示 | FR-AU-006 | SCR-MC-008 |
| MOD-FE-MC-004 | OutboxMonitor | `src/features/outbox-mon/` | DLQ 監視・手動再投入 | FR-SY-007/008 | SCR-MC-007 |
| MOD-FE-MC-005 | ReportGenerator | `src/features/reports/` | 帳票出力（RP-001〜010）| RP-001〜010 | SCR-MC-009 |
| MOD-FE-MC-006 | ConcessionApprovalConsole | `src/features/concession-approval/` | 特採申請一覧・電子サイン・有効期限設定 | FR-IQ-010 | SCR-MC-010 |
| MOD-FE-MC-007 | IqcDashboard | `src/features/iqc-dashboard/` | 仕入先品質実績・厳しさ状態・不合格率推移（個人別集計禁止）| FR-IQ-014, FR-IQ-015 | SCR-MC-011/012 |
| MOD-FE-MC-008 | DispositionApprovalConsole | `src/features/disposition-approval/` | ディスポジション判定・品質担当+監督の二者サイン UI | FR-ST-013, FR-EV-014 | SCR-MC-013 |
| MOD-FE-MC-009 | ReworkTraceabilityViewer | `src/features/rework-trace/` | parent_case_id ↔ rework_case_id 双方向照会・ALCOA+ Original 確認 | FR-KZ-009, FR-KZ-010 | SCR-MC-014/015 |

---

## 5. 共通（MOD-SH）

| MOD-ID | 物理名 | 責務 | 関連 FR |
|---|---|---|---|
| MOD-SH-001 | LocaleResolver | i18n（ja/en/ja-simple）・翻訳ロード | FR-UI-001〜003 |
| MOD-SH-002 | IdGenerator | UUID v7 生成・Idempotency Key | 全 API |
| MOD-SH-003 | ClockService | 時刻抽象（テスト替え可能）| NFR-DQ（Contemporaneous）|
| MOD-SH-004 | ApiClient | OpenAPI 生成クライアント・エラーハンドリング共通 | 全 API |

---

**本節で確定した方針**
- **MOD-BE（9+2）・MOD-FE-HA（10）・MOD-FE-MA（10）・MOD-FE-MC（9）・MOD-SH（4）の計 44 モジュールを確定し、責務・依存関係・担当 FR を一元管理した。**
- **依存方向はバックエンド: wnav_api → wnav_domain ← wnav_db、フロントエンド: features → shared（一方向）で統一し、循環依存を禁止する。**
- **IQC/リワーク対応で MOD-BE-008（wnav_iqc）・MOD-BE-009（wnav_rework）の 2 BC クレート、MOD-FE-HA-009/010・MOD-FE-MA-007〜010・MOD-FE-MC-006〜009 の計 12 モジュールを追加した。**

---

## 参照業界分析

### 必須
- [`90_業界分析/07_スマートファクトリーと作業のデジタル化.md`](../../90_業界分析/07_スマートファクトリーと作業のデジタル化.md)
