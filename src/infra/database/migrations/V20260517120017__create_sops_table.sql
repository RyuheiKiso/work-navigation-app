-- V20260517120017__create_sops_table.sql
-- TBL-007 sops: 作業手順書マスタ（版管理）。sop_code 形式: {operation_code}-SOP-{連番3桁}。

-- EN-008 SOP — 作業手順書マスタ（版管理）
CREATE TABLE IF NOT EXISTS sops (
    -- SOP 識別子。UUID v7（時系列順）。Rust 側で生成する。
    sop_id              UUID        NOT NULL DEFAULT gen_random_uuid(),
    -- 所属するオペレーションの識別子。
    operation_id        UUID        NOT NULL,
    -- SOP コード。形式: {operation_code}-SOP-{連番3桁}。変更不可の公開識別子。
    sop_code            VARCHAR(64) NOT NULL,
    -- 現在有効版の master_version_id。PUBLISHED 状態のレコードのみ設定する（アプリ層で制御）。NULL は未公開。
    current_version_id  UUID        NULL,
    -- 有効フラグ。廃止時に FALSE に設定する。物理 DELETE は禁止。
    is_active           BOOLEAN     NOT NULL DEFAULT TRUE,
    -- レコード作成日時。
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    -- レコード最終更新日時。
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT pk_sops PRIMARY KEY (sop_id),
    CONSTRAINT uq_sops_code UNIQUE (sop_code),
    -- operations テーブルへの外部キー。オペレーション削除時は RESTRICT。
    CONSTRAINT fk_sops_operation FOREIGN KEY (operation_id)
        REFERENCES operations (operation_id) ON DELETE RESTRICT,
    -- master_versions テーブルへの外部キー。版削除時は RESTRICT。
    CONSTRAINT fk_sops_current_version FOREIGN KEY (current_version_id)
        REFERENCES master_versions (master_version_id) ON DELETE RESTRICT
);

COMMENT ON TABLE  sops IS 'EN-008 SOP — 作業手順書マスタ。sop_code 形式: {operation_code}-SOP-{連番3桁}。current_version_id は PUBLISHED 状態の最新版を指す。';
COMMENT ON COLUMN sops.sop_id             IS 'UUID v7（時系列順）。Rust 側で生成する。';
COMMENT ON COLUMN sops.operation_id       IS '所属するオペレーションの operation_id。';
COMMENT ON COLUMN sops.sop_code           IS 'SOP コード。形式: {operation_code}-SOP-{連番3桁}。変更不可の公開識別子。';
COMMENT ON COLUMN sops.current_version_id IS '現在有効版の master_version_id。PUBLISHED 状態のレコードのみ設定する（アプリ層で制御）。NULL は未公開。';
COMMENT ON COLUMN sops.is_active          IS '廃止時に FALSE に設定する。物理 DELETE は禁止。';
COMMENT ON COLUMN sops.created_at         IS 'レコード作成日時。';
COMMENT ON COLUMN sops.updated_at         IS 'レコード最終更新日時。';
