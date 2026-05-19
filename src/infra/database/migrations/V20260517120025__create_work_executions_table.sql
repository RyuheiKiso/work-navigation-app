-- V20260517120025__create_work_executions_table.sql
-- TBL-005 work_executions: 作業セッション管理テーブル（更新可）。7年以上保存。
-- work_events.case_id の外部キー参照元。

-- EN-011 WorkInstance — 作業セッション管理テーブル（更新可）
CREATE TABLE IF NOT EXISTS work_executions (
    -- 作業実行識別子。UUID v7（時系列順）。Rust 側で生成する。
    work_execution_id    UUID        NOT NULL DEFAULT gen_random_uuid(),
    -- 実行する SOP の識別子。
    sop_id               UUID        NOT NULL,
    -- 作業開始時点の master_version_id を固定。SOP 改訂後も当時の版を参照可能にする。
    sop_version_id       UUID        NOT NULL,
    -- 主担当作業員の識別子。
    primary_worker_id    UUID        NOT NULL,
    -- 使用デバイスの識別子。
    device_id            UUID        NOT NULL,
    -- 関連するワークオーダーの識別子。NULL はワークオーダーなし（SOP 単独実行）。
    work_order_id        UUID        NULL,
    -- 作業実行ステータス。5 値のみ許可する。
    status               VARCHAR(16) NOT NULL DEFAULT 'NOT_STARTED',
    -- 作業開始日時。NULL は未着手。
    started_at           TIMESTAMPTZ NULL,
    -- 作業完了日時。NULL は未完了。
    completed_at         TIMESTAMPTZ NULL,
    -- プレースキーパー。中断・再開時に次に実行するステップ位置を保持する。
    current_step_index   SMALLINT    NOT NULL DEFAULT 0,
    -- レコード作成日時。
    created_at           TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    -- レコード最終更新日時。
    updated_at           TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT pk_work_executions PRIMARY KEY (work_execution_id),
    -- sops テーブルへの外部キー。SOP 削除時は RESTRICT。
    CONSTRAINT fk_work_executions_sop FOREIGN KEY (sop_id)
        REFERENCES sops (sop_id) ON DELETE RESTRICT,
    -- master_versions テーブルへの外部キー（SOP バージョン）。
    CONSTRAINT fk_work_executions_version FOREIGN KEY (sop_version_id)
        REFERENCES master_versions (master_version_id) ON DELETE RESTRICT,
    -- users テーブルへの外部キー（主担当作業員）。
    CONSTRAINT fk_work_executions_worker FOREIGN KEY (primary_worker_id)
        REFERENCES users (user_id) ON DELETE RESTRICT,
    -- devices テーブルへの外部キー（使用デバイス）。
    CONSTRAINT fk_work_executions_device FOREIGN KEY (device_id)
        REFERENCES devices (device_id) ON DELETE RESTRICT,
    -- work_orders テーブルへの外部キー（NULL 許容）。
    CONSTRAINT fk_work_executions_order FOREIGN KEY (work_order_id)
        REFERENCES work_orders (work_order_id) ON DELETE RESTRICT,
    -- status は 5 値のみ許可する。
    CONSTRAINT ck_work_executions_status CHECK (
        status IN ('NOT_STARTED', 'IN_PROGRESS', 'SUSPENDED', 'COMPLETED', 'CANCELLED')
    ),
    -- current_step_index は 0 以上の非負整数のみ許可する。
    CONSTRAINT ck_work_executions_step_index_non_negative CHECK (current_step_index >= 0),
    -- completed_at が設定されている場合は started_at も必須かつ completed >= started を保証する。
    CONSTRAINT ck_work_executions_completed_after_started CHECK (
        NOT (completed_at IS NOT NULL AND started_at IS NULL)
        AND NOT (completed_at IS NOT NULL AND completed_at < started_at)
    )
);

COMMENT ON TABLE  work_executions IS 'EN-011 WorkInstance — 作業セッション管理。1 件 = 1 回の SOP 実行。work_events.case_id の外部キー参照元。7年以上保存。';
COMMENT ON COLUMN work_executions.work_execution_id  IS 'UUID v7（時系列順）。Rust 側で生成する。';
COMMENT ON COLUMN work_executions.sop_id             IS '実行する SOP の sop_id。';
COMMENT ON COLUMN work_executions.sop_version_id     IS '作業開始時点の master_version_id を固定。SOP 改訂後も当時の版を参照可能にする。';
COMMENT ON COLUMN work_executions.primary_worker_id  IS '主担当作業員の user_id。';
COMMENT ON COLUMN work_executions.device_id          IS '使用デバイスの device_id。';
COMMENT ON COLUMN work_executions.work_order_id      IS '関連するワークオーダーの work_order_id。NULL はワークオーダーなし（SOP 単独実行）。';
COMMENT ON COLUMN work_executions.status             IS 'NOT_STARTED / IN_PROGRESS / SUSPENDED / COMPLETED / CANCELLED の 5 値。';
COMMENT ON COLUMN work_executions.started_at         IS '作業開始日時。NULL は未着手。';
COMMENT ON COLUMN work_executions.completed_at       IS '作業完了日時。NULL は未完了。';
COMMENT ON COLUMN work_executions.current_step_index IS 'プレースキーパー。中断・再開時に次に実行するステップ位置を保持する。';
COMMENT ON COLUMN work_executions.created_at         IS 'レコード作成日時。';
COMMENT ON COLUMN work_executions.updated_at         IS 'レコード最終更新日時。';
