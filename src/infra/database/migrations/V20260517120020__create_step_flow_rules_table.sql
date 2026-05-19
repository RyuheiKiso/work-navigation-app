-- V20260517120020__create_step_flow_rules_table.sql
-- TBL-030 step_flow_rules: JSON Logic ベースの条件分岐ルール定義マスタ（版管理）。PUBLISHED 後は内容変更禁止。

-- EN-027 StepFlowRule — JSON Logic ベースの条件分岐ルール定義マスタ（版管理）
CREATE TABLE IF NOT EXISTS step_flow_rules (
    -- フロールール識別子。UUID v7（時系列順）。Rust 側で生成する。
    rule_id          UUID        NOT NULL DEFAULT gen_random_uuid(),
    -- このルールが属する SOP の識別子。
    sop_id           UUID        NOT NULL,
    -- 遷移元ステップの識別子。
    from_step_id     UUID        NOT NULL,
    -- 遷移先ステップの識別子。
    to_step_id       UUID        NOT NULL,
    -- 同一 from_step_id に複数ルールが存在する場合の評価優先度。小さい値が優先。
    rule_priority    SMALLINT    NOT NULL DEFAULT 0,
    -- JSON Logic 形式の条件式 JSONB。詳細スキーマは 05_JSONBスキーマ定義.md §1 参照。
    rule_definition  JSONB       NOT NULL,
    -- 有効フラグ。廃止時に FALSE に設定する。物理 DELETE は禁止。
    is_active        BOOLEAN     NOT NULL DEFAULT TRUE,
    -- レコード作成日時。
    created_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    -- レコード最終更新日時。
    updated_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT pk_step_flow_rules PRIMARY KEY (rule_id),
    -- sops テーブルへの外部キー。SOP 削除時は RESTRICT。
    CONSTRAINT fk_step_flow_rules_sop FOREIGN KEY (sop_id)
        REFERENCES sops (sop_id) ON DELETE RESTRICT,
    -- 遷移元ステップへの外部キー。ステップ削除時は RESTRICT。
    CONSTRAINT fk_step_flow_rules_from FOREIGN KEY (from_step_id)
        REFERENCES steps (step_id) ON DELETE RESTRICT,
    -- 遷移先ステップへの外部キー。ステップ削除時は RESTRICT。
    CONSTRAINT fk_step_flow_rules_to FOREIGN KEY (to_step_id)
        REFERENCES steps (step_id) ON DELETE RESTRICT,
    -- 自己ループ（from_step_id = to_step_id）は禁止する。
    CONSTRAINT ck_step_flow_rules_no_self_loop CHECK (from_step_id <> to_step_id),
    -- rule_priority は 0 以上の非負整数のみ許可する。
    CONSTRAINT ck_step_flow_rules_priority_non_negative CHECK (rule_priority >= 0),
    -- rule_definition は JSONB オブジェクト型のみ許可する。
    CONSTRAINT ck_step_flow_rules_definition_is_object CHECK (
        jsonb_typeof(rule_definition) = 'object'
    )
);

COMMENT ON TABLE  step_flow_rules IS 'EN-027 StepFlowRule — JSON Logic 条件分岐ルール。StepEngine が条件評価し次ステップを決定する。PUBLISHED 後は内容変更禁止。';
COMMENT ON COLUMN step_flow_rules.rule_id         IS 'UUID v7（時系列順）。Rust 側で生成する。';
COMMENT ON COLUMN step_flow_rules.sop_id          IS 'このルールが属する SOP の sop_id。';
COMMENT ON COLUMN step_flow_rules.from_step_id    IS '遷移元ステップの step_id。';
COMMENT ON COLUMN step_flow_rules.to_step_id      IS '遷移先ステップの step_id。';
COMMENT ON COLUMN step_flow_rules.rule_priority   IS '同一 from_step_id に複数ルールが存在する場合の評価優先度。小さい値が優先。';
COMMENT ON COLUMN step_flow_rules.rule_definition IS 'JSON Logic 形式の条件式 JSONB。詳細スキーマは 05_JSONBスキーマ定義.md §1 参照。';
COMMENT ON COLUMN step_flow_rules.is_active       IS '廃止時に FALSE に設定する。物理 DELETE は禁止。';
COMMENT ON COLUMN step_flow_rules.created_at      IS 'レコード作成日時。';
COMMENT ON COLUMN step_flow_rules.updated_at      IS 'レコード最終更新日時。';
