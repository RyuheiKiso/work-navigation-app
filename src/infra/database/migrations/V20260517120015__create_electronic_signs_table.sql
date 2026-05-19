-- V20260517120015__create_electronic_signs_table.sql
-- TBL-002 electronic_signs: 電子サインレコード。Append-only。ALCOA+ Original / Attributable 要件。

-- EN-015 ElectronicSign — 電子サインレコード。Append-only。ALCOA+ 承認証拠。
CREATE TABLE IF NOT EXISTS electronic_signs (
    -- サイン識別子。UUID v7（時系列順）。Rust 側で生成する。
    sign_id         UUID        NOT NULL DEFAULT gen_random_uuid(),
    -- 署名者のユーザー識別子。
    signer_id       UUID        NOT NULL,
    -- 署名日時。
    signed_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    -- 署名目的の列挙値。6 値のみ許可する。
    sign_purpose    VARCHAR(64) NOT NULL,
    -- 署名対象エンティティの種別。5 値のみ許可する。
    target_type     VARCHAR(32) NOT NULL,
    -- 署名対象エンティティの識別子（UUID）。
    target_id       UUID        NOT NULL,
    -- 署名方式。PIN / BIOMETRIC / PASSWORD / HARDWARE_TOKEN の 4 値。
    sign_method     VARCHAR(32) NOT NULL DEFAULT 'PIN',
    -- 認証資格情報のハッシュ（生 PIN・パスワードは保存しない）。SHA-256 の 64 文字 hex。
    credential_hash CHAR(64)    NOT NULL,
    -- 署名時のクライアント IP。プライバシー保護のため /24 マスクを推奨（アプリ層で制御）。
    ip_address      INET        NULL,
    -- 署名に使用したデバイスの識別子。NULL はデスクトップ操作。
    device_id       UUID        NULL,

    CONSTRAINT pk_electronic_signs PRIMARY KEY (sign_id),
    -- users テーブルへの外部キー。署名者ユーザー削除時は RESTRICT。
    CONSTRAINT fk_electronic_signs_signer FOREIGN KEY (signer_id)
        REFERENCES users (user_id) ON DELETE RESTRICT,
    -- devices テーブルへの外部キー。デバイス削除時は RESTRICT。
    CONSTRAINT fk_electronic_signs_device FOREIGN KEY (device_id)
        REFERENCES devices (device_id) ON DELETE RESTRICT,
    -- sign_purpose は 6 値のみ許可する。
    CONSTRAINT ck_electronic_signs_purpose CHECK (
        sign_purpose IN (
            'step_completed_approval',
            'work_completed_approval',
            'master_publish_approval',
            'suspension_approval',
            'capa_closure_approval',
            'nonconformity_closure_approval'
        )
    ),
    -- target_type は 5 値のみ許可する。
    CONSTRAINT ck_electronic_signs_target_type CHECK (
        target_type IN ('work_event', 'master_version', 'suspension', 'capa', 'nonconformity')
    ),
    -- sign_method は 4 値のみ許可する。
    CONSTRAINT ck_electronic_signs_method CHECK (
        sign_method IN ('PIN', 'BIOMETRIC', 'PASSWORD', 'HARDWARE_TOKEN')
    ),
    -- credential_hash は SHA-256 の 64 文字 hex 固定長とする。
    CONSTRAINT ck_electronic_signs_credential_hash_length CHECK (length(credential_hash) = 64)
);

COMMENT ON TABLE  electronic_signs IS 'EN-015 ElectronicSign — 電子サインレコード。Append-only。ALCOA+ Original / Attributable 要件。sign_id は master_versions・suspensions・capas から FK 参照される。';
COMMENT ON COLUMN electronic_signs.sign_id         IS 'UUID v7（時系列順）。Rust 側で生成する。';
COMMENT ON COLUMN electronic_signs.signer_id       IS '署名者のユーザー識別子。';
COMMENT ON COLUMN electronic_signs.signed_at       IS '署名日時。';
COMMENT ON COLUMN electronic_signs.sign_purpose    IS '署名目的の列挙値。';
COMMENT ON COLUMN electronic_signs.target_type     IS '署名対象エンティティの種別。';
COMMENT ON COLUMN electronic_signs.target_id       IS '署名対象エンティティの識別子（UUID）。';
COMMENT ON COLUMN electronic_signs.sign_method     IS 'PIN / BIOMETRIC / PASSWORD / HARDWARE_TOKEN の 4 値。';
COMMENT ON COLUMN electronic_signs.credential_hash IS '認証資格情報のハッシュ（生 PIN・パスワードは保存しない）。SHA-256 の 64 文字 hex。';
COMMENT ON COLUMN electronic_signs.ip_address      IS '署名時のクライアント IP。プライバシー保護のため /24 マスク推奨（例: 192.168.1.0）。アプリ層でマスク処理する。';
COMMENT ON COLUMN electronic_signs.device_id       IS '署名に使用したデバイスの識別子。NULL はデスクトップ操作。';

-- Append-only 保証: UPDATE/DELETE を全ロールから REVOKE する。
REVOKE UPDATE, DELETE ON electronic_signs FROM PUBLIC;
REVOKE UPDATE, DELETE ON electronic_signs FROM app_event_writer;
