-- V20260517120029__create_suspensions_table.sql
-- TBL-011 suspensions: 作業中断レコード（Append-only）。ADR-010 準拠で resumed_at 列なし。

-- EN-016 Suspension — 作業中断レコード（Append-only）。
-- ADR-010: resumed_at / resume_sign_id 列は存在しない。
-- 再開は work_events.activity='work_resumed' で記録する。
CREATE TABLE IF NOT EXISTS suspensions (
    -- 中断レコード識別子。UUID v4。gen_random_uuid() で自動生成。
    suspension_id           UUID        NOT NULL DEFAULT gen_random_uuid(),
    -- 中断対象の作業セッション識別子。
    work_execution_id       UUID        NOT NULL,
    -- 中断を行ったユーザーの識別子。
    suspended_by            UUID        NOT NULL,
    -- 中断理由カテゴリ。7 種の列挙値のみ許可（CHECK 制約）。
    suspend_reason_category VARCHAR(32) NOT NULL,
    -- 中断理由の自由記述。NULL 許容。
    suspend_comment         TEXT        NULL,
    -- 中断時の電子サイン識別子。NULL = サイン不要の中断。
    sign_id                 UUID        NULL,
    -- 中断時刻。
    suspended_at            TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    -- 中断時の current_step_index。再開時に work_executions.current_step_index に転写される。
    step_index_at_suspend   SMALLINT    NOT NULL,

    -- 主キー
    CONSTRAINT pk_suspensions PRIMARY KEY (suspension_id),
    -- work_executions への外部キー。
    CONSTRAINT fk_suspensions_execution FOREIGN KEY (work_execution_id)
        REFERENCES work_executions (work_execution_id) ON DELETE RESTRICT,
    -- 中断者ユーザーへの外部キー。
    CONSTRAINT fk_suspensions_suspended_by FOREIGN KEY (suspended_by)
        REFERENCES users (user_id) ON DELETE RESTRICT,
    -- 電子サインへの外部キー。NULL 許容。
    CONSTRAINT fk_suspensions_sign FOREIGN KEY (sign_id)
        REFERENCES electronic_signs (sign_id) ON DELETE RESTRICT,
    -- suspend_reason_category は 7 種の列挙値のみ許可する。
    CONSTRAINT ck_suspensions_reason CHECK (
        suspend_reason_category IN (
            'MATERIAL_WAIT',
            'EQUIPMENT_FAILURE',
            'QUALITY_HOLD',
            'WORKER_ABSENCE',
            'SAFETY_ISSUE',
            'PROCESS_CHANGE',
            'OTHER'
        )
    ),
    -- step_index_at_suspend は 0 以上の値のみ許可する。
    CONSTRAINT ck_suspensions_step_non_negative CHECK (step_index_at_suspend >= 0)
);

COMMENT ON TABLE  suspensions IS 'EN-016 Suspension — 作業中断レコード。Append-only。中断状態のみ記録する。再開は work_events.activity=''work_resumed'' イベントで記録する（ADR-010 準拠）。7年以上保存。';
COMMENT ON COLUMN suspensions.step_index_at_suspend IS '中断時の current_step_index。再開時に work_executions.current_step_index に転写される（アプリ層で制御）。';

-- Append-only 強制: app_event_writer ロールから UPDATE/DELETE を剥奪する
REVOKE UPDATE, DELETE ON suspensions FROM PUBLIC;
REVOKE UPDATE, DELETE ON suspensions FROM app_event_writer;
-- app_event_writer に INSERT と SELECT のみを許可する
GRANT INSERT, SELECT ON suspensions TO app_event_writer;
