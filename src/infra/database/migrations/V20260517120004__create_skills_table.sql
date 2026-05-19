-- V20260517120004__create_skills_table.sql
-- TBL-018 skills: 作業員スキル定義マスタ。skill_level は定義上の最高レベルを示す。

-- EN-003 Skill — 作業員スキル定義マスタ
CREATE TABLE IF NOT EXISTS skills (
    -- スキル識別子。UUID v7（時系列順）。Rust 側で生成する。
    skill_id      UUID          NOT NULL DEFAULT gen_random_uuid(),
    -- スキルコード。形式: {カテゴリ英字}-{連番3桁}。例: WLD-001（溶接技能）。変更不可の公開識別子。
    skill_code    VARCHAR(64)   NOT NULL,
    -- スキル名称。
    skill_name    VARCHAR(128)  NOT NULL,
    -- このスキル定義が想定する上限レベル。1〜5 の範囲で指定する。
    skill_level   SMALLINT      NOT NULL DEFAULT 1,
    -- スキルの説明文。
    description   TEXT          NOT NULL DEFAULT '',
    -- 有効フラグ。廃止時に FALSE に設定する。物理 DELETE は禁止。
    is_active     BOOLEAN       NOT NULL DEFAULT TRUE,
    -- レコード作成日時。
    created_at    TIMESTAMPTZ   NOT NULL DEFAULT NOW(),
    -- レコード最終更新日時。
    updated_at    TIMESTAMPTZ   NOT NULL DEFAULT NOW(),

    CONSTRAINT pk_skills PRIMARY KEY (skill_id),
    CONSTRAINT uq_skills_code UNIQUE (skill_code),
    -- skill_level は 1〜5 の範囲のみ許可する。
    CONSTRAINT ck_skills_level CHECK (skill_level BETWEEN 1 AND 5)
);

COMMENT ON TABLE  skills IS 'EN-003 Skill — 作業員スキル定義マスタ。skill_level は定義上の最高レベルを示す。';
COMMENT ON COLUMN skills.skill_id    IS 'UUID v7（時系列順）。Rust 側で生成する。';
COMMENT ON COLUMN skills.skill_code  IS 'スキルコード。形式: {カテゴリ英字}-{連番3桁}。例: WLD-001（溶接技能）。変更不可の公開識別子。';
COMMENT ON COLUMN skills.skill_name  IS 'スキル名称。';
COMMENT ON COLUMN skills.skill_level IS '1〜5。このスキル定義が想定する上限レベル。';
COMMENT ON COLUMN skills.description IS 'スキルの説明文。';
COMMENT ON COLUMN skills.is_active   IS '廃止時に FALSE に設定する。物理 DELETE は禁止。';
COMMENT ON COLUMN skills.created_at  IS 'レコード作成日時。';
COMMENT ON COLUMN skills.updated_at  IS 'レコード最終更新日時。';
