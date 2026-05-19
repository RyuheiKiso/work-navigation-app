-- V20260519120011__create_missing_tables.sql
-- FIX 8: ハンドラが参照するが既存マイグレーションに存在しないテーブルを追加作成する
-- 各テーブルの列定義はハンドラ実装（SQL クエリ）から導出した
-- =====================================================

-- =====================================================
-- batch_execution_logs（バッチ実行ログ）
-- BAT-001 (hash_chain_verify) / BAT-011 (rework_cost) が INSERT する
-- 列: id UUID, bat_id TEXT, status TEXT, executed_at TIMESTAMPTZ
-- =====================================================
CREATE TABLE IF NOT EXISTS batch_execution_logs (
    id          UUID        NOT NULL DEFAULT gen_random_uuid(),
    bat_id      TEXT        NOT NULL,
    status      TEXT        NOT NULL DEFAULT 'completed',
    executed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    error_message TEXT      NULL,

    CONSTRAINT pk_batch_execution_logs PRIMARY KEY (id)
);

COMMENT ON TABLE batch_execution_logs IS 'バッチ実行ログ。BAT-001 / BAT-011 等の実行結果を記録する。';

-- =====================================================
-- webhook_secrets（HMAC 署名秘密鍵管理）
-- work_assignments.rs の fetch_webhook_hmac_key が参照する
-- 列: purpose TEXT, secret_value TEXT, is_active BOOLEAN, created_at TIMESTAMPTZ
-- =====================================================
CREATE TABLE IF NOT EXISTS webhook_secrets (
    id            UUID        NOT NULL DEFAULT gen_random_uuid(),
    purpose       TEXT        NOT NULL,
    secret_value  TEXT        NOT NULL,
    is_active     BOOLEAN     NOT NULL DEFAULT TRUE,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT pk_webhook_secrets PRIMARY KEY (id)
);

COMMENT ON TABLE webhook_secrets IS 'Webhook HMAC 署名秘密鍵管理テーブル。work_assignment_push 等の目的別に秘密鍵を管理する。';

-- =====================================================
-- work_cases（作業ケース一覧）
-- trace.rs forward_trace が参照する（SELECT EXISTS FROM work_cases WHERE id = $1）
-- =====================================================
CREATE TABLE IF NOT EXISTS work_cases (
    id            UUID        NOT NULL DEFAULT gen_random_uuid(),
    work_order_id UUID        NULL,
    case_status   TEXT        NOT NULL DEFAULT 'OPEN',
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT pk_work_cases PRIMARY KEY (id)
);

COMMENT ON TABLE work_cases IS '作業ケース一覧。trace.rs の順方向トレースが case_id 存在確認に使用する。';

-- =====================================================
-- lot_case_mappings（ロット ↔ ケース紐付け）
-- trace.rs backward_trace が参照する
-- 列: lot_id TEXT, case_id UUID
-- =====================================================
CREATE TABLE IF NOT EXISTS lot_case_mappings (
    lot_id   TEXT    NOT NULL,
    case_id  UUID    NOT NULL,

    CONSTRAINT pk_lot_case_mappings PRIMARY KEY (lot_id, case_id)
);

COMMENT ON TABLE lot_case_mappings IS 'ロット ID とケース ID の紐付けテーブル。逆方向トレース（lot → case）に使用する。';

-- =====================================================
-- lot_records（ロット記録）
-- trace.rs backward_trace が参照する
-- 列: lot_id TEXT, lot_type TEXT, process_id UUID, processed_from TIMESTAMPTZ, processed_to TIMESTAMPTZ
-- =====================================================
CREATE TABLE IF NOT EXISTS lot_records (
    lot_id          TEXT        NOT NULL,
    lot_type        TEXT        NOT NULL DEFAULT 'MATERIAL',
    process_id      UUID        NULL,
    processed_from  TIMESTAMPTZ NULL,
    processed_to    TIMESTAMPTZ NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT pk_lot_records PRIMARY KEY (lot_id)
);

COMMENT ON TABLE lot_records IS 'ロット記録テーブル。逆方向トレースの起点情報（ロット種別・工程・処理期間）を保持する。';

-- =====================================================
-- lot_lineage（ロット系譜）
-- trace.rs backward_trace が upstream/downstream を取得する
-- 列: upstream_lot_id TEXT, downstream_lot_id TEXT
-- =====================================================
CREATE TABLE IF NOT EXISTS lot_lineage (
    upstream_lot_id   TEXT NOT NULL,
    downstream_lot_id TEXT NOT NULL,

    CONSTRAINT pk_lot_lineage PRIMARY KEY (upstream_lot_id, downstream_lot_id)
);

COMMENT ON TABLE lot_lineage IS 'ロット系譜テーブル。上流/下流ロットの親子関係を保持する（リワーク由来ロット追跡に使用）。';

-- =====================================================
-- local_sync_state（端末マスタ同期状態）
-- master_sync.rs が SELECT / INSERT ... ON CONFLICT で使用する
-- ON CONFLICT (id) DO UPDATE のため id 列が必要
-- =====================================================
CREATE TABLE IF NOT EXISTS local_sync_state (
    id           UUID    NOT NULL DEFAULT gen_random_uuid(),
    sync_version BIGINT  NOT NULL DEFAULT 0,
    synced_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT pk_local_sync_state PRIMARY KEY (id)
);

-- 初期レコードを 1 件挿入する（ON CONFLICT DO UPDATE で更新するシングルトン設計）
INSERT INTO local_sync_state (sync_version, synced_at)
VALUES (0, NOW())
ON CONFLICT DO NOTHING;

COMMENT ON TABLE local_sync_state IS '端末マスタ同期状態テーブル。BAT-003 Master Sync Puller が sync_version を管理するシングルトンテーブル。';

-- =====================================================
-- sync_log（マスタ同期ログ）
-- master_sync.rs が INSERT に使用する
-- 列: sync_version BIGINT, synced_count INT, status TEXT, synced_at TIMESTAMPTZ
-- =====================================================
CREATE TABLE IF NOT EXISTS sync_log (
    id            UUID    NOT NULL DEFAULT gen_random_uuid(),
    sync_version  BIGINT  NOT NULL DEFAULT 0,
    synced_count  BIGINT  NOT NULL DEFAULT 0,
    status        TEXT    NOT NULL DEFAULT 'success',
    synced_at     TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT pk_sync_log PRIMARY KEY (id)
);

COMMENT ON TABLE sync_log IS 'マスタ同期ログ。BAT-003 が同期完了ごとに記録する。';

-- =====================================================
-- kaizen_reports（リワーク・カイゼン集計）
-- rework_cost.rs が ON CONFLICT (report_date, reason_code) で UPSERT する
-- 列: id UUID, report_date DATE, rework_count INT, total_hours FLOAT, reason_code TEXT, created_at TIMESTAMPTZ
-- =====================================================
CREATE TABLE IF NOT EXISTS kaizen_reports (
    id            UUID    NOT NULL DEFAULT gen_random_uuid(),
    report_date   DATE    NOT NULL,
    rework_count  INTEGER NOT NULL DEFAULT 0,
    total_hours   FLOAT   NOT NULL DEFAULT 0,
    reason_code   TEXT    NOT NULL DEFAULT '',
    cost_amount   NUMERIC NULL,
    cost_currency TEXT    NULL,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT pk_kaizen_reports PRIMARY KEY (id),
    CONSTRAINT uq_kaizen_reports_date_reason UNIQUE (report_date, reason_code)
);

COMMENT ON TABLE kaizen_reports IS 'リワーク・カイゼン集計レポートテーブル。BAT-011 が日次集計して挿入する。';

-- =====================================================
-- report_jobs（帳票生成ジョブ）
-- reports.rs が INSERT に使用する
-- 列: id UUID, report_type TEXT, status TEXT, requested_by UUID, from_date TIMESTAMPTZ, to_date TIMESTAMPTZ,
--       filters JSONB, format TEXT, created_at TIMESTAMPTZ
-- =====================================================
CREATE TABLE IF NOT EXISTS report_jobs (
    id            UUID    NOT NULL DEFAULT gen_random_uuid(),
    report_type   TEXT    NOT NULL,
    status        TEXT    NOT NULL DEFAULT 'queued',
    requested_by  UUID    NULL,
    from_date     TIMESTAMPTZ NULL,
    to_date       TIMESTAMPTZ NULL,
    filters       JSONB   NULL DEFAULT '{}',
    format        TEXT    NOT NULL DEFAULT 'pdf',
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at  TIMESTAMPTZ NULL,

    CONSTRAINT pk_report_jobs PRIMARY KEY (id)
);

COMMENT ON TABLE report_jobs IS '帳票生成ジョブキュー。reports.rs が非同期ジョブ登録に使用する。';

-- =====================================================
-- report_files（帳票ファイル）
-- 帳票生成完了時に生成ファイル情報を記録する（reports.rs 拡張用）
-- =====================================================
CREATE TABLE IF NOT EXISTS report_files (
    id          UUID    NOT NULL DEFAULT gen_random_uuid(),
    job_id      UUID    NOT NULL,
    file_path   TEXT    NOT NULL,
    file_size   BIGINT  NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT pk_report_files PRIMARY KEY (id),
    CONSTRAINT fk_report_files_job FOREIGN KEY (job_id) REFERENCES report_jobs (id) ON DELETE CASCADE
);

COMMENT ON TABLE report_files IS '帳票ファイル管理テーブル。生成された帳票ファイルのパスとサイズを記録する。';

-- =====================================================
-- outbox_dead_letters（Outbox Dead Letter Queue）
-- ops.rs の list_dlq / requeue が参照する
-- 列: id UUID, event_id UUID, event_type TEXT, last_error TEXT, retry_count INT,
--       dead_lettered_at TIMESTAMPTZ, deleted_at TIMESTAMPTZ, requeue_reason TEXT
-- requeue: INSERT INTO outbox_events から event_type, event_id を参照する
-- =====================================================
CREATE TABLE IF NOT EXISTS outbox_dead_letters (
    id               UUID    NOT NULL DEFAULT gen_random_uuid(),
    event_id         UUID    NOT NULL,
    event_type       TEXT    NOT NULL,
    last_error       TEXT    NOT NULL DEFAULT '',
    retry_count      INTEGER NOT NULL DEFAULT 0,
    dead_lettered_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at       TIMESTAMPTZ NULL,
    requeue_reason   TEXT    NULL,
    payload          JSONB   NULL,

    CONSTRAINT pk_outbox_dead_letters PRIMARY KEY (id)
);

COMMENT ON TABLE outbox_dead_letters IS 'Outbox Dead Letter Queue。最大リトライ回数を超過した outbox_events をここに移動する。ops.rs が照会・再キュー操作を提供する。';
