-- V20260517120023__create_sampling_plans_table.sql
-- TBL-039 sampling_plans: AQL サンプリング計画マスタ（版管理・時点固定 JSONB）。

-- EN-031 SamplingPlan — AQL サンプリング計画マスタ（版管理・時点固定 JSONB）
CREATE TABLE IF NOT EXISTS sampling_plans (
    -- サンプリング計画識別子。UUID v7（時系列順）。Rust 側で生成する。
    plan_id                 UUID            NOT NULL DEFAULT gen_random_uuid(),
    -- 対象材料の識別子。
    material_id             UUID            NOT NULL,
    -- 対象仕入先の識別子。
    supplier_id             UUID            NOT NULL,
    -- AQL 値（Acceptable Quality Level）。0 より大きい値のみ許可。
    aql                     NUMERIC(5,2)    NOT NULL,
    -- JIS Z 9015-1 の検査水準（S-1/S-2/S-3/S-4/I/II/III）。デフォルトは II（なみ検査）。
    inspection_level        TEXT            NOT NULL DEFAULT 'II',
    -- JIS Z 9015-1 サンプル文字表 + AQL マスタ表の時点固定スナップショット（JSONB）。作成後に変更しない。
    aql_table_snapshot      JSONB           NOT NULL,
    -- レコードバージョン番号。1 以上の正整数。更新のたびにインクリメントする。
    version                 INTEGER         NOT NULL DEFAULT 1,
    -- 有効フラグ。廃止時に FALSE に設定する。物理 DELETE は禁止。
    is_active               BOOLEAN         NOT NULL DEFAULT TRUE,
    -- 計画を作成したユーザーの識別子。
    created_by              UUID            NOT NULL,
    -- レコード作成日時。
    created_at              TIMESTAMPTZ     NOT NULL DEFAULT NOW(),
    -- レコード最終更新日時。
    updated_at              TIMESTAMPTZ     NOT NULL DEFAULT NOW(),

    CONSTRAINT pk_sampling_plans PRIMARY KEY (plan_id),
    -- materials テーブルへの外部キー。
    CONSTRAINT fk_sampling_plans_material FOREIGN KEY (material_id)
        REFERENCES materials (material_id),
    -- suppliers テーブルへの外部キー。
    CONSTRAINT fk_sampling_plans_supplier FOREIGN KEY (supplier_id)
        REFERENCES suppliers (supplier_id),
    -- users テーブルへの外部キー（計画作成者）。
    CONSTRAINT fk_sampling_plans_creator  FOREIGN KEY (created_by)
        REFERENCES users (user_id),
    -- aql は 0 より大きい値のみ許可する。
    CONSTRAINT ck_sampling_plans_aql CHECK (aql > 0),
    -- inspection_level は 7 値のみ許可する（JIS Z 9015-1 準拠）。
    CONSTRAINT ck_sampling_plans_level CHECK (
        inspection_level IN ('S-1', 'S-2', 'S-3', 'S-4', 'I', 'II', 'III')
    ),
    -- version は 1 以上の正整数のみ許可する。
    CONSTRAINT ck_sampling_plans_version CHECK (version >= 1),
    -- aql_table_snapshot は JSONB オブジェクト型のみ許可する。
    CONSTRAINT ck_sampling_plans_snapshot_is_object CHECK (
        jsonb_typeof(aql_table_snapshot) = 'object'
    )
);

COMMENT ON TABLE  sampling_plans IS 'EN-031 SamplingPlan — AQL 計画マスタ。aql_table_snapshot に JIS Z 9015-1 の n/Ac/Re 表を時点固定で JSONB スナップショットとして保存する。';
COMMENT ON COLUMN sampling_plans.plan_id            IS 'UUID v7（時系列順）。Rust 側で生成する。';
COMMENT ON COLUMN sampling_plans.material_id        IS '対象材料の material_id。';
COMMENT ON COLUMN sampling_plans.supplier_id        IS '対象仕入先の supplier_id。';
COMMENT ON COLUMN sampling_plans.aql                IS 'AQL 値（Acceptable Quality Level）。JIS Z 9015-1 の規定値（例: 0.65, 1.0, 2.5, 4.0）。';
COMMENT ON COLUMN sampling_plans.inspection_level   IS 'JIS Z 9015-1 の検査水準（S-1/S-2/S-3/S-4/I/II/III）。';
COMMENT ON COLUMN sampling_plans.aql_table_snapshot IS 'JIS Z 9015-1 サンプル文字表 + AQL マスタ表の時点固定スナップショット（JSONB）。作成後に変更しない。';
COMMENT ON COLUMN sampling_plans.version            IS 'レコードバージョン番号。1 以上の正整数。更新のたびにインクリメントする。';
COMMENT ON COLUMN sampling_plans.is_active          IS '廃止時に FALSE に設定する。物理 DELETE は禁止。';
COMMENT ON COLUMN sampling_plans.created_by         IS '計画を作成したユーザーの user_id。';
COMMENT ON COLUMN sampling_plans.created_at         IS 'レコード作成日時。';
COMMENT ON COLUMN sampling_plans.updated_at         IS 'レコード最終更新日時。';

-- 材料×仕入先の有効計画を高速検索するためのインデックス（IDX-026）
CREATE INDEX idx_sampling_plans_material_supplier ON sampling_plans (material_id, supplier_id) WHERE is_active = TRUE;
