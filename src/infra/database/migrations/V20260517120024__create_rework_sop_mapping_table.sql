-- V20260517120024__create_rework_sop_mapping_table.sql
-- TBL-046 rework_sop_mapping: 不適合カテゴリ×リワーク種別→対象SOP のマッピングマスタ。

-- EN-035 ReworkSopMapping — 不適合カテゴリ×リワーク種別→対象SOP のマッピングマスタ
CREATE TABLE IF NOT EXISTS rework_sop_mapping (
    -- マッピング識別子。UUID v7（時系列順）。Rust 側で生成する。
    mapping_id                  UUID            NOT NULL DEFAULT gen_random_uuid(),
    -- 不適合カテゴリ（自由記述）。例: 寸法不良、外観傷、接着不良。
    nonconformity_category      VARCHAR(128)    NOT NULL,
    -- 元となる SOP の識別子（NULL は汎用マッピング）。
    source_sop_id               UUID            NULL,
    -- 元となるステップの識別子（NULL は SOP 全体に適用）。
    source_step_id              UUID            NULL,
    -- 適用するリワーク SOP の識別子。
    target_rework_sop_id        UUID            NOT NULL,
    -- リワーク種別。5 値のみ許可する。
    rework_type                 TEXT            NOT NULL,
    -- 有効フラグ。廃止時に FALSE に設定する。物理 DELETE は禁止。
    is_active                   BOOLEAN         NOT NULL DEFAULT TRUE,
    -- レコード作成日時。
    created_at                  TIMESTAMPTZ     NOT NULL DEFAULT NOW(),

    CONSTRAINT pk_rework_sop_mapping PRIMARY KEY (mapping_id),
    -- target_rework_sop_id → sops テーブルへの外部キー。
    CONSTRAINT fk_rework_sop_mapping_target_sop FOREIGN KEY (target_rework_sop_id)
        REFERENCES sops (sop_id) ON DELETE RESTRICT,
    -- source_sop_id → sops テーブルへの外部キー（NULL 許容）。
    CONSTRAINT fk_rework_sop_mapping_source_sop FOREIGN KEY (source_sop_id)
        REFERENCES sops (sop_id) ON DELETE RESTRICT,
    -- source_step_id → steps テーブルへの外部キー（NULL 許容）。
    CONSTRAINT fk_rework_sop_mapping_source_step FOREIGN KEY (source_step_id)
        REFERENCES steps (step_id) ON DELETE RESTRICT,
    -- rework_type は 5 値のみ許可する。
    CONSTRAINT ck_rework_sop_mapping_type CHECK (
        rework_type IN ('TOUCH_UP', 'REWORK_FULL', 'SORTING', 'SCRAP', 'RETURN')
    )
);

COMMENT ON TABLE  rework_sop_mapping IS 'EN-035 ReworkSopMapping — 不適合カテゴリ×リワーク種別から適用 SOP を決定するマッピングマスタ。';
COMMENT ON COLUMN rework_sop_mapping.mapping_id             IS 'UUID v7（時系列順）。Rust 側で生成する。';
COMMENT ON COLUMN rework_sop_mapping.nonconformity_category IS '不適合カテゴリ（自由記述）。例: 寸法不良、外観傷、接着不良。';
COMMENT ON COLUMN rework_sop_mapping.source_sop_id          IS '元となる SOP の識別子。NULL は汎用マッピング（全 SOP に適用）。';
COMMENT ON COLUMN rework_sop_mapping.source_step_id         IS '元となるステップの識別子。NULL は SOP 全体に適用。';
COMMENT ON COLUMN rework_sop_mapping.target_rework_sop_id   IS '適用するリワーク SOP の識別子。';
COMMENT ON COLUMN rework_sop_mapping.rework_type            IS 'TOUCH_UP / REWORK_FULL / SORTING / SCRAP / RETURN の 5 値。';
COMMENT ON COLUMN rework_sop_mapping.is_active              IS '廃止時に FALSE に設定する。物理 DELETE は禁止。';
COMMENT ON COLUMN rework_sop_mapping.created_at             IS 'レコード作成日時。';
