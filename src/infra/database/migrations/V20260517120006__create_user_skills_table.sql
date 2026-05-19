-- V20260517120006__create_user_skills_table.sql
-- TBL-020 user_skills: EN-001 × EN-003 N:M 中間テーブル。ユーザーへのスキル認定記録。

-- EN-001 × EN-003 N:M 中間テーブル — ユーザーへのスキル認定記録
CREATE TABLE IF NOT EXISTS user_skills (
    -- スキルを認定されたユーザーの識別子。
    user_id         UUID        NOT NULL,
    -- 認定されたスキルの識別子。
    skill_id        UUID        NOT NULL,
    -- この認定時点での達成レベル。スキルゲートは steps.skill_level_required と比較する。
    achieved_level  SMALLINT    NOT NULL,
    -- スキル認定日時。
    certified_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    -- 認定者の user_id。supervisor 以上が操作可能（アプリ層で制御）。
    certified_by    UUID        NOT NULL,

    -- 複合主キー。1 ユーザーに同一スキルは 1 レコードのみ許可する（再認定は UPDATE）。
    CONSTRAINT pk_user_skills PRIMARY KEY (user_id, skill_id),
    -- ユーザー参照外部キー。
    CONSTRAINT fk_user_skills_user FOREIGN KEY (user_id)
        REFERENCES users (user_id) ON DELETE RESTRICT,
    -- スキル参照外部キー。
    CONSTRAINT fk_user_skills_skill FOREIGN KEY (skill_id)
        REFERENCES skills (skill_id) ON DELETE RESTRICT,
    -- 認定者参照外部キー。
    CONSTRAINT fk_user_skills_certified_by FOREIGN KEY (certified_by)
        REFERENCES users (user_id) ON DELETE RESTRICT,
    -- achieved_level は 1〜5 の範囲のみ許可する。
    CONSTRAINT ck_user_skills_level CHECK (achieved_level BETWEEN 1 AND 5)
);

COMMENT ON TABLE  user_skills IS 'EN-001×EN-003 N:M 中間テーブル。スキルゲート（BR-BUS-015）の判定源。';
COMMENT ON COLUMN user_skills.user_id        IS 'スキルを認定されたユーザーの user_id。';
COMMENT ON COLUMN user_skills.skill_id       IS '認定されたスキルの skill_id。';
COMMENT ON COLUMN user_skills.achieved_level IS '1〜5。この認定時点での達成レベル。スキルゲートは steps.skill_level_required と比較する。';
COMMENT ON COLUMN user_skills.certified_at   IS 'スキル認定日時。';
COMMENT ON COLUMN user_skills.certified_by   IS '認定者の user_id。supervisor 以上が操作可能（アプリ層で制御）。';
