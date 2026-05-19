-- V20260517120039__create_idempotency_keys_table.sql
-- TBL-035 idempotency_keys: API 冪等性キーキャッシュ（TTL 24時間）。全体方針の DELETE 禁止の唯一例外。

-- 制御テーブル — API 冪等性キーキャッシュ。TTL 24 時間。
-- 唯一 DELETE が許可される制御テーブル（pg_cron または BAT-003 が 24 時間超のレコードを DELETE する）。
CREATE TABLE IF NOT EXISTS idempotency_keys (
    -- クライアントが送信する Idempotency-Key ヘッダ値（UUID v4）。PRIMARY KEY により一意性を保証する。
    idempotency_key  UUID         NOT NULL,
    -- 対応する API エンドポイントパス。例: /api/v1/work-events。
    endpoint         VARCHAR(128) NOT NULL,
    -- 最初のリクエストに対するレスポンス HTTP ステータスコード。100〜599 の範囲のみ許可。
    response_status  SMALLINT     NOT NULL,
    -- 最初のリクエスト応答 JSON。同一キーの再送時にこのボディを返却する（アプリ層で制御）。
    response_body    JSONB        NOT NULL DEFAULT '{}',
    -- レコード作成時刻。TTL 24 時間の基点となる。
    created_at       TIMESTAMPTZ  NOT NULL DEFAULT NOW(),

    -- 主キー（Idempotency-Key の UUID が PK）
    CONSTRAINT pk_idempotency_keys PRIMARY KEY (idempotency_key),
    -- response_status は 100〜599 の有効な HTTP ステータスコードの範囲のみ許可する。
    CONSTRAINT ck_idempotency_response_status CHECK (
        response_status BETWEEN 100 AND 599
    )
);

COMMENT ON TABLE  idempotency_keys IS '制御テーブル。API 冪等性キーキャッシュ（TTL 24 時間）。pg_cron または BAT-003 が 24 時間超のレコードを DELETE する（全体方針の DELETE 禁止の唯一例外）。';
COMMENT ON COLUMN idempotency_keys.idempotency_key IS 'クライアントが送信する Idempotency-Key ヘッダ値（UUID v4）。PRIMARY KEY により一意性を保証。';
COMMENT ON COLUMN idempotency_keys.response_body   IS '最初のリクエスト応答 JSON。同一キーの再送時にこのボディを返却する（アプリ層で制御）。';
