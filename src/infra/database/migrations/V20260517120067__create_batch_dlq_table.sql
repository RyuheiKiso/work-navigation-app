-- V20260517120067__create_batch_dlq_table.sql
-- batch_dlq: 全バッチジョブ共通の Dead Letter Queue テーブル。
-- 権威: docs/05_詳細設計/07_アルゴリズム詳細設計/06_バッチジョブ処理詳細（BAT-001〜010）.md §12-2
-- 処理失敗したジョブのペイロードを保持し、手動再処理を可能にする。
-- resolved_at が NULL = 未解決。非 NULL = 手動解決済み。

CREATE TABLE IF NOT EXISTS batch_dlq (
    -- DLQ エントリ識別子。UUID v4。gen_random_uuid() で自動生成。
    id              UUID         NOT NULL DEFAULT gen_random_uuid(),
    -- バッチ識別子。'BAT-001'〜'BAT-010' のいずれか。
    bat_id          TEXT         NOT NULL,
    -- バッチ冪等性キー。重複処理防止に使用する。
    idempotency_key TEXT         NOT NULL,
    -- 失敗したジョブのペイロード。再処理時に参照する。
    payload         JSONB        NOT NULL,
    -- エラーメッセージ。失敗原因を記録する。
    error_message   TEXT         NOT NULL,
    -- リトライ回数。0 = DLQ 送りにした時点で既にリトライ上限に達した状態。
    retry_count     INTEGER      NOT NULL DEFAULT 0,
    -- レコード作成日時。
    created_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    -- 手動解決日時。NULL = 未解決。管理者が手動でペイロードを再処理した後に設定する。
    resolved_at     TIMESTAMPTZ  NULL,

    CONSTRAINT pk_batch_dlq PRIMARY KEY (id),
    -- bat_id は 'BAT-' で始まる形式を強制する。
    CONSTRAINT ck_batch_dlq_bat_id CHECK (bat_id LIKE 'BAT-%'),
    -- retry_count は 0 以上の非負整数のみ許可する。
    CONSTRAINT ck_batch_dlq_retry_count_non_negative CHECK (retry_count >= 0)
);

COMMENT ON TABLE batch_dlq IS
    '全バッチジョブ共通 Dead Letter Queue（BAT-001〜010）。処理失敗ペイロードを保持し、管理者による手動解決を可能にする。resolved_at IS NULL = 未解決。';
COMMENT ON COLUMN batch_dlq.idempotency_key IS 'バッチ冪等性キー。重複処理防止に使用する。';
COMMENT ON COLUMN batch_dlq.payload         IS '失敗したジョブのペイロード。再処理時に参照する。';
COMMENT ON COLUMN batch_dlq.resolved_at     IS '手動解決日時。NULL = 未解決。管理者が手動処理後に設定する。';

-- 未解決 DLQ の検索インデックス（管理コンソールで未解決一覧を表示する）
CREATE INDEX idx_batch_dlq_unresolved
    ON batch_dlq USING BTREE (bat_id, created_at DESC)
    WHERE resolved_at IS NULL;

-- app_read_write に INSERT/SELECT/UPDATE を許可する（resolved_at の更新に必要）
GRANT INSERT, SELECT, UPDATE ON batch_dlq TO app_read_write;
