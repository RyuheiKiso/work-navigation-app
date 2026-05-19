-- V20260517120013__create_equipments_table.sql
-- TBL-025 equipments: 生産設備マスタ。scan_code / tool_subtype / calibration_due_date を含む拡張版。

-- EN-019 Equipment — 生産設備マスタ
CREATE TABLE IF NOT EXISTS equipments (
    -- 設備識別子。UUID v7（時系列順）。Rust 側で生成する。
    equipment_id         UUID        NOT NULL DEFAULT gen_random_uuid(),
    -- 設備コード。変更不可の公開識別子。
    equipment_code       VARCHAR(64) NOT NULL,
    -- 多言語名称 JSONB。{"ja": "設備名", "en": "Equipment Name"} 形式。ja キーは必須。
    name                 JSONB       NOT NULL,
    -- 設備種別の自由記述文字列（例: INJECTION_MOLD, ASSEMBLY_JIG, CONVEYOR）。コード体系は外部定義。
    equipment_type       VARCHAR(64) NOT NULL,
    -- 主に使用するプロセスの process_id。NULL は汎用設備。
    process_id           UUID        NULL,
    -- スキャン照合用 ID（GS1 EID/AI 8004 互換）。NULL は照合対象外。FR-EV-013。
    scan_code            VARCHAR(64) NULL,
    -- 工具・治具のサブ種別（例: TORQUE_WRENCH, FIXTURE_JIG）。NULL は設備（生産機械等）。
    tool_subtype         VARCHAR(64) NULL,
    -- 治具点検期限（NULL は点検不要）。BR-BUS-007 のハードブロック対象範囲を計測器から治具に拡張。
    calibration_due_date DATE        NULL,
    -- 有効フラグ。廃止時に FALSE に設定する。物理 DELETE は禁止。
    is_active            BOOLEAN     NOT NULL DEFAULT TRUE,
    -- レコード作成日時。
    created_at           TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    -- レコード最終更新日時。
    updated_at           TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT pk_equipments PRIMARY KEY (equipment_id),
    CONSTRAINT uq_equipments_code UNIQUE (equipment_code),
    -- scan_code は UNIQUE（NULL は複数許可）。
    CONSTRAINT uq_equipments_scan_code UNIQUE (scan_code),
    -- processes テーブルへの外部キー。プロセス削除時は RESTRICT。
    CONSTRAINT fk_equipments_process FOREIGN KEY (process_id)
        REFERENCES processes (process_id) ON DELETE RESTRICT,
    -- name の ja キーは必須かつ空文字禁止とする。
    CONSTRAINT ck_equipments_name_has_ja CHECK (
        jsonb_typeof(name -> 'ja') = 'string'
        AND length(name ->> 'ja') > 0
    ),
    -- tool_subtype は 7 値または NULL のみ許可する。
    CONSTRAINT ck_equipments_jig_subtype CHECK (
        tool_subtype IS NULL OR tool_subtype IN (
            'TORQUE_WRENCH', 'FIXTURE_JIG', 'DRILL_GUIDE',
            'TEMPLATE', 'GAUGE', 'ASSEMBLY_JIG', 'OTHER'
        )
    )
);

COMMENT ON TABLE  equipments IS 'EN-019 Equipment — 生産設備マスタ。アンドン発報（TBL-012）の equipment_type 参照元。';
COMMENT ON COLUMN equipments.equipment_id         IS 'UUID v7（時系列順）。Rust 側で生成する。';
COMMENT ON COLUMN equipments.equipment_code       IS '設備コード。変更不可の公開識別子。';
COMMENT ON COLUMN equipments.name                 IS '多言語名称 JSONB。{"ja": "設備名", "en": "Equipment Name"} 形式。ja キーは必須。';
COMMENT ON COLUMN equipments.equipment_type       IS '設備種別の自由記述文字列（例: INJECTION_MOLD, ASSEMBLY_JIG, CONVEYOR）。コード体系は外部定義。';
COMMENT ON COLUMN equipments.process_id           IS '主に使用するプロセスの process_id。NULL は汎用設備。';
COMMENT ON COLUMN equipments.scan_code            IS 'スキャン照合用 ID（GS1 EID/AI 8004 互換）。NULL は照合対象外。FR-EV-013。';
COMMENT ON COLUMN equipments.tool_subtype         IS '工具・治具のサブ種別（例: TORQUE_WRENCH, FIXTURE_JIG）。NULL は設備（生産機械等）。';
COMMENT ON COLUMN equipments.calibration_due_date IS '治具点検期限（NULL は点検不要）。BR-BUS-007 のハードブロック対象範囲を計測器から治具に拡張。';
COMMENT ON COLUMN equipments.is_active            IS '廃止時に FALSE に設定する。物理 DELETE は禁止。';
COMMENT ON COLUMN equipments.created_at           IS 'レコード作成日時。';
COMMENT ON COLUMN equipments.updated_at           IS 'レコード最終更新日時。';
