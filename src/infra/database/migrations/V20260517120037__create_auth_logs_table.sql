-- V20260517120037__create_auth_logs_table.sql
-- TBL-032 auth_logs: 認証イベントログ（Append-only）。90日保存後アーカイブ。

-- EN-022 AuthLog — 認証イベントログ（Append-only）。
-- user_id は NULL 許容（未認証ユーザーの失敗ログ）。90日保存後アーカイブ。
CREATE TABLE IF NOT EXISTS auth_logs (
    -- 認証ログ識別子。UUID v4。gen_random_uuid() で自動生成。
    log_id      UUID        NOT NULL DEFAULT gen_random_uuid(),
    -- 認証イベントに紐付くユーザー識別子。LOGIN_FAILURE 時は NULL 許容（未認証ユーザーの失敗ログ）。
    user_id     UUID        NULL,
    -- 認証アクション種別。6 種の列挙値のみ許可（CHECK 制約）。
    action      VARCHAR(64) NOT NULL,
    -- 認証に使用したデバイスの識別子。NULL 許容（デバイス情報取得不可の場合）。
    device_id   UUID        NULL,
    -- クライアント IP アドレス。プライバシー保護のため /24 マスク推奨（アプリ層でマスク処理）。
    ip_address  INET        NULL,
    -- 認証イベント発生時刻。
    occurred_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- 主キー
    CONSTRAINT pk_auth_logs PRIMARY KEY (log_id),
    -- ユーザーへの外部キー。NULL 許容（未認証ユーザーの失敗ログに対応）。
    CONSTRAINT fk_auth_logs_user FOREIGN KEY (user_id)
        REFERENCES users (user_id) ON DELETE RESTRICT,
    -- デバイスへの外部キー。NULL 許容。
    CONSTRAINT fk_auth_logs_device FOREIGN KEY (device_id)
        REFERENCES devices (device_id) ON DELETE RESTRICT,
    -- action は 6 種の列挙値のみ許可する。
    CONSTRAINT ck_auth_logs_action CHECK (
        action IN (
            'LOGIN_SUCCESS',
            'LOGIN_FAILURE',
            'LOGOUT',
            'TOKEN_REFRESH',
            'PASSWORD_CHANGE',
            'ACCOUNT_LOCKED'
        )
    )
);

COMMENT ON TABLE  auth_logs IS 'EN-022 AuthLog — 認証イベントログ。Append-only。user_id は NULL 許容（未認証ユーザーの失敗ログ）。90日保存後アーカイブ。';
COMMENT ON COLUMN auth_logs.ip_address IS 'クライアント IP アドレス。プライバシー保護のため /24 マスク推奨（例: 192.168.1.0）。アプリ層でマスク処理する。';
COMMENT ON COLUMN auth_logs.user_id    IS 'LOGIN_FAILURE 時は NULL または存在しない user_id の可能性あり。外部キー制約は ON DELETE RESTRICT のため廃止ユーザーの参照が残る。';

-- Append-only 強制: app_event_writer ロールから UPDATE/DELETE を剥奪する
REVOKE UPDATE, DELETE ON auth_logs FROM PUBLIC;
REVOKE UPDATE, DELETE ON auth_logs FROM app_event_writer;
-- app_event_writer に INSERT と SELECT のみを許可する
GRANT INSERT, SELECT ON auth_logs TO app_event_writer;
