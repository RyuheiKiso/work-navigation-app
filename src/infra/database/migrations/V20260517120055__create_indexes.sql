-- V20260517120055__create_indexes.sql
-- IDX-001〜035 全インデックス + マテリアライズドビュー用 2 件を作成する。
-- Note: 本番環境での追加は CONCURRENTLY で行うこと
--       マイグレーション内では CONCURRENTLY はトランザクションと共存できないため使用しない。
--
-- 対象ドキュメント: docs/05_詳細設計/01_データベース詳細設計/03_インデックス詳細設計（IDXカタログ）.md

-- =============================================================================
-- IDX-001: TBL-001 work_events — case_id B-Tree（最優先）
-- 目的: 同一作業セッション（XES Case）のイベント一覧検索（NFR-PRF-001: P95 200ms）
-- パーティション適用: 全月次パーティションに自動継承
-- =============================================================================
CREATE INDEX idx_work_events_case_id
    ON work_events USING BTREE (case_id);

COMMENT ON INDEX idx_work_events_case_id IS
    'IDX-001 — work_events を case_id（XES Case ID）で検索するためのインデックス。StepEngine がステップ完了処理で当該セッションの最終イベントを取得する際に使用。NFR-PRF-001 達成の主要手段。';

-- =============================================================================
-- IDX-002: TBL-001 work_events — timestamp_server B-Tree（時系列範囲検索）
-- 目的: 時系列範囲クエリ（監査ログ・バッチ処理の期間指定）
-- パーティション適用: パーティション境界と一致するため高効率
-- =============================================================================
CREATE INDEX idx_work_events_timestamp_server
    ON work_events USING BTREE (timestamp_server DESC);

COMMENT ON INDEX idx_work_events_timestamp_server IS
    'IDX-002 — timestamp_server の降順 B-Tree。最新イベント取得（hash_chain 検証の起点取得）と時系列範囲クエリに使用。パーティションプルーニングと組み合わせて高効率。';

-- =============================================================================
-- IDX-003: TBL-001 work_events — resource B-Tree Partial（is_offline=FALSE）
-- 目的: 作業員別イベント検索（FR-AU-003 監査ログ）
-- Partial 条件: オンライン記録のみ（is_offline=FALSE）が監査ログ検索対象
-- =============================================================================
CREATE INDEX idx_work_events_resource
    ON work_events USING BTREE (resource)
    WHERE is_offline = FALSE;

COMMENT ON INDEX idx_work_events_resource IS
    'IDX-003 — resource（user_id）Partial B-Tree。is_offline=FALSE のオンライン記録のみ対象とし、インデックスサイズを低減。FR-AU-003（監査ログ検索）で使用。';

-- =============================================================================
-- IDX-004: TBL-001 work_events — (case_id, step_id) 複合 B-Tree Partial
-- 目的: 特定セッション内の特定ステップのイベント検索（ロックステップ確認・重複チェック）
-- =============================================================================
CREATE INDEX idx_work_events_case_id_step_id
    ON work_events USING BTREE (case_id, step_id)
    WHERE step_id IS NOT NULL;

COMMENT ON INDEX idx_work_events_case_id_step_id IS
    'IDX-004 — (case_id, step_id) 複合 Partial B-Tree。step_id IS NOT NULL の行のみ対象（work_started 等 step 不要のイベントを除外）。同一セッション内の同一ステップ重複送信チェックに使用。';

-- =============================================================================
-- IDX-005: TBL-003 outbox_events — (status, created_at) B-Tree Partial（PENDING/FAILED）
-- 目的: Outbox Consumer の PENDING キュー取得（NFR-PRF-010）
-- =============================================================================
CREATE INDEX idx_outbox_events_status_created_at
    ON outbox_events USING BTREE (status, created_at ASC)
    WHERE status IN ('PENDING', 'FAILED');

COMMENT ON INDEX idx_outbox_events_status_created_at IS
    'IDX-005 — outbox_events の Partial B-Tree。PENDING/FAILED ステータスの行のみを対象とし、Outbox Consumer（BAT-002）が未送信キューを古い順に効率的に取得する。SENT/DLQ 行を除外するためインデックスサイズが小さい。NFR-PRF-010 対応。';

-- =============================================================================
-- IDX-006: TBL-005 work_executions — primary_worker_id B-Tree
-- 目的: 作業員別作業セッション一覧（管理画面 SCR-MC-003）
-- =============================================================================
CREATE INDEX idx_work_executions_primary_worker_id
    ON work_executions USING BTREE (primary_worker_id);

COMMENT ON INDEX idx_work_executions_primary_worker_id IS
    'IDX-006 — primary_worker_id B-Tree。作業員別の作業履歴一覧（管理画面・スキル評価）で使用。';

-- =============================================================================
-- IDX-007: TBL-005 work_executions — status B-Tree Partial（完了済み除外）
-- 目的: 進行中・中断中の作業セッション検索（FR-NV-013）
-- =============================================================================
CREATE INDEX idx_work_executions_status
    ON work_executions USING BTREE (status)
    WHERE status NOT IN ('COMPLETED', 'CANCELLED');

COMMENT ON INDEX idx_work_executions_status IS
    'IDX-007 — status Partial B-Tree。COMPLETED/CANCELLED を除外し、アクティブなセッションのみを対象とする。v_active_work_executions ビュー（VW-001）のベースインデックス。FR-NV-013 対応。';

-- =============================================================================
-- IDX-008: TBL-007 sops — (operation_id, is_active) 複合 B-Tree
-- 目的: オペレーション別アクティブ SOP 一覧（マスタ管理画面・SOP 選択）
-- =============================================================================
CREATE INDEX idx_sops_operation_id_is_active
    ON sops USING BTREE (operation_id, is_active);

COMMENT ON INDEX idx_sops_operation_id_is_active IS
    'IDX-008 — (operation_id, is_active) 複合 B-Tree。オペレーション別の有効 SOP 取得。v_published_sops ビュー（VW-002）のベースインデックス。FR-MA-001〜015 対応。';

-- =============================================================================
-- IDX-009: TBL-008 steps — (sop_id, step_number) 複合 B-Tree
-- 目的: SOP 内ステップの順序取得（StepEngine のステップシーケンス構築）
-- =============================================================================
CREATE INDEX idx_steps_sop_id_step_number
    ON steps USING BTREE (sop_id, step_number ASC);

COMMENT ON INDEX idx_steps_sop_id_step_number IS
    'IDX-009 — (sop_id, step_number) 複合 B-Tree 昇順。StepEngine が SOP 実行時にステップシーケンスを構築する際の主要インデックス。v_step_sequence ビュー（VW-004）のベースインデックス。';

-- =============================================================================
-- IDX-010: TBL-009 evidence_files — event_id B-Tree
-- 目的: イベント別証拠ファイル取得（FR-EV-002）
-- =============================================================================
CREATE INDEX idx_evidence_files_event_id
    ON evidence_files USING BTREE (event_id);

COMMENT ON INDEX idx_evidence_files_event_id IS
    'IDX-010 — event_id B-Tree。特定 WorkEvent に紐付く証拠ファイルの取得。step_completed イベントの証拠確認（BR-BUS-003）で使用。FR-EV-002 対応。';

-- =============================================================================
-- IDX-011: TBL-016 users — login_id UNIQUE B-Tree Partial（is_active=TRUE）
-- 目的: ログイン認証時の login_id 検索（FR-SY-001）
-- UNIQUE 制約: アクティブユーザー内での login_id 重複を禁止
-- =============================================================================
CREATE UNIQUE INDEX idx_users_login_id_active
    ON users USING BTREE (login_id)
    WHERE is_active = TRUE;

COMMENT ON INDEX idx_users_login_id_active IS
    'IDX-011 — login_id の Partial UNIQUE B-Tree（is_active=TRUE）。退職ユーザー（is_active=FALSE）の login_id は除外されるため、同名での新規登録が可能。FR-SY-001（認証）の主要インデックス。';

-- =============================================================================
-- IDX-012: TBL-016 users — user_id B-Tree Partial（is_active=TRUE）
-- 目的: アクティブユーザー全件取得（ユーザー選択 UI・スキルゲート検索）
-- =============================================================================
CREATE INDEX idx_users_is_active
    ON users USING BTREE (user_id)
    WHERE is_active = TRUE;

COMMENT ON INDEX idx_users_is_active IS
    'IDX-012 — is_active=TRUE の Partial B-Tree。退職ユーザーを除いたアクティブユーザー全件取得。v_user_skills_full ビュー（VW-003）のベースインデックス。';

-- =============================================================================
-- IDX-013: TBL-027 external_key_bindings — external_key GIN（JSONB 包含検索）
-- 目的: 親機 ERP からの JSONB キー逆引き（IF-001 外部システム連携）
-- =============================================================================
CREATE INDEX idx_external_key_bindings_external_key_gin
    ON external_key_bindings USING GIN (external_key);

COMMENT ON INDEX idx_external_key_bindings_external_key_gin IS
    'IDX-013 — external_key JSONB の GIN インデックス。@> 演算子による部分一致検索（例: external_key @> ''{"lot_id": "L001"}''::jsonb）を高速化する。IF-001 外部システム連携の主要インデックス。';

-- =============================================================================
-- IDX-014: TBL-031 hash_chain_blocks — created_at B-Tree
-- 目的: チェーン検証順序のブロック取得（BAT-001 週次検証）
-- =============================================================================
CREATE INDEX idx_hash_chain_blocks_created_at
    ON hash_chain_blocks USING BTREE (created_at DESC);

COMMENT ON INDEX idx_hash_chain_blocks_created_at IS
    'IDX-014 — created_at 降順 B-Tree。BAT-001 が最新ブロック（前回チェックポイント）を取得する際に使用。v_hash_chain_latest ビュー（VW-008）のベースインデックス。';

-- =============================================================================
-- IDX-015: TBL-032 auth_logs — (user_id, occurred_at DESC) 複合 B-Tree
-- 目的: 認証監査ログの作業員別時系列検索（FR-AU-004）
-- B-Tree を採用: user_id（ランダム UUID v7）は物理ページと相関しないため BRIN では
--               user_id 絞り込みに実効性がない。年 50 万行規模では複合 B-Tree の
--               挿入コスト増は許容範囲。
-- 注記: 旧設計では BRIN を指定していたが IDX カタログ・概要設計 06 章・採番台帳 3 箇所で
--       表記が分裂していた。ADR-011 対応で B-Tree 複合に統一した（指摘6対応）。
-- =============================================================================
CREATE INDEX idx_auth_logs_user_id_occurred_at
    ON auth_logs USING BTREE (user_id, occurred_at DESC);

COMMENT ON INDEX idx_auth_logs_user_id_occurred_at IS
    'IDX-015 — (user_id, occurred_at DESC) 複合 B-Tree。FR-AU-004（認証監査）のユーザー別時系列検索（新しい順）に最適。user_id がランダム UUID のため BRIN より B-Tree が有効。3 箇所（IDX カタログ / 概要設計 06 章 / 採番台帳）の表記揺れを統一（指摘6対応）。';

-- =============================================================================
-- IDX-016: TBL-035 idempotency_keys — idempotency_key UNIQUE B-Tree
-- 目的: API 冪等性チェックの主キー検索（アーキテクチャ原則 P3）
-- PRIMARY KEY で既に UNIQUE インデックスが存在するため CREATE INDEX は不要。
-- PostgreSQL が自動作成した idempotency_keys_pkey にコメントを付与する。
-- =============================================================================
COMMENT ON INDEX idempotency_keys_pkey IS
    'IDX-016 — idempotency_key PRIMARY KEY インデックス（PostgreSQL が自動作成）。API リクエストの Idempotency-Key ヘッダ値で UNIQUE を保証。同一キーの重複 INSERT を排除し P3（Idempotent API）を実現する。';

-- =============================================================================
-- IDX-017: TBL-051 case_locks — terminal_id B-Tree
-- 目的: 端末別の保有 case 一覧取得（BAT-013 処理、デバッグ、監査）
-- =============================================================================
CREATE INDEX idx_case_locks_terminal_id
    ON case_locks USING BTREE (terminal_id);

COMMENT ON INDEX idx_case_locks_terminal_id IS
    'IDX-017 — case_locks を terminal_id で検索するインデックス。端末別保有 case の一覧取得に使用。';

-- =============================================================================
-- IDX-018: TBL-051 case_locks — heartbeat_at Partial（ACTIVE のみ）
-- 目的: BAT-013 の EXPIRED 化対象を効率的に取得
-- =============================================================================
CREATE INDEX idx_case_locks_heartbeat_at_active
    ON case_locks USING BTREE (heartbeat_at)
    WHERE lock_status = 'ACTIVE';

COMMENT ON INDEX idx_case_locks_heartbeat_at_active IS
    'IDX-018 — case_locks の heartbeat_at 昇順 Partial インデックス（ACTIVE のみ）。BAT-013 が heartbeat_at < NOW() - INTERVAL ''5 minutes'' で EXPIRED 化対象を絞り込むために使用。';

-- =============================================================================
-- IDX-019: TBL-009 evidence_files — created_at BRIN
-- 目的: サーバー受信時刻による時系列アクセス（06_インデックス §1「全 Append-only テーブルは created_at 降順インデックス必須」）
-- BRIN を採用: Append-only で自然挿入順が時系列のため BRIN が効率的
-- =============================================================================
CREATE INDEX idx_evidence_files_created_at
    ON evidence_files USING BRIN (created_at);

COMMENT ON INDEX idx_evidence_files_created_at IS
    'IDX-019 — created_at BRIN インデックス。Append-only テーブルの時系列挿入順と一致するため BRIN が B-Tree より低コスト。証拠ファイルの受信時刻範囲検索に使用。06_インデックス §1 準拠。';

-- =============================================================================
-- IDX-020: TBL-010 measurements — created_at BRIN
-- 目的: サーバー受信時刻による時系列アクセス（06_インデックス §1「全 Append-only テーブルは created_at 降順インデックス必須」）
-- =============================================================================
CREATE INDEX idx_measurements_created_at
    ON measurements USING BRIN (created_at);

COMMENT ON INDEX idx_measurements_created_at IS
    'IDX-020 — created_at BRIN インデックス。Append-only で時系列挿入順が保証されるため BRIN を採用。計測値の受信時刻範囲検索に使用。06_インデックス §1 準拠。';

-- =============================================================================
-- IDX-021: TBL-038 incoming_inspections — lot_id B-Tree
-- 目的: ロット別入荷検査履歴検索（FR-IQ-001）
-- =============================================================================
CREATE INDEX idx_incoming_insp_lot
    ON incoming_inspections USING BTREE (lot_id);

COMMENT ON INDEX idx_incoming_insp_lot IS
    'IDX-021 — lot_id B-Tree。入荷ロット単位の検査履歴取得。lot_qc_states との JOIN で使用。FR-IQ-001 対応。';

-- =============================================================================
-- IDX-022: TBL-038 incoming_inspections — (supplier_id, qc_status) 複合 B-Tree
-- 目的: 仕入先別・ステータス別検査一覧（品質管理ダッシュボード FR-IQ-003）
-- =============================================================================
CREATE INDEX idx_incoming_insp_supplier_status
    ON incoming_inspections USING BTREE (supplier_id, qc_status);

COMMENT ON INDEX idx_incoming_insp_supplier_status IS
    'IDX-022 — (supplier_id, qc_status) 複合 B-Tree。仕入先別 QC ステータス集計・ダッシュボード表示に使用。FR-IQ-003 対応。';

-- =============================================================================
-- IDX-023: TBL-040 incoming_inspection_measurements — inspection_id B-Tree
-- 目的: 検査 ID 別サンプル測定値一覧取得（FR-IQ-002）
-- =============================================================================
CREATE INDEX idx_insp_meas_inspection
    ON incoming_inspection_measurements USING BTREE (inspection_id);

COMMENT ON INDEX idx_insp_meas_inspection IS
    'IDX-023 — inspection_id B-Tree。入荷検査ヘッダに対するサンプル測定値明細の取得に使用。FR-IQ-002 対応。';

-- =============================================================================
-- IDX-024: TBL-043 reworks — parent_nonconformity_id B-Tree
-- 目的: 不適合 ID 別リワーク一覧取得（FR-RW-001）
-- =============================================================================
CREATE INDEX idx_reworks_nonconformity
    ON reworks USING BTREE (parent_nonconformity_id);

COMMENT ON INDEX idx_reworks_nonconformity IS
    'IDX-024 — parent_nonconformity_id B-Tree。不適合レコードからリワーク作業への参照取得に使用。FR-RW-001 対応。';

-- =============================================================================
-- IDX-025: TBL-043 reworks — status B-Tree Partial（未完了のみ）
-- 目的: 進行中リワーク一覧取得（リワーク管理ダッシュボード）
-- =============================================================================
CREATE INDEX idx_reworks_status
    ON reworks USING BTREE (status)
    WHERE status NOT IN ('CLOSED_OK_RELEASE', 'CLOSED_DOWNGRADE', 'CLOSED_SCRAP', 'CLOSED_RETURN');

COMMENT ON INDEX idx_reworks_status IS
    'IDX-025 — status Partial B-Tree。未完了リワーク（完了・クローズ以外）のみを対象とし、進行中リワーク一覧取得に使用。Partial 条件で完了済みを除外しサイズを最小化。';

-- =============================================================================
-- IDX-026: TBL-039 sampling_plans — (material_id, supplier_id) 複合 B-Tree Partial（有効のみ）
-- 目的: 材料 × 仕入先のサンプリング計画検索（FR-IQ-001 AQL 計画引き当て）
-- =============================================================================
CREATE INDEX idx_sampling_plans_material_supplier
    ON sampling_plans USING BTREE (material_id, supplier_id)
    WHERE is_active = TRUE;

COMMENT ON INDEX idx_sampling_plans_material_supplier IS
    'IDX-026 — (material_id, supplier_id) 複合 Partial B-Tree。is_active=TRUE の有効計画のみを対象。入荷検査時の AQL サンプリング計画引き当てに使用。FR-IQ-001 対応。';

-- =============================================================================
-- IDX-027: TBL-024 lots — supplier_id B-Tree Partial
-- 目的: 仕入先別ロット一覧取得（入荷管理・トレーサビリティ）
-- =============================================================================
CREATE INDEX idx_lots_supplier_id
    ON lots USING BTREE (supplier_id)
    WHERE supplier_id IS NOT NULL;

COMMENT ON INDEX idx_lots_supplier_id IS
    'IDX-027 — supplier_id Partial B-Tree（NULL 除外）。仕入先別ロット一覧・入荷検査履歴への JOIN に使用。';

-- =============================================================================
-- IDX-028: TBL-024 lots — material_id B-Tree Partial
-- 目的: 材料別ロット一覧取得（材料トレーサビリティ）
-- =============================================================================
CREATE INDEX idx_lots_material_id
    ON lots USING BTREE (material_id)
    WHERE material_id IS NOT NULL;

COMMENT ON INDEX idx_lots_material_id IS
    'IDX-028 — material_id Partial B-Tree（NULL 除外）。材料別ロット追跡・材料影響範囲分析に使用。';

-- =============================================================================
-- IDX-029: TBL-024 lots — parent_lot_id B-Tree Partial
-- 目的: 親ロット → 派生ロット（リワーク後）の追跡（FR-RW-008）
-- =============================================================================
CREATE INDEX idx_lots_parent_lot_id
    ON lots USING BTREE (parent_lot_id)
    WHERE parent_lot_id IS NOT NULL;

COMMENT ON INDEX idx_lots_parent_lot_id IS
    'IDX-029 — parent_lot_id Partial B-Tree（NULL 除外）。リワーク後の派生ロット追跡（親子 lot 追従）に使用。FR-RW-008 対応。';

-- =============================================================================
-- IDX-030: TBL-024 lots — qc_status B-Tree Partial（未完了のみ）
-- 目的: QC 未完了ロットの後工程ゲート判定（ERR-BIZ-015）
-- =============================================================================
CREATE INDEX idx_lots_qc_status
    ON lots USING BTREE (qc_status)
    WHERE qc_status NOT IN ('PASSED', 'SCRAPPED', 'RETURNED');

COMMENT ON INDEX idx_lots_qc_status IS
    'IDX-030 — qc_status Partial B-Tree。PASSED/SCRAPPED/RETURNED 以外のロットに絞り込み。後工程スキャン時の ERR-BIZ-015 ゲート判定（lot_qc_states と合わせて二重確認）に使用。';

-- =============================================================================
-- IDX-031: TBL-040 incoming_inspection_measurements — (qc_case_id, content_hash) 複合 B-Tree
-- 目的: IQC チェーン検証時の qc_case_id 単位全件取得（BAT-001 拡張）
-- =============================================================================
CREATE INDEX idx_inspection_qc_case_chain
    ON incoming_inspection_measurements USING BTREE (qc_case_id, content_hash);

COMMENT ON INDEX idx_inspection_qc_case_chain IS
    'IDX-031 — (qc_case_id, content_hash) 複合 B-Tree。BAT-001 の IQC チェーン検証時に qc_case_id 単位で測定値レコードを時系列順に取得し content_hash の連続性を検証する（ADR-011）。';

-- =============================================================================
-- IDX-032: TBL-052 work_assignments — (target_terminal_id, status) 複合 B-Tree Partial
-- 目的: 端末別の受付済み・処理中割当取得（作業割当ディスパッチャー）
-- =============================================================================
CREATE INDEX idx_wa_terminal_status
    ON work_assignments USING BTREE (target_terminal_id, status)
    WHERE status IN ('pending', 'pending_resolution', 'dispatched');

COMMENT ON INDEX idx_wa_terminal_status IS
    'IDX-032 — (target_terminal_id, status) 複合 Partial B-Tree。端末別の未完了割当（pending/pending_resolution/dispatched）取得に使用。SSE ディスパッチャーが端末への割当配信を効率化する。';

-- =============================================================================
-- IDX-033: TBL-052 work_assignments — received_at BRIN
-- 目的: 受付時刻による時系列範囲検索（割当アーカイブ・監査）
-- =============================================================================
CREATE INDEX idx_wa_received
    ON work_assignments USING BRIN (received_at);

COMMENT ON INDEX idx_wa_received IS
    'IDX-033 — received_at BRIN インデックス。Append-only 的な挿入順（受付時刻昇順）が保証されるため BRIN が効率的。割当の時系列範囲検索・アーカイブ処理に使用。';

-- =============================================================================
-- IDX-034: TBL-052 work_assignments — (external_system, idempotency_key) UNIQUE
-- 目的: 外部システムからの割当重複送信防止（冪等性保証）
-- =============================================================================
CREATE UNIQUE INDEX idx_wa_idempotency
    ON work_assignments (external_system, idempotency_key);

COMMENT ON INDEX idx_wa_idempotency IS
    'IDX-034 — (external_system, idempotency_key) UNIQUE インデックス。外部システム（ERP/MES）からの割当重複送信を防止する。同一外部システム内での idempotency_key 一意性を保証。';

-- =============================================================================
-- IDX-035: TBL-053 sse_dispatch_log — (assignment_id, terminal_id) 複合 B-Tree
-- 目的: 割当 ID + 端末 ID による SSE 配信ログ検索（配信状態確認・再送処理）
-- =============================================================================
CREATE INDEX idx_sdl_assignment_terminal
    ON sse_dispatch_log USING BTREE (assignment_id, terminal_id);

COMMENT ON INDEX idx_sdl_assignment_terminal IS
    'IDX-035 — (assignment_id, terminal_id) 複合 B-Tree。特定割当の特定端末への配信ログ取得に使用。SSE 配信失敗時の再送処理および配信状態確認（delivery_status）に使用。';

-- =============================================================================
-- MV 用インデックス: mv_daily_work_summary UNIQUE INDEX
-- 目的: REFRESH CONCURRENTLY を可能にするための UNIQUE インデックス（VW-006）
-- 注意: マテリアライズドビュー本体は V20260517120057 で作成するため、
--       このインデックスは V20260517120057 内で mv 作成直後に定義する。
--       ここではドキュメント上の IDX カタログに記載されている内容を参照用コメントとして残す。
-- =============================================================================
-- idx_mv_daily_work_summary_pk と idx_mv_daily_work_summary_date は
-- V20260517120057__create_materialized_views.sql で定義する。
