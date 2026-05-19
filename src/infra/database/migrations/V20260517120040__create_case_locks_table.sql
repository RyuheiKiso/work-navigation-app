-- V20260517120040__create_case_locks_table.sql
-- TBL-051 case_locks: Case 端末排他占有テーブル（FR-SY-011 / ADR-009）。app_event_insert に INSERT/UPDATE/DELETE を許可する例外テーブル。

-- TBL-051 case_locks — Case 端末排他占有テーブル（FR-SY-011 / ADR-009）。
-- 制御テーブル: app_event_insert ロールに INSERT/UPDATE/DELETE を許可する（idempotency_keys と同様の例外）。
-- 1 case_id に同時 1 端末のみ ACTIVE 可。BAT-013 が heartbeat_at 5分超過で EXPIRED 化する。
CREATE TABLE IF NOT EXISTS case_locks (
    -- ロック対象の作業セッション識別子。PRIMARY KEY（1 case_id に 1 件のロック）。
    case_id             UUID        NOT NULL,
    -- ロックを取得した端末の識別子。
    terminal_id         UUID        NOT NULL,
    -- ロックを取得したユーザーの識別子。
    user_id             UUID        NOT NULL,
    -- ロック取得時刻。
    acquired_at         TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    -- 最終ハートビート時刻。BAT-013 が 5 分超過で EXPIRED 化する。
    heartbeat_at        TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    -- ロックステータス。ACTIVE / RELEASED / EXPIRED の 3 種のみ許可（初期値: ACTIVE）。
    lock_status         VARCHAR(16) NOT NULL DEFAULT 'ACTIVE',

    -- 主キー（case_id 単体が PK、1 case_id に 1 件のロックを保証する）
    CONSTRAINT pk_case_locks PRIMARY KEY (case_id),
    -- work_executions への外部キー。作業セッション削除時はカスケード削除する。
    CONSTRAINT fk_case_locks_case FOREIGN KEY (case_id)
        REFERENCES work_executions (work_execution_id) ON DELETE CASCADE,
    -- 端末への外部キー。
    CONSTRAINT fk_case_locks_terminal FOREIGN KEY (terminal_id)
        REFERENCES devices (device_id) ON DELETE RESTRICT,
    -- ユーザーへの外部キー。
    CONSTRAINT fk_case_locks_user FOREIGN KEY (user_id)
        REFERENCES users (user_id) ON DELETE RESTRICT,
    -- lock_status は 3 種の列挙値のみ許可する。
    CONSTRAINT ck_case_locks_status CHECK (
        lock_status IN ('ACTIVE', 'RELEASED', 'EXPIRED')
    )
);

COMMENT ON TABLE case_locks IS 'TBL-051 — case_id 単位の端末排他占有テーブル。1 case_id に同時 1 端末のみ ACTIVE 可。BAT-013 が heartbeat_at 5min 超過で EXPIRED 化。制御テーブルのため app_event_insert ロールに INSERT/UPDATE/DELETE を許可（例外）。';

-- 例外テーブルのため app_event_insert ロールに INSERT/UPDATE/DELETE を許可する
GRANT INSERT, UPDATE, DELETE ON case_locks TO app_event_insert;
