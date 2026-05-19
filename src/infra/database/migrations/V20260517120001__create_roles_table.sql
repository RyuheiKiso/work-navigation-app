-- V20260517120001__create_roles_table.sql
-- TBL-017 roles: 固定 6 種の権限ロールマスタ。CHECK 制約で新種追加を防止する（スキーマ変更必須）。

-- EN-002 Role — 固定 6 種の権限ロールマスタ。アプリ起動時にシードデータを INSERT し、以後 UPDATE しない。
CREATE TABLE IF NOT EXISTS roles (
    -- ロール識別子。固定 UUID。シードデータで定数値を使用する。
    role_id     UUID            NOT NULL,
    -- ロール名称。6 値のみ許可（CHECK 制約）。
    role_name   VARCHAR(64)     NOT NULL,
    -- ロールの説明文。
    description TEXT            NOT NULL DEFAULT '',
    -- レコード作成日時。
    created_at  TIMESTAMPTZ     NOT NULL DEFAULT NOW(),

    CONSTRAINT pk_roles PRIMARY KEY (role_id),
    CONSTRAINT uq_roles_name UNIQUE (role_name),
    -- role_name は固定 6 値のみ許可する。新種追加はスキーマ変更必須。
    CONSTRAINT ck_roles_name_valid CHECK (
        role_name IN ('operator', 'supervisor', 'master_admin', 'quality_admin', 'system_admin', 'executive')
    )
);

COMMENT ON TABLE  roles IS 'EN-002 Role — 固定 6 種の権限ロール。CHECK 制約で新種追加を防止する（スキーマ変更必須）。';
COMMENT ON COLUMN roles.role_id   IS '固定 UUID。シードデータで定数値を使用する（例: 00000000-0000-7000-8000-000000000001）。';
COMMENT ON COLUMN roles.role_name IS 'operator / supervisor / master_admin / quality_admin / system_admin / executive の 6 値のみ許可。';
COMMENT ON COLUMN roles.description IS 'ロールの説明文。シードデータで設定する。';
COMMENT ON COLUMN roles.created_at IS 'レコード作成日時。シードデータ投入時刻。';
