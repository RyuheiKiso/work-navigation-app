-- V20260517120066__create_batch_job_logs_table.sql
-- batch_job_logs: 全バッチジョブ共通の実行ログテーブル。
-- 権威: docs/05_詳細設計/07_アルゴリズム詳細設計/06_バッチジョブ処理詳細（BAT-001〜010）.md §12-1
-- BAT-001〜010（wnav_master_api / wnav_terminal_api 内 tokio task）が INSERT + UPDATE する。
-- status: RUNNING（開始時）→ SUCCEEDED / FAILED / SKIPPED（完了時）

CREATE TABLE IF NOT EXISTS batch_job_logs (
    -- バッチ実行ログ識別子。UUID v4。gen_random_uuid() で自動生成。
    id            UUID         NOT NULL DEFAULT gen_random_uuid(),
    -- バッチ識別子。'BAT-001'〜'BAT-010' のいずれか。
    bat_id        TEXT         NOT NULL,
    -- バッチ開始日時。
    started_at    TIMESTAMPTZ  NOT NULL,
    -- バッチ完了日時。実行中（RUNNING）は NULL。
    finished_at   TIMESTAMPTZ  NULL,
    -- 実行ステータス。RUNNING（実行中）/ SUCCEEDED（成功）/ FAILED（失敗）/ SKIPPED（スキップ）。
    status        TEXT         NOT NULL,
    -- エラーメッセージ。FAILED 時に設定する。
    error_message TEXT         NULL,
    -- バッチ固有のメタデータ（件数・対象期間等）を JSONB で格納する。
    metadata      JSONB        NULL,
    -- レコード作成日時。
    created_at    TIMESTAMPTZ  NOT NULL DEFAULT NOW(),

    CONSTRAINT pk_batch_job_logs PRIMARY KEY (id),
    -- status は 4 値のみ許可する。
    CONSTRAINT ck_batch_job_logs_status CHECK (
        status IN ('RUNNING', 'SUCCEEDED', 'FAILED', 'SKIPPED')
    ),
    -- bat_id は 'BAT-' で始まる形式を強制する。
    CONSTRAINT ck_batch_job_logs_bat_id CHECK (bat_id LIKE 'BAT-%'),
    -- finished_at は started_at 以降でなければならない。
    CONSTRAINT ck_batch_job_logs_finished_after_started CHECK (
        finished_at IS NULL OR finished_at >= started_at
    )
);

COMMENT ON TABLE batch_job_logs IS
    '全バッチジョブ共通実行ログ（BAT-001〜010）。RUNNING 開始 → SUCCEEDED/FAILED/SKIPPED 完了の状態遷移を記録する。';
COMMENT ON COLUMN batch_job_logs.bat_id    IS 'バッチ識別子。BAT-001〜BAT-010 のいずれか。';
COMMENT ON COLUMN batch_job_logs.status    IS 'RUNNING（実行中）/ SUCCEEDED（成功）/ FAILED（失敗）/ SKIPPED（スキップ）。';
COMMENT ON COLUMN batch_job_logs.metadata  IS 'バッチ固有メタデータ（例: verified_count / target_period 等）。';

-- bat_id + started_at の複合インデックス（バッチ別の実行履歴検索に使用する）
CREATE INDEX idx_batch_job_logs_bat_id_started_at
    ON batch_job_logs USING BTREE (bat_id, started_at DESC);

-- app_read_write に INSERT/SELECT/UPDATE を許可する（RUNNING→SUCCEEDED/FAILED への状態遷移に必要）
GRANT INSERT, SELECT, UPDATE ON batch_job_logs TO app_read_write;
