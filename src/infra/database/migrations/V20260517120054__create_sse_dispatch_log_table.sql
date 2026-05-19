-- V20260517120054__create_sse_dispatch_log_table.sql
-- TBL-053 sse_dispatch_log: SSE（Server-Sent Events）配信ログ。
-- Push 型作業指示（TBL-052 work_assignments）の端末配信状態を追跡する。
-- delivery_status / ack_at / retry_count のみ UPDATE 許可。dispatched_at は Append-only。

CREATE TABLE IF NOT EXISTS sse_dispatch_log (
    -- 配信ログ識別子。UUID v7（時系列順）。Rust 側で生成する。
    dispatch_id     UUID         NOT NULL DEFAULT gen_random_uuid(),
    -- 関連する作業指示の識別子。
    assignment_id   UUID         NOT NULL,
    -- 配信先端末の識別子。
    terminal_id     UUID         NOT NULL,
    -- SSE 配信日時。Append-only（変更不可）。
    dispatched_at   TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    -- 配信ステータス。queued → sent → ack の正常遷移、failed / expired は異常終了。
    delivery_status VARCHAR(16)  NOT NULL DEFAULT 'queued',
    -- 端末からの ACK 受信日時。NULL は未受信。
    ack_at          TIMESTAMPTZ  NULL,
    -- リトライ回数。0 以上の非負整数。
    retry_count     INTEGER      NOT NULL DEFAULT 0,

    CONSTRAINT pk_sse_dispatch_log PRIMARY KEY (dispatch_id),
    CONSTRAINT fk_sse_dispatch_log_assignment FOREIGN KEY (assignment_id)
        REFERENCES work_assignments (assignment_id) ON DELETE RESTRICT,
    CONSTRAINT fk_sse_dispatch_log_terminal FOREIGN KEY (terminal_id)
        REFERENCES devices (device_id) ON DELETE RESTRICT,
    CONSTRAINT ck_sse_dispatch_log_delivery_status CHECK (
        delivery_status IN ('queued', 'sent', 'ack', 'failed', 'expired')
    ),
    CONSTRAINT ck_sse_dispatch_log_retry_count_non_negative CHECK (retry_count >= 0)
);

COMMENT ON TABLE  sse_dispatch_log IS 'EN-053 SseDispatchLog — SSE 配信ログ。work_assignments の端末配信状態を追跡する。dispatched_at は Append-only。delivery_status / ack_at / retry_count のみ UPDATE 許可。';
COMMENT ON COLUMN sse_dispatch_log.dispatch_id     IS 'UUID v7（時系列順）。Rust 側で生成する。';
COMMENT ON COLUMN sse_dispatch_log.assignment_id   IS '関連する作業指示の識別子（work_assignments.assignment_id）。';
COMMENT ON COLUMN sse_dispatch_log.terminal_id     IS '配信先端末の識別子（devices.device_id）。';
COMMENT ON COLUMN sse_dispatch_log.dispatched_at   IS 'SSE 配信日時。Append-only（変更不可）。';
COMMENT ON COLUMN sse_dispatch_log.delivery_status IS '配信ステータス。queued / sent / ack / failed / expired の 5 値。';
COMMENT ON COLUMN sse_dispatch_log.ack_at          IS '端末からの ACK 受信日時。NULL は未受信。';
COMMENT ON COLUMN sse_dispatch_log.retry_count     IS 'リトライ回数。0 以上の非負整数。';
