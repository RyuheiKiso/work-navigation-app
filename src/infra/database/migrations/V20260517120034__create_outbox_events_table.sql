-- V20260517120034__create_outbox_events_table.sql
-- TBL-003 outbox_events: Transactional Outbox パターン送信キュー。INSERT + status UPDATE のみ許可。event_type 18種。

-- EN-021 OutboxEvent — Outbox パターンの送信キュー。INSERT は常に許可。status UPDATE のみ許可。
CREATE TABLE IF NOT EXISTS outbox_events (
    -- アウトボックス識別子。UUID v4。gen_random_uuid() で自動生成。
    outbox_id        UUID        NOT NULL DEFAULT gen_random_uuid(),
    -- 送信対象のイベント識別子。UNIQUE 制約で重複エントリを防止する。
    event_id         UUID        NOT NULL,
    -- イベント種別。18 種の列挙値のみ許可（CHECK 制約）。
    event_type       VARCHAR(64) NOT NULL,
    -- 送信ステータス。PENDING / SENDING / SENT / FAILED / DLQ の 5 種のみ許可（初期値: PENDING）。
    status           VARCHAR(16) NOT NULL DEFAULT 'PENDING',
    -- 送信するペイロード JSONB。
    payload          JSONB       NOT NULL,
    -- バックエンド→外部送信時の Idempotency-Key ヘッダ値。重複送信防止。UNIQUE 制約あり。
    idempotency_key  UUID        NOT NULL,
    -- レコード作成時刻。
    created_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    -- 送信完了時刻。NULL = 未送信。
    sent_at          TIMESTAMPTZ NULL,
    -- 次回リトライ予定時刻。NULL = リトライ不要。
    next_retry_at    TIMESTAMPTZ NULL,
    -- 再送試行回数。0 以上の値のみ許可（CHECK 制約）。上限は CFG-002（デフォルト 5）。
    retry_count      SMALLINT    NOT NULL DEFAULT 0,
    -- DLQ 移行理由テキスト。status=DLQ 時は必須（CHECK 制約）。
    dlq_reason       TEXT        NULL,

    -- 主キー
    CONSTRAINT pk_outbox_events PRIMARY KEY (outbox_id),
    -- event_id は UNIQUE でなければならない（同一イベントの二重送信防止）。
    CONSTRAINT uq_outbox_events_event_id UNIQUE (event_id),
    -- idempotency_key は UNIQUE でなければならない。
    CONSTRAINT uq_outbox_events_idempotency UNIQUE (idempotency_key),
    -- status は 5 種の列挙値のみ許可する。
    CONSTRAINT ck_outbox_events_status CHECK (
        status IN ('PENDING', 'SENDING', 'SENT', 'FAILED', 'DLQ')
    ),
    -- event_type は 18 種の列挙値のみ許可する（IQC/リワーク 9 種を含む）。
    CONSTRAINT ck_outbox_events_event_type CHECK (
        event_type IN (
            'work_event',
            'electronic_sign',
            'evidence_file',
            'measurement',
            'suspension',
            'andon_alert',
            'nonconformity',
            'capa',
            'kaizen_proposal',
            'incoming_inspection',
            'incoming_inspection_measurement',
            'concession_approval',
            'rework',
            'disposition',
            'rework_verification',
            'reworked_lot_label',
            'scrap_record',
            'return_to_vendor'
        )
    ),
    -- retry_count は 0 以上の値のみ許可する。
    CONSTRAINT ck_outbox_events_retry_non_negative CHECK (retry_count >= 0),
    -- DLQ 状態では dlq_reason が必須である。
    CONSTRAINT ck_outbox_events_dlq_requires_reason CHECK (
        NOT (status = 'DLQ' AND dlq_reason IS NULL)
    )
);

COMMENT ON TABLE  outbox_events IS 'EN-021 OutboxEvent — Transactional Outbox パターン送信キュー。INSERT は常に許可。status UPDATE のみ app_read_write に許可（PENDING→SENDING→SENT 等）。90日後アーカイブ。';
COMMENT ON COLUMN outbox_events.idempotency_key IS 'バックエンド→外部送信時の Idempotency-Key ヘッダ値。重複送信防止。';
COMMENT ON COLUMN outbox_events.retry_count     IS '再送試行回数。上限は CFG-002（デフォルト 5）。上限到達時に DLQ ステータスへ遷移。';
COMMENT ON COLUMN outbox_events.dlq_reason      IS 'DLQ 移行理由テキスト。status=DLQ 時は必須（CHECK 制約）。';

-- app_read_write に status 列の UPDATE のみを許可する
GRANT UPDATE (status, sent_at, next_retry_at, retry_count, dlq_reason) ON outbox_events TO app_read_write;
