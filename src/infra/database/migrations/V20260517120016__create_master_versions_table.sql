-- V20260517120016__create_master_versions_table.sql
-- TBL-004 master_versions: SOP・Step・StepTypeDefinition・StepFlowRule の版数管理テーブル。

-- EN-010 MasterVersion — SOP・Step・StepTypeDefinition・StepFlowRule の版数管理
CREATE TABLE IF NOT EXISTS master_versions (
    -- バージョン識別子。UUID v7（時系列順）。Rust 側で生成する。
    master_version_id  UUID         NOT NULL DEFAULT gen_random_uuid(),
    -- 版数対象マスタの種別。SOP / STEP / STEP_TYPE / FLOW_RULE の 4 値。
    master_type        VARCHAR(32)  NOT NULL,
    -- 版数対象マスタの主キー（sop_id / step_id / step_type_definition_id / step_flow_rule_id）。
    master_id          UUID         NOT NULL,
    -- 1 から始まる増分番号。同一 master_type + master_id の最大値 + 1 を Rust 層で計算して INSERT する。
    version_number     SMALLINT     NOT NULL,
    -- DRAFT → UNDER_REVIEW → PUBLISHED → ARCHIVED の順序で遷移。逆遷移不可（アプリ層で制御）。
    status             VARCHAR(16)  NOT NULL DEFAULT 'DRAFT',
    -- 施行日（PUBLISHED 状態への遷移時に設定する）。NULL は未設定。
    effective_date     DATE         NULL,
    -- 版を作成したユーザーの識別子。
    created_by         UUID         NOT NULL,
    -- 版を公開承認したユーザーの識別子。PUBLISHED / ARCHIVED 時は必須。
    published_by       UUID         NULL,
    -- PUBLISHED 遷移時に付与する電子サインの sign_id。ALCOA+ Original 要件。
    sign_id            UUID         NULL,
    -- レコード作成日時。
    created_at         TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    -- レコード最終更新日時。
    updated_at         TIMESTAMPTZ  NOT NULL DEFAULT NOW(),

    CONSTRAINT pk_master_versions PRIMARY KEY (master_version_id),
    -- 同一マスタタイプ・同一マスタ ID の同一バージョン番号は 1 つのみ許可する。
    CONSTRAINT uq_master_versions_composite UNIQUE (master_type, master_id, version_number),
    -- users テーブルへの外部キー（作成者）。
    CONSTRAINT fk_master_versions_created_by FOREIGN KEY (created_by)
        REFERENCES users (user_id) ON DELETE RESTRICT,
    -- users テーブルへの外部キー（公開承認者）。
    CONSTRAINT fk_master_versions_published_by FOREIGN KEY (published_by)
        REFERENCES users (user_id) ON DELETE RESTRICT,
    -- electronic_signs テーブルへの外部キー。
    CONSTRAINT fk_master_versions_sign FOREIGN KEY (sign_id)
        REFERENCES electronic_signs (sign_id) ON DELETE RESTRICT,
    -- master_type は 4 値のみ許可する。
    CONSTRAINT ck_master_versions_type CHECK (
        master_type IN ('SOP', 'STEP', 'STEP_TYPE', 'FLOW_RULE')
    ),
    -- status は 4 値のみ許可する。
    CONSTRAINT ck_master_versions_status CHECK (
        status IN ('DRAFT', 'UNDER_REVIEW', 'PUBLISHED', 'ARCHIVED')
    ),
    -- version_number は 1 以上の正整数のみ許可する。
    CONSTRAINT ck_master_versions_version_positive CHECK (version_number > 0),
    -- PUBLISHED / ARCHIVED 状態では published_by は必須とする。
    CONSTRAINT ck_master_versions_published_requires_publisher CHECK (
        NOT (status IN ('PUBLISHED', 'ARCHIVED') AND published_by IS NULL)
    )
);

COMMENT ON TABLE  master_versions IS 'EN-010 MasterVersion — SOP/Step/StepTypeDefinition/StepFlowRule の版数管理テーブル。PUBLISHED 状態への遷移後は status 以外の列を変更しない（アプリ層で強制）。';
COMMENT ON COLUMN master_versions.master_version_id IS 'UUID v7（時系列順）。Rust 側で生成する。';
COMMENT ON COLUMN master_versions.master_type       IS 'SOP / STEP / STEP_TYPE / FLOW_RULE の 4 値。';
COMMENT ON COLUMN master_versions.master_id         IS '版数対象マスタの主キー（sop_id / step_id / step_type_definition_id / step_flow_rule_id）。';
COMMENT ON COLUMN master_versions.version_number    IS '1 から始まる増分番号。同一 master_type + master_id の最大値 + 1 を Rust 層で計算して INSERT する。';
COMMENT ON COLUMN master_versions.status            IS 'DRAFT → UNDER_REVIEW → PUBLISHED → ARCHIVED の順序で遷移。逆遷移不可（アプリ層で制御）。';
COMMENT ON COLUMN master_versions.effective_date    IS '施行日（PUBLISHED 状態への遷移時に設定する）。NULL は未設定。';
COMMENT ON COLUMN master_versions.created_by        IS '版を作成したユーザーの user_id。';
COMMENT ON COLUMN master_versions.published_by      IS '版を公開承認したユーザーの user_id。PUBLISHED / ARCHIVED 時は必須（CHECK 制約）。';
COMMENT ON COLUMN master_versions.sign_id           IS 'PUBLISHED 遷移時に付与する電子サインの sign_id。ALCOA+ Original 要件。';
COMMENT ON COLUMN master_versions.created_at        IS 'レコード作成日時。';
COMMENT ON COLUMN master_versions.updated_at        IS 'レコード最終更新日時。';
