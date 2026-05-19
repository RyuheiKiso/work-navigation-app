-- V20260517120018__create_steps_table.sql
-- TBL-008 steps: SOP を構成する作業ステップマスタ（版管理）。PUBLISHED 後は内容変更禁止。

-- EN-009 Step — SOP を構成する作業ステップマスタ（版管理）
CREATE TABLE IF NOT EXISTS steps (
    -- ステップ識別子。UUID v7（時系列順）。Rust 側で生成する。
    step_id                UUID        NOT NULL DEFAULT gen_random_uuid(),
    -- 所属する SOP の識別子。
    sop_id                 UUID        NOT NULL,
    -- SOP 内でのステップ番号。1 以上の正整数。
    step_number            SMALLINT    NOT NULL,
    -- 入力種別。9 値のみ許可する。
    input_type             VARCHAR(32) NOT NULL,
    -- 多言語作業指示 JSONB。{"ja": "...", "en": "...", "ja-simple": "..."} 形式。ja キー必須。
    instruction_text       JSONB       NOT NULL,
    -- 合否判定条件 JSONB。{"usl": 10.5, "lsl": 9.5, "unit": "mm", "tolerance": 0.5} 形式。numeric_input 時は必須。
    judgment_condition     JSONB       NULL,
    -- TRUE のとき、このステップ完了時に evidence_files の登録がゲート条件となる（BR-BUS-003）。
    evidence_required      BOOLEAN     NOT NULL DEFAULT FALSE,
    -- FMEA の RPN 高リスク項目フラグ。TRUE のとき強調表示・監督承認が必要（アプリ層で制御）。
    fmea_rpn_flag          BOOLEAN     NOT NULL DEFAULT FALSE,
    -- 1〜5。作業員の user_skills.achieved_level >= この値でなければ作業開始不可（FR-NV-008）。
    skill_level_required   SMALLINT    NOT NULL DEFAULT 1,
    -- UCUM コード。例: mm, celsius, kPa。numeric_input 時の表示単位。
    expected_unit          VARCHAR(32) NULL,
    -- 参照メディア JSONB。[{"type": "VIDEO", "url": "...", "title": {"ja": "..."}}] 形式の配列。
    media_refs             JSONB       NULL,
    -- コツ・注意事項 JSONB。[{"text": {"ja": "...", "en": "..."}, "severity": "WARNING"}] 形式の配列。
    tips_refs              JSONB       NULL,
    -- レコード作成日時。
    created_at             TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    -- レコード最終更新日時。
    updated_at             TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT pk_steps PRIMARY KEY (step_id),
    -- sops テーブルへの外部キー。SOP 削除時は RESTRICT。
    CONSTRAINT fk_steps_sop FOREIGN KEY (sop_id)
        REFERENCES sops (sop_id) ON DELETE RESTRICT,
    -- input_type は 9 値のみ許可する。
    CONSTRAINT ck_steps_input_type CHECK (
        input_type IN (
            'boolean_check', 'numeric_input', 'photo_capture',
            'text_input', 'slider_range', 'multi_select',
            'signature', 'barcode_scan', 'nfc_read'
        )
    ),
    -- skill_level_required は 1〜5 の範囲のみ許可する。
    CONSTRAINT ck_steps_skill_level CHECK (skill_level_required BETWEEN 1 AND 5),
    -- step_number は 1 以上の正整数のみ許可する。
    CONSTRAINT ck_steps_step_number_positive CHECK (step_number > 0),
    -- instruction_text の ja キーは必須かつ空文字禁止とする。
    CONSTRAINT ck_steps_instruction_has_ja CHECK (
        jsonb_typeof(instruction_text -> 'ja') = 'string'
        AND length(instruction_text ->> 'ja') > 0
    ),
    -- numeric_input の場合は judgment_condition が必須とする。
    CONSTRAINT ck_steps_numeric_requires_condition CHECK (
        NOT (input_type = 'numeric_input' AND judgment_condition IS NULL)
    ),
    -- 同一 SOP 内でのステップ番号は一意とする。
    CONSTRAINT uq_steps_sop_step_number UNIQUE (sop_id, step_number)
);

COMMENT ON TABLE  steps IS 'EN-009 Step — SOP ステップマスタ。PUBLISHED 後は内容変更禁止（DB トリガで強制）。';
COMMENT ON COLUMN steps.step_id              IS 'UUID v7（時系列順）。Rust 側で生成する。';
COMMENT ON COLUMN steps.sop_id              IS '所属する SOP の sop_id。';
COMMENT ON COLUMN steps.step_number         IS 'SOP 内でのステップ番号。1 以上の正整数。';
COMMENT ON COLUMN steps.input_type          IS 'boolean_check / numeric_input / photo_capture / text_input / slider_range / multi_select / signature / barcode_scan / nfc_read の 9 値。';
COMMENT ON COLUMN steps.instruction_text    IS '多言語作業指示 JSONB。{"ja": "...", "en": "...", "ja-simple": "..."} 形式。ja キー必須。';
COMMENT ON COLUMN steps.judgment_condition  IS '合否判定条件 JSONB。{"usl": 10.5, "lsl": 9.5, "unit": "mm", "tolerance": 0.5} 形式。numeric_input 時は必須。';
COMMENT ON COLUMN steps.evidence_required   IS 'TRUE のとき、このステップ完了時に evidence_files の登録がゲート条件となる（BR-BUS-003）。';
COMMENT ON COLUMN steps.fmea_rpn_flag       IS 'FMEA の RPN 高リスク項目フラグ。TRUE のとき強調表示・監督承認が必要（アプリ層で制御）。';
COMMENT ON COLUMN steps.skill_level_required IS '1〜5。作業員の user_skills.achieved_level >= この値でなければ作業開始不可（FR-NV-008）。';
COMMENT ON COLUMN steps.expected_unit       IS 'UCUM コード。例: mm, celsius, kPa。numeric_input 時の表示単位。';
COMMENT ON COLUMN steps.media_refs          IS '参照メディア JSONB。[{"type": "VIDEO", "url": "...", "title": {"ja": "..."}}] 形式の配列。';
COMMENT ON COLUMN steps.tips_refs           IS 'コツ・注意事項 JSONB。[{"text": {"ja": "...", "en": "..."}, "severity": "WARNING"}] 形式の配列。';
COMMENT ON COLUMN steps.created_at          IS 'レコード作成日時。';
COMMENT ON COLUMN steps.updated_at          IS 'レコード最終更新日時。';
