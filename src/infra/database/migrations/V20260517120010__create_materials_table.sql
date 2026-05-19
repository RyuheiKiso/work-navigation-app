-- V20260517120010__create_materials_table.sql
-- TBL-036 materials: 材料・部品・工具・包材マスタ（版管理）。物理削除禁止（is_active=FALSE で論理削除）。

-- EN-028 Material — 材料・部品・工具・包材マスタ（版管理）
CREATE TABLE IF NOT EXISTS materials (
    -- 材料識別子。UUID v7（時系列順）。Rust 側で生成する。
    material_id     UUID            NOT NULL DEFAULT gen_random_uuid(),
    -- 材料コード（購買システム連携キー）。UNIQUE 制約。
    material_code   VARCHAR(64)     NOT NULL,
    -- 材料名称。
    name            VARCHAR(256)    NOT NULL,
    -- 材料種別。RAW_MATERIAL=原材料 / COMPONENT=部品 / TOOL=工具 / PACKAGING=包材。
    material_type   TEXT            NOT NULL,
    -- 材料の説明文。
    description     TEXT            NOT NULL DEFAULT '',
    -- レコードバージョン番号。1 以上の正整数。更新のたびにインクリメントする。
    version         INTEGER         NOT NULL DEFAULT 1,
    -- 有効フラグ。廃止時に FALSE に設定する。物理 DELETE は禁止。
    is_active       BOOLEAN         NOT NULL DEFAULT TRUE,
    -- レコード作成日時。
    created_at      TIMESTAMPTZ     NOT NULL DEFAULT NOW(),
    -- レコード最終更新日時。
    updated_at      TIMESTAMPTZ     NOT NULL DEFAULT NOW(),

    CONSTRAINT pk_materials PRIMARY KEY (material_id),
    CONSTRAINT uq_materials_code UNIQUE (material_code),
    -- material_type は 4 値のみ許可する。
    CONSTRAINT ck_materials_type CHECK (
        material_type IN ('RAW_MATERIAL', 'COMPONENT', 'TOOL', 'PACKAGING')
    ),
    -- version は 1 以上の正整数のみ許可する。
    CONSTRAINT ck_materials_version CHECK (version >= 1)
);

COMMENT ON TABLE  materials IS 'EN-028 Material — 材料マスタ。物理削除禁止（is_active=FALSE で論理削除）。';
COMMENT ON COLUMN materials.material_id   IS 'UUID v7（時系列順）。Rust 側で生成する。';
COMMENT ON COLUMN materials.material_code IS '材料コード（購買システム連携キー）。UNIQUE 制約。';
COMMENT ON COLUMN materials.name          IS '材料名称。';
COMMENT ON COLUMN materials.material_type IS 'RAW_MATERIAL=原材料 / COMPONENT=部品 / TOOL=工具 / PACKAGING=包材。';
COMMENT ON COLUMN materials.description   IS '材料の説明文。';
COMMENT ON COLUMN materials.version       IS 'レコードバージョン番号。1 以上の正整数。更新のたびにインクリメントする。';
COMMENT ON COLUMN materials.is_active     IS '廃止時に FALSE に設定する。物理 DELETE は禁止。';
COMMENT ON COLUMN materials.created_at    IS 'レコード作成日時。';
COMMENT ON COLUMN materials.updated_at    IS 'レコード最終更新日時。';
