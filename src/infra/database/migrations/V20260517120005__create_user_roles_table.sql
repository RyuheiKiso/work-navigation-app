-- V20260517120005__create_user_roles_table.sql
-- TBL-019 user_roles: EN-001 × EN-002 N:M 中間テーブル。ユーザーへのロール付与記録。

-- EN-001 × EN-002 N:M 中間テーブル — ユーザーへのロール付与記録
CREATE TABLE IF NOT EXISTS user_roles (
    -- ロールを付与されたユーザーの識別子。
    user_id     UUID        NOT NULL,
    -- 付与されたロールの識別子。
    role_id     UUID        NOT NULL,
    -- ロール付与日時。
    granted_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    -- ロールを付与したユーザーの user_id。system_admin ロールのみ操作可能（アプリ層で制御）。
    granted_by  UUID        NOT NULL,

    -- 複合主キー。1 ユーザーに同一ロールは 1 レコードのみ許可する。
    CONSTRAINT pk_user_roles PRIMARY KEY (user_id, role_id),
    -- ユーザー参照外部キー。ユーザー削除時は RESTRICT（物理削除禁止のため発動しない前提）。
    CONSTRAINT fk_user_roles_user FOREIGN KEY (user_id)
        REFERENCES users (user_id) ON DELETE RESTRICT,
    -- ロール参照外部キー。ロール削除時は RESTRICT。
    CONSTRAINT fk_user_roles_role FOREIGN KEY (role_id)
        REFERENCES roles (role_id) ON DELETE RESTRICT,
    -- ロール付与者参照外部キー。付与者ユーザー削除時は RESTRICT。
    CONSTRAINT fk_user_roles_granted_by FOREIGN KEY (granted_by)
        REFERENCES users (user_id) ON DELETE RESTRICT
);

COMMENT ON TABLE  user_roles IS 'EN-001×EN-002 N:M 中間テーブル。ロール付与の証跡として granted_by を必須とする。';
COMMENT ON COLUMN user_roles.user_id    IS 'ロールを付与されたユーザーの user_id。';
COMMENT ON COLUMN user_roles.role_id    IS '付与されたロールの role_id。';
COMMENT ON COLUMN user_roles.granted_at IS 'ロール付与日時。';
COMMENT ON COLUMN user_roles.granted_by IS 'ロールを付与したユーザーの user_id。system_admin ロールのみ操作可能（アプリ層で制御）。';
