# 03 インデックス詳細設計（IDX カタログ）

本章の責務は、IDX-001〜016 の全インデックス定義を CREATE INDEX 全文で確定することである。`04_概要設計/04_データ設計/06_インデックス・パーティション・アーカイブ方式.md` §1 で方針を定めたインデックスを物理定義に落とし、対象列・インデックス種別・Partial 条件・パーティション対応・根拠 NFR を全件記述する。

---

## 1. インデックス設計原則

### 1-1. 適用ルール

- NFR-PRF-001（Step 完了応答 P95 200ms）を達成するため `work_events` への検索インデックスを最優先とする
- Partial インデックスで不要行を除外し、インデックスサイズと INSERT コストを低減する
- Append-only テーブル（work_events 等）は月次パーティション子テーブルにも自動的にインデックスが継承される（`CREATE INDEX ON` 親テーブルで全パーティションに適用）
- GIN インデックスは JSONB 列の `@>` 演算子（包含検索）にのみ使用する
- BRIN インデックスは自然挿入順（時系列）が保証される Append-only テーブルの大量行対象列に使用する

### 1-2. パーティション対応

work_events は月次 RANGE パーティションのため、親テーブルに `CREATE INDEX` することで全パーティション子テーブルに自動適用される。インデックス名は親テーブルと子テーブルで自動的に `{name}_{partition_suffix}` 形式になる。

---

## 2. IDX カタログ（IDX-001〜016）

### IDX-001: work_events.case_id B-Tree（最優先）

```sql
-- IDX-001: TBL-001 work_events — case_id B-Tree
-- 目的: 同一作業セッション（XES Case）のイベント一覧検索（NFR-PRF-001: P95 200ms）
-- パーティション適用: 全月次パーティションに自動継承
CREATE INDEX CONCURRENTLY idx_work_events_case_id
    ON work_events USING BTREE (case_id);

COMMENT ON INDEX idx_work_events_case_id IS
    'IDX-001 — work_events を case_id（XES Case ID）で検索するためのインデックス。StepEngine がステップ完了処理で当該セッションの最終イベントを取得する際に使用。NFR-PRF-001 達成の主要手段。';
```

### IDX-002: work_events.timestamp_server B-Tree（時系列範囲検索）

```sql
-- IDX-002: TBL-001 work_events — timestamp_server B-Tree
-- 目的: 時系列範囲クエリ（監査ログ・バッチ処理の期間指定）
-- パーティション適用: パーティション境界と一致するため高効率
CREATE INDEX CONCURRENTLY idx_work_events_timestamp_server
    ON work_events USING BTREE (timestamp_server DESC);

COMMENT ON INDEX idx_work_events_timestamp_server IS
    'IDX-002 — timestamp_server の降順 B-Tree。最新イベント取得（hash_chain 検証の起点取得）と時系列範囲クエリに使用。パーティションプルーニングと組み合わせて高効率。';
```

### IDX-003: work_events.resource B-Tree（Partial）

```sql
-- IDX-003: TBL-001 work_events — resource B-Tree Partial（is_offline=FALSE）
-- 目的: 作業員別イベント検索（FR-AU-003 監査ログ）
-- Partial 条件: オンライン記録のみ（is_offline=FALSE）が監査ログ検索対象
CREATE INDEX CONCURRENTLY idx_work_events_resource
    ON work_events USING BTREE (resource)
    WHERE is_offline = FALSE;

COMMENT ON INDEX idx_work_events_resource IS
    'IDX-003 — resource（user_id）Partial B-Tree。is_offline=FALSE のオンライン記録のみ対象とし、インデックスサイズを低減。FR-AU-003（監査ログ検索）で使用。';
```

### IDX-004: work_events.(case_id, step_id) 複合 B-Tree

```sql
-- IDX-004: TBL-001 work_events — (case_id, step_id) 複合 B-Tree
-- 目的: 特定セッション内の特定ステップのイベント検索（ロックステップ確認・重複チェック）
CREATE INDEX CONCURRENTLY idx_work_events_case_id_step_id
    ON work_events USING BTREE (case_id, step_id)
    WHERE step_id IS NOT NULL;

COMMENT ON INDEX idx_work_events_case_id_step_id IS
    'IDX-004 — (case_id, step_id) 複合 Partial B-Tree。step_id IS NOT NULL の行のみ対象（work_started 等 step 不要のイベントを除外）。同一セッション内の同一ステップ重複送信チェックに使用。';
```

### IDX-005: outbox_events.status, created_at B-Tree（Partial: PENDING）

```sql
-- IDX-005: TBL-003 outbox_events — (status, created_at) B-Tree Partial（PENDING/FAILED）
-- 目的: Outbox Consumer の PENDING キュー取得（NFR-PRF-010）
CREATE INDEX CONCURRENTLY idx_outbox_events_status_created_at
    ON outbox_events USING BTREE (status, created_at ASC)
    WHERE status IN ('PENDING', 'FAILED');

COMMENT ON INDEX idx_outbox_events_status_created_at IS
    'IDX-005 — outbox_events の Partial B-Tree。PENDING/FAILED ステータスの行のみを対象とし、Outbox Consumer（BAT-002）が未送信キューを古い順に効率的に取得する。SENT/DLQ 行を除外するためインデックスサイズが小さい。NFR-PRF-010 対応。';
```

### IDX-006: work_executions.primary_worker_id B-Tree

```sql
-- IDX-006: TBL-005 work_executions — primary_worker_id B-Tree
-- 目的: 作業員別作業セッション一覧（管理画面 SCR-MC-003）
CREATE INDEX CONCURRENTLY idx_work_executions_primary_worker_id
    ON work_executions USING BTREE (primary_worker_id);

COMMENT ON INDEX idx_work_executions_primary_worker_id IS
    'IDX-006 — primary_worker_id B-Tree。作業員別の作業履歴一覧（管理画面・スキル評価）で使用。';
```

### IDX-007: work_executions.status B-Tree（Partial: 未完了）

```sql
-- IDX-007: TBL-005 work_executions — status B-Tree Partial（完了済み除外）
-- 目的: 進行中・中断中の作業セッション検索（FR-NV-013）
CREATE INDEX CONCURRENTLY idx_work_executions_status
    ON work_executions USING BTREE (status)
    WHERE status NOT IN ('COMPLETED', 'CANCELLED');

COMMENT ON INDEX idx_work_executions_status IS
    'IDX-007 — status Partial B-Tree。COMPLETED/CANCELLED を除外し、アクティブなセッションのみを対象とする。v_active_work_executions ビュー（VW-001）のベースインデックス。FR-NV-013 対応。';
```

### IDX-008: sops.(operation_id, is_active) 複合 B-Tree

```sql
-- IDX-008: TBL-007 sops — (operation_id, is_active) 複合 B-Tree
-- 目的: オペレーション別アクティブ SOP 一覧（マスタ管理画面・SOP 選択）
CREATE INDEX CONCURRENTLY idx_sops_operation_id_is_active
    ON sops USING BTREE (operation_id, is_active);

COMMENT ON INDEX idx_sops_operation_id_is_active IS
    'IDX-008 — (operation_id, is_active) 複合 B-Tree。オペレーション別の有効 SOP 取得。v_published_sops ビュー（VW-002）のベースインデックス。FR-MA-001〜015 対応。';
```

### IDX-009: steps.(sop_id, step_number) 複合 B-Tree

```sql
-- IDX-009: TBL-008 steps — (sop_id, step_number) 複合 B-Tree
-- 目的: SOP 内ステップの順序取得（StepEngine のステップシーケンス構築）
CREATE INDEX CONCURRENTLY idx_steps_sop_id_step_number
    ON steps USING BTREE (sop_id, step_number ASC);

COMMENT ON INDEX idx_steps_sop_id_step_number IS
    'IDX-009 — (sop_id, step_number) 複合 B-Tree 昇順。StepEngine が SOP 実行時にステップシーケンスを構築する際の主要インデックス。v_step_sequence ビュー（VW-004）のベースインデックス。';
```

### IDX-010: evidence_files.event_id B-Tree

```sql
-- IDX-010: TBL-009 evidence_files — event_id B-Tree
-- 目的: イベント別証拠ファイル取得（FR-EV-002）
CREATE INDEX CONCURRENTLY idx_evidence_files_event_id
    ON evidence_files USING BTREE (event_id);

COMMENT ON INDEX idx_evidence_files_event_id IS
    'IDX-010 — event_id B-Tree。特定 WorkEvent に紐付く証拠ファイルの取得。step_completed イベントの証拠確認（BR-BUS-003）で使用。FR-EV-002 対応。';
```

### IDX-011: users.login_id UNIQUE B-Tree（Partial: is_active=TRUE）

```sql
-- IDX-011: TBL-016 users — login_id UNIQUE B-Tree Partial（is_active=TRUE）
-- 目的: ログイン認証時の login_id 検索（FR-SY-001）
-- UNIQUE 制約: アクティブユーザー内での login_id 重複を禁止
CREATE UNIQUE INDEX CONCURRENTLY idx_users_login_id_active
    ON users USING BTREE (login_id)
    WHERE is_active = TRUE;

COMMENT ON INDEX idx_users_login_id_active IS
    'IDX-011 — login_id の Partial UNIQUE B-Tree（is_active=TRUE）。退職ユーザー（is_active=FALSE）の login_id は除外されるため、同名での新規登録が可能。FR-SY-001（認証）の主要インデックス。';
```

### IDX-012: users.is_active B-Tree（Partial: is_active=TRUE）

```sql
-- IDX-012: TBL-016 users — is_active B-Tree Partial（アクティブのみ）
-- 目的: アクティブユーザー全件取得（ユーザー選択 UI・スキルゲート検索）
CREATE INDEX CONCURRENTLY idx_users_is_active
    ON users USING BTREE (user_id)
    WHERE is_active = TRUE;

COMMENT ON INDEX idx_users_is_active IS
    'IDX-012 — is_active=TRUE の Partial B-Tree。退職ユーザーを除いたアクティブユーザー全件取得。v_user_skills_full ビュー（VW-003）のベースインデックス。';
```

### IDX-013: external_key_bindings.external_key GIN（JSONB 包含検索）

```sql
-- IDX-013: TBL-027 external_key_bindings — external_key GIN
-- 目的: 親機 ERP からの JSONB キー逆引き（IF-001 外部システム連携）
CREATE INDEX CONCURRENTLY idx_external_key_bindings_external_key_gin
    ON external_key_bindings USING GIN (external_key);

COMMENT ON INDEX idx_external_key_bindings_external_key_gin IS
    'IDX-013 — external_key JSONB の GIN インデックス。@> 演算子による部分一致検索（例: external_key @> ''{"lot_id": "L001"}''::jsonb）を高速化する。IF-001 外部システム連携の主要インデックス。';
```

### IDX-014: hash_chain_blocks.created_at B-Tree

```sql
-- IDX-014: TBL-031 hash_chain_blocks — created_at B-Tree
-- 目的: チェーン検証順序のブロック取得（BAT-001 週次検証）
CREATE INDEX CONCURRENTLY idx_hash_chain_blocks_created_at
    ON hash_chain_blocks USING BTREE (created_at DESC);

COMMENT ON INDEX idx_hash_chain_blocks_created_at IS
    'IDX-014 — created_at 降順 B-Tree。BAT-001 が最新ブロック（前回チェックポイント）を取得する際に使用。v_hash_chain_latest ビュー（VW-008）のベースインデックス。';
```

### IDX-015: auth_logs.(user_id, occurred_at) B-Tree

```sql
-- IDX-015: TBL-032 auth_logs — (user_id, occurred_at) B-Tree
-- 目的: 認証監査ログの作業員別時系列検索（FR-AU-004）
-- BRIN を使用: auth_logs は Append-only で自然挿入順（時系列）のため BRIN が効率的
CREATE INDEX CONCURRENTLY idx_auth_logs_user_id_occurred_at
    ON auth_logs USING BRIN (user_id, occurred_at);

COMMENT ON INDEX idx_auth_logs_user_id_occurred_at IS
    'IDX-015 — (user_id, occurred_at) BRIN インデックス。auth_logs は Append-only で挿入順が時系列のため BRIN が効率的（B-Tree より低コスト）。FR-AU-004（認証監査）のユーザー別ログ検索に使用。';
```

### IDX-016: idempotency_keys.idempotency_key UNIQUE B-Tree

```sql
-- IDX-016: TBL-035 idempotency_keys — idempotency_key UNIQUE B-Tree
-- 目的: API 冪等性チェックの主キー検索（アーキテクチャ原則 P3）
-- PRIMARY KEY で既に UNIQUE インデックスが存在するため、コメントのみ記録
-- CREATE UNIQUE INDEX は PRIMARY KEY 制約と重複するため不要（PostgreSQL が自動作成）

COMMENT ON INDEX idempotency_keys_pkey IS
    'IDX-016 — idempotency_key PRIMARY KEY インデックス（PostgreSQL が自動作成）。API リクエストの Idempotency-Key ヘッダ値で UNIQUE を保証。同一キーの重複 INSERT を排除し P3（Idempotent API）を実現する。';
```

---

## 3. インデックスサマリー

| IDX-ID | 物理名 | TBL | 種別 | 対象列 | Partial 条件 | 根拠 |
|---|---|---|---|---|---|---|
| IDX-001 | idx_work_events_case_id | TBL-001 | B-Tree | case_id | — | NFR-PRF-001 |
| IDX-002 | idx_work_events_timestamp_server | TBL-001 | B-Tree DESC | timestamp_server | — | FR-AU-003 |
| IDX-003 | idx_work_events_resource | TBL-001 | B-Tree Partial | resource | is_offline=FALSE | FR-AU-003 |
| IDX-004 | idx_work_events_case_id_step_id | TBL-001 | B-Tree Partial | (case_id, step_id) | step_id IS NOT NULL | ロックステップ確認 |
| IDX-005 | idx_outbox_events_status_created_at | TBL-003 | B-Tree Partial | (status, created_at) | status IN ('PENDING','FAILED') | NFR-PRF-010 |
| IDX-006 | idx_work_executions_primary_worker_id | TBL-005 | B-Tree | primary_worker_id | — | 管理画面検索 |
| IDX-007 | idx_work_executions_status | TBL-005 | B-Tree Partial | status | status NOT IN ('COMPLETED','CANCELLED') | FR-NV-013 |
| IDX-008 | idx_sops_operation_id_is_active | TBL-007 | B-Tree | (operation_id, is_active) | — | FR-MA-001〜015 |
| IDX-009 | idx_steps_sop_id_step_number | TBL-008 | B-Tree ASC | (sop_id, step_number) | — | StepEngine |
| IDX-010 | idx_evidence_files_event_id | TBL-009 | B-Tree | event_id | — | FR-EV-002 |
| IDX-011 | idx_users_login_id_active | TBL-016 | B-Tree UNIQUE Partial | login_id | is_active=TRUE | FR-SY-001 |
| IDX-012 | idx_users_is_active | TBL-016 | B-Tree Partial | user_id | is_active=TRUE | スキルゲート |
| IDX-013 | idx_external_key_bindings_external_key_gin | TBL-027 | GIN | external_key | — | IF-001 |
| IDX-014 | idx_hash_chain_blocks_created_at | TBL-031 | B-Tree DESC | created_at | — | BAT-001 |
| IDX-015 | idx_auth_logs_user_id_occurred_at | TBL-032 | BRIN | (user_id, occurred_at) | — | FR-AU-004 |
| IDX-016 | idempotency_keys_pkey（自動）| TBL-035 | B-Tree UNIQUE | idempotency_key | — | P3（Idempotent API）|

次採番値: **IDX-017**

---

**本節で確定した方針**
- **IDX-001〜016 全件の CREATE INDEX 全文を確定し、対象列・種別・Partial 条件・根拠 NFR/FR を全て明記した。次採番値 IDX-017 を台帳に記録する。**
- **work_events（TBL-001）の月次パーティションには CREATE INDEX を親テーブルに実行することで全パーティションへ自動継承され、CONCURRENTLY オプションで運用停止なしに作成する。**
- **Partial インデックス（IDX-003〜005・007・011〜012）により、完了済み・オフライン・退職済みの行をインデックスから除外し INSERT コストとインデックスサイズを最小化する。**

---

## 参照業界分析

### 必須
- [`90_業界分析/06_品質管理とトレーサビリティ.md`](../../../90_業界分析/06_品質管理とトレーサビリティ.md)
- [`90_業界分析/21_電子記録の法規制とALCOA+.md`](../../../90_業界分析/21_電子記録の法規制とALCOA+.md)

### 関連
- [`90_業界分析/27_オフライン同期とデータ整合性.md`](../../../90_業界分析/27_オフライン同期とデータ整合性.md)
