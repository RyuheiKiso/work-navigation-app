-- V20260517120036__create_hash_chain_blocks_table.sql
-- TBL-031 hash_chain_blocks: SHA-256 ハッシュチェーンの週次チェックポイント（Append-only）。

-- EN-025 HashChainBlock — SHA-256 ハッシュチェーンの週次チェックポイント（Append-only）。
-- BAT-001 が週次で生成する。7年以上保存。
CREATE TABLE IF NOT EXISTS hash_chain_blocks (
    -- ブロック識別子。UUID v4。gen_random_uuid() で自動生成。
    block_id           UUID     NOT NULL DEFAULT gen_random_uuid(),
    -- 集計週の開始日（月曜日）。UNIQUE 制約により週次 1 レコードを保証する。
    block_period       DATE     NOT NULL,
    -- 期間内のイベント件数。0 より大きい値のみ許可（CHECK 制約）。
    event_count        INTEGER  NOT NULL,
    -- 期間内最終 WorkEvent の event_id。次週の BAT-001 がチェーン継続点として使用する。
    last_event_id      UUID     NOT NULL,
    -- 期間内最終 WorkEvent の content_hash（64 文字 hex）。
    last_content_hash  CHAR(64) NOT NULL,
    -- レコード作成時刻（BAT-001 実行時刻）。
    created_at         TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    -- このブロック自体のハッシュ。SHA-256(block_period || last_content_hash)。
    block_hash         CHAR(64) NOT NULL,

    -- 主キー
    CONSTRAINT pk_hash_chain_blocks PRIMARY KEY (block_id),
    -- block_period は週次 1 レコードを保証するために UNIQUE でなければならない。
    CONSTRAINT uq_hash_chain_blocks_period UNIQUE (block_period),
    -- event_count は 0 より大きい値のみ許可する。
    CONSTRAINT ck_hash_chain_event_count_positive CHECK (event_count > 0),
    -- last_content_hash と block_hash は 64 文字（SHA-256 hex）でなければならない。
    CONSTRAINT ck_hash_chain_hash_lengths CHECK (
        length(last_content_hash) = 64 AND length(block_hash) = 64
    )
);

COMMENT ON TABLE  hash_chain_blocks IS 'EN-025 HashChainBlock — 週次ハッシュチェーンチェックポイント。BAT-001 が週次で生成する。7年以上保存。Append-only。';
COMMENT ON COLUMN hash_chain_blocks.block_period      IS '集計週の開始日（月曜日）。UNIQUE 制約により週次 1 レコードを保証。';
COMMENT ON COLUMN hash_chain_blocks.last_content_hash IS '期間内最終 WorkEvent の content_hash。次週の BAT-001 がチェーン継続点として使用する。';
COMMENT ON COLUMN hash_chain_blocks.block_hash        IS 'このブロック自体のハッシュ。SHA-256(block_period || last_content_hash)。';

-- Append-only 強制: app_event_writer ロールから UPDATE/DELETE を剥奪する
REVOKE UPDATE, DELETE ON hash_chain_blocks FROM PUBLIC;
REVOKE UPDATE, DELETE ON hash_chain_blocks FROM app_event_writer;
-- app_event_writer に INSERT と SELECT のみを許可する
GRANT INSERT, SELECT ON hash_chain_blocks TO app_event_writer;
