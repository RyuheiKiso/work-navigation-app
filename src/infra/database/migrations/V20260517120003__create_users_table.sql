-- V20260517120003__create_users_table.sql
-- TBL-016 users: 作業員・管理者・システムユーザーを統合管理するマスタ。物理削除禁止。

-- EN-001 User — 作業員・管理者・システムユーザーを統合管理するマスタ
CREATE TABLE IF NOT EXISTS users (
    -- ユーザー識別子。UUID v7（時系列順）。Rust 側で生成し INSERT する。WorkEvent.resource FK として不変。
    user_id         UUID            NOT NULL DEFAULT gen_random_uuid(),
    -- ログイン識別子。LDAP 連携時は LDAP DN 形式。匿名化後は内部 UUID 文字列に置換される。
    login_id        VARCHAR(128)    NOT NULL,
    -- 表示名。匿名化後は "anonymized-{user_id 前 8 桁}" に置換される。空白のみ禁止（trim 後 1 文字以上必須）。
    display_name    VARCHAR(256)    NOT NULL,
    -- 有効フラグ。退職・無効化時に FALSE。物理 DELETE は禁止。
    is_active       BOOLEAN         NOT NULL DEFAULT TRUE,
    -- PII 匿名化実施時刻。CFG-010 で設定された日数（デフォルト 60 日）経過後に BAT-004 が設定する。
    anonymized_at   TIMESTAMPTZ     NULL,
    -- レコード作成日時。
    created_at      TIMESTAMPTZ     NOT NULL DEFAULT NOW(),
    -- レコード最終更新日時。
    updated_at      TIMESTAMPTZ     NOT NULL DEFAULT NOW(),

    CONSTRAINT pk_users PRIMARY KEY (user_id),
    CONSTRAINT uq_users_login_id UNIQUE (login_id),
    -- display_name は空白のみ禁止する（trim 後 1 文字以上必須）。
    CONSTRAINT ck_users_display_name_not_empty CHECK (length(trim(display_name)) > 0),
    -- is_active=TRUE のユーザーが匿名化済みであってはならない（論理的整合性）。
    CONSTRAINT ck_users_anonymized_active CHECK (
        NOT (is_active = TRUE AND anonymized_at IS NOT NULL)
    )
);

COMMENT ON TABLE  users IS 'EN-001 User — 作業員・管理者・システムユーザーマスタ。物理削除禁止。退職後 is_active=FALSE、60日後に BAT-004 が PII を匿名化する。';
COMMENT ON COLUMN users.user_id        IS 'UUID v7（時系列順）。Rust 側で生成し INSERT する。WorkEvent.resource FK として不変。';
COMMENT ON COLUMN users.login_id       IS 'ログイン識別子。LDAP 連携時は LDAP DN 形式。匿名化後は内部 UUID 文字列に置換される。';
COMMENT ON COLUMN users.display_name   IS '表示名。匿名化後は "anonymized-{user_id 前 8 桁}" に置換される。';
COMMENT ON COLUMN users.is_active      IS '退職・無効化時に FALSE。物理 DELETE は禁止。';
COMMENT ON COLUMN users.anonymized_at  IS 'PII 匿名化実施時刻。CFG-010 で設定された日数（デフォルト 60 日）経過後に BAT-004 が設定する。';
COMMENT ON COLUMN users.created_at     IS 'レコード作成日時。';
COMMENT ON COLUMN users.updated_at     IS 'レコード最終更新日時。';
