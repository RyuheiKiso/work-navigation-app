-- 対応 §: ロードマップ §10.6 §10.6.1 §27 F-002 §29 R-016
-- LWW-Register（ユーザー設定）と Idempotency-Key（24h 重複排除窓）の永続化スキーマ。
-- 同期形式モデル（ADR-0003）に従い、(lamport_ts, device_id) の lex 順で決定的に値が定まる。

-- =====================================================================
-- user_settings: LWW-Register
-- =====================================================================
CREATE TABLE IF NOT EXISTS user_settings (
    -- 設定の論理キー（LWW-Register の key）
    setting_key          TEXT PRIMARY KEY,
    -- 値（JSON 文字列。LWW での更新で書き換わる）
    value                TEXT NOT NULL,
    -- Lamport タイムスタンプ（INV-08）
    lamport              BIGINT NOT NULL CHECK (lamport >= 0),
    -- 主体端末（lex 順比較で決定的に値を選ぶ）
    device_id            TEXT NOT NULL,
    -- 更新時刻（UTC、§20.2）
    updated_at           TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- LWW 最大値を高速に取得するための補助索引
CREATE INDEX IF NOT EXISTS idx_user_settings_lamport
    ON user_settings (lamport DESC, device_id DESC);

-- =====================================================================
-- idempotency_keys: 24h 重複排除窓（§10.3.1 F-005）
-- =====================================================================
CREATE TABLE IF NOT EXISTS idempotency_keys (
    -- Idempotency-Key 文字列
    key                  TEXT PRIMARY KEY,
    -- 観測時刻（UTC）
    seen_at              TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    -- 関連リソース（任意。順序情報なら order_id）
    related_resource     TEXT
);

-- 期限切れ判定用の索引
CREATE INDEX IF NOT EXISTS idx_idempotency_keys_seen_at
    ON idempotency_keys (seen_at);

-- =====================================================================
-- credentials: ID＋パスワード認証（ADR-0007、F-006）
-- =====================================================================
CREATE TABLE IF NOT EXISTS credentials (
    -- 利用者 ID
    user_id              TEXT PRIMARY KEY,
    -- 表示名
    display_name         TEXT NOT NULL,
    -- アカウント有効化フラグ（§10.5.1）
    enabled              BOOLEAN NOT NULL DEFAULT TRUE,
    -- Argon2id PHC ハッシュ
    password_hash        TEXT NOT NULL,
    -- 作成・更新時刻
    created_at           TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at           TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
