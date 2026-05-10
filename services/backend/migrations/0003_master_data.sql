-- 対応 §: ロードマップ §10.2.1（マスタ編集）§10.3.6 RACI（基幹原典）§16
-- マスタデータと作業実行コンテキストの永続化スキーマ。
-- 製品／設備／部材の最小マスタ＋ tasks の current_step／responsible_user。

-- =====================================================================
-- products: 製品マスタ
-- =====================================================================
CREATE TABLE IF NOT EXISTS products (
    code         TEXT PRIMARY KEY,
    name         TEXT NOT NULL,
    industry     TEXT,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at   TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- =====================================================================
-- equipments: 設備マスタ
-- =====================================================================
CREATE TABLE IF NOT EXISTS equipments (
    code         TEXT PRIMARY KEY,
    name         TEXT NOT NULL,
    location     TEXT,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at   TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- =====================================================================
-- parts: 部材マスタ
-- =====================================================================
CREATE TABLE IF NOT EXISTS parts (
    code         TEXT PRIMARY KEY,
    name         TEXT NOT NULL,
    unit         TEXT,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at   TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- =====================================================================
-- tasks の拡張: 担当者・現在ステップ・進捗（既存テーブルに ALTER）
-- =====================================================================
ALTER TABLE tasks ADD COLUMN IF NOT EXISTS responsible_user TEXT;
ALTER TABLE tasks ADD COLUMN IF NOT EXISTS current_step_id TEXT;
ALTER TABLE tasks ADD COLUMN IF NOT EXISTS title TEXT;
ALTER TABLE tasks ADD COLUMN IF NOT EXISTS flow_id TEXT;

-- =====================================================================
-- task_steps: タスクの経路ステップ（端末がバックエンドから取得する）
-- =====================================================================
CREATE TABLE IF NOT EXISTS task_steps (
    id                   TEXT NOT NULL,
    task_id              TEXT NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    sequence             INT NOT NULL,
    label                TEXT NOT NULL,
    completion_criteria  TEXT NOT NULL CHECK (completion_criteria IN ('manual', 'photo')),
    standard_time_seconds INT NOT NULL DEFAULT 60,
    done                 BOOLEAN NOT NULL DEFAULT FALSE,
    PRIMARY KEY (task_id, id)
);

CREATE INDEX IF NOT EXISTS idx_task_steps_seq ON task_steps (task_id, sequence);

-- =====================================================================
-- flows: フロー定義（ヘッダ）
-- =====================================================================
CREATE TABLE IF NOT EXISTS flows (
    id           TEXT NOT NULL,
    version      INT NOT NULL,
    name         TEXT NOT NULL,
    industry     TEXT,
    status       TEXT NOT NULL DEFAULT 'draft' CHECK (status IN ('draft', 'trial', 'production', 'archived')),
    body         TEXT NOT NULL,  -- JSON: nodes/edges
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (id, version)
);
