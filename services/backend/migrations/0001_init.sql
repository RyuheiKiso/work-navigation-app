-- 対応 §: ロードマップ §10.6 §10.6.1 §11.4.1 §15
-- 作業ナビアプリの初期スキーマ。
-- tasks: Aggregate ルート相当。状態遷移とドメイン値の射影。
-- records: G-Set 相当（追記のみ、§10.6.1）。
-- audit_log: 追記不変ストア（§11.4.1 INV-07 の前提）。

-- 拡張: gen_random_uuid を使うため pgcrypto を有効化する。
CREATE EXTENSION IF NOT EXISTS pgcrypto;

-- =====================================================================
-- tasks: 「作業（Task）」Aggregate
-- =====================================================================
CREATE TABLE IF NOT EXISTS tasks (
    -- ドメインの TaskId と一致する文字列キー
    id                   TEXT PRIMARY KEY,
    -- HSM 状態ラベル（§3.4.1）
    state                TEXT NOT NULL CHECK (state IN (
        'Idle', 'Ready', 'Running', 'Suspended', 'Exception',
        'Completed', 'Failed', 'Aborted'
    )),
    -- 主体端末（§10.6.1 DeviceId）
    device_id            TEXT NOT NULL,
    -- Lamport タイムスタンプ（§10.6.1 INV-08）
    lamport              BIGINT NOT NULL CHECK (lamport >= 0),
    -- 完了条件タグ（manual / photo の文字列）
    completion_criteria  TEXT NOT NULL,
    -- 作成・更新時刻（UTC、§20.2）
    created_at           TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at           TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 端末別の参照を高速化する補助索引
CREATE INDEX IF NOT EXISTS idx_tasks_device_id ON tasks (device_id);

-- =====================================================================
-- records: 作業実績の追記専用テーブル（G-Set、§10.6.1）
-- =====================================================================
CREATE TABLE IF NOT EXISTS records (
    -- レコード ID（DB 自動採番、UUID v4 で十分）
    id                   UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    -- 対象 Task
    task_id              TEXT NOT NULL REFERENCES tasks(id),
    -- 発生端末
    device_id            TEXT NOT NULL,
    -- Lamport タイムスタンプ
    lamport              BIGINT NOT NULL CHECK (lamport >= 0),
    -- payload（JSON 文字列、後段で jsonb 化を検討）
    payload              TEXT NOT NULL,
    -- 受領時刻
    received_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 同期ストリーム再開用の索引（§10.6.1 再同期境界）
CREATE INDEX IF NOT EXISTS idx_records_task_lamport
    ON records (task_id, lamport);

-- =====================================================================
-- audit_log: 操作監査ログ（追記不変、§11.4.1 INV-07）
-- =====================================================================
CREATE TABLE IF NOT EXISTS audit_log (
    -- ログ ID
    id                   UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    -- 主体 ID（端末／ユーザー）
    actor_id             TEXT NOT NULL,
    -- 操作種別（"start_task", "complete_task" 等）
    action               TEXT NOT NULL,
    -- 対象識別子
    target_id            TEXT,
    -- 端末時刻（§20.2 端末時刻とサーバ時刻の両記録）
    terminal_time        TIMESTAMPTZ,
    -- サーバ時刻（受領時）
    server_time          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    -- ペイロード（JSON 文字列）
    payload              TEXT
);

-- 監査検索用の索引
CREATE INDEX IF NOT EXISTS idx_audit_actor ON audit_log (actor_id);
CREATE INDEX IF NOT EXISTS idx_audit_action ON audit_log (action);
CREATE INDEX IF NOT EXISTS idx_audit_server_time ON audit_log (server_time);

-- =====================================================================
-- 改ざん抑止トリガ（§11.4.1 INV-07 の運用前提）
-- audit_log への UPDATE / DELETE を物理的に禁止する。
-- =====================================================================
CREATE OR REPLACE FUNCTION audit_log_no_mutation() RETURNS trigger AS $$
BEGIN
    -- 更新・削除を例外で拒否する
    RAISE EXCEPTION 'audit_log は追記専用です（§11.4.1 INV-07）';
END;
$$ LANGUAGE plpgsql;

-- UPDATE トリガ
DROP TRIGGER IF EXISTS audit_log_block_update ON audit_log;
CREATE TRIGGER audit_log_block_update
    BEFORE UPDATE ON audit_log
    FOR EACH ROW EXECUTE FUNCTION audit_log_no_mutation();

-- DELETE トリガ
DROP TRIGGER IF EXISTS audit_log_block_delete ON audit_log;
CREATE TRIGGER audit_log_block_delete
    BEFORE DELETE ON audit_log
    FOR EACH ROW EXECUTE FUNCTION audit_log_no_mutation();
