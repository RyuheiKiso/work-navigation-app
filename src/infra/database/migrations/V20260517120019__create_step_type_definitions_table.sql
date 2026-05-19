-- V20260517120019__create_step_type_definitions_table.sql
-- TBL-029 step_type_definitions: ステップ入力型の詳細定義マスタ（版管理）。steps.input_type と 1:1 対応。

-- EN-026 StepTypeDefinition — ステップ入力型の詳細定義マスタ（版管理）
CREATE TABLE IF NOT EXISTS step_type_definitions (
    -- ステップ型定義識別子。UUID v7（時系列順）。Rust 側で生成する。
    step_type_def_id   UUID        NOT NULL DEFAULT gen_random_uuid(),
    -- steps.input_type と一致する 9 値。UNIQUE 制約により 1 対 1 対応を保証する。
    type_code          VARCHAR(32) NOT NULL,
    -- 多言語表示名 JSONB。{"ja": "真偽値チェック", "en": "Boolean Check"} 形式。ja キーは必須。
    display_name       JSONB       NOT NULL,
    -- このステップ型の payload JSON Schema。StepEngine がバリデーションに使用する。
    schema_definition  JSONB       NOT NULL,
    -- 有効フラグ。廃止時に FALSE に設定する。物理 DELETE は禁止。
    is_active          BOOLEAN     NOT NULL DEFAULT TRUE,
    -- レコード作成日時。
    created_at         TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    -- レコード最終更新日時。
    updated_at         TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT pk_step_type_definitions PRIMARY KEY (step_type_def_id),
    -- type_code は UNIQUE とする（1 型コードに 1 定義）。
    CONSTRAINT uq_step_type_definitions_code UNIQUE (type_code),
    -- type_code は 9 値のみ許可する（steps.input_type と一致する値）。
    CONSTRAINT ck_step_type_def_type_code CHECK (
        type_code IN (
            'boolean_check', 'numeric_input', 'photo_capture',
            'text_input', 'slider_range', 'multi_select',
            'signature', 'barcode_scan', 'nfc_read'
        )
    ),
    -- display_name の ja キーは必須かつ空文字禁止とする。
    CONSTRAINT ck_step_type_def_display_name_has_ja CHECK (
        jsonb_typeof(display_name -> 'ja') = 'string'
        AND length(display_name ->> 'ja') > 0
    ),
    -- schema_definition は JSONB オブジェクト型のみ許可する。
    CONSTRAINT ck_step_type_def_schema_is_object CHECK (
        jsonb_typeof(schema_definition) = 'object'
    )
);

COMMENT ON TABLE  step_type_definitions IS 'EN-026 StepTypeDefinition — ステップ入力型の JSON Schema 定義マスタ。steps.input_type と 1:1 対応。version 管理のため master_versions（TBL-004）を使用する。';
COMMENT ON COLUMN step_type_definitions.step_type_def_id  IS 'UUID v7（時系列順）。Rust 側で生成する。';
COMMENT ON COLUMN step_type_definitions.type_code         IS 'steps.input_type と一致する 9 値。';
COMMENT ON COLUMN step_type_definitions.display_name      IS '多言語表示名 JSONB。{"ja": "真偽値チェック", "en": "Boolean Check"} 形式。ja キーは必須。';
COMMENT ON COLUMN step_type_definitions.schema_definition IS 'このステップ型の payload JSON Schema。StepEngine がバリデーションに使用する。';
COMMENT ON COLUMN step_type_definitions.is_active         IS '廃止時に FALSE に設定する。物理 DELETE は禁止。';
COMMENT ON COLUMN step_type_definitions.created_at        IS 'レコード作成日時。';
COMMENT ON COLUMN step_type_definitions.updated_at        IS 'レコード最終更新日時。';
