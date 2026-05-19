-- V20260519130004__create_sqlite_work_cache.sql
--
-- 端末ローカルの作業実行キャッシュテーブル群 — Offline-First 対応
--
-- ペア・マイグレーション規則（ADR-006）:
--   PG 対応: work_executions (TBL-005)、work_events (TBL-001) 等がサーバー側権威
--   SQLite: 本ファイルの local_work_executions / local_work_steps / local_work_assignments
--           は端末ローカルキャッシュとして機能し、同期完了後はサーバーが権威となる
--
-- 設計意図（Offline-First 原則; src/CLAUDE.md §1）:
--   端末は「一次記録メモリ」として機能し、サーバーは「複製」として位置づける。
--   この非対称性は絶対に逆転させない。
--   synced_to_server フラグにより同期状態を管理し、
--   未同期レコードは local_outbox 経由でサーバーに送信する。
--
-- マルチデバイス排他原則（src/CLAUDE.md §2; ADR-009）:
--   1 case_id = 1 端末。サーバーの TBL-051 case_locks が物理的に排他占有を保証する。
--   端末ローカルキャッシュはサーバー排他制御の補完であり、代替ではない。

PRAGMA journal_mode=WAL;
PRAGMA foreign_keys=ON;

-- ============================================================
-- local_work_executions — ローカル作業実行セッション
--   PG 対応: work_executions (TBL-005; 更新可ミラー)
--   作業セッションの開始・中断・完了状態をローカルで管理する
-- ============================================================
CREATE TABLE IF NOT EXISTS local_work_executions (
    -- 作業実行 ID（UUID v7; アプリ側で生成）
    id                  TEXT PRIMARY KEY,
    -- 作業指示 ID（XES Case ID; マルチデバイス排他の単位）
    case_id             TEXT NOT NULL,
    -- 実行する SOP の ID（ミラー FK）
    sop_id              TEXT NOT NULL REFERENCES sops(id),
    -- 作業状態（'running' | 'suspended' | 'completed'）
    status              TEXT NOT NULL,
    -- 現在実行中のステップ ID（null = セッション完了またはステップ未選択）
    current_step_id     TEXT,
    -- 主担当作業者の ID（ミラー FK）
    primary_worker_id   TEXT NOT NULL REFERENCES users(id),
    -- 作業開始日時（ISO 8601 UTC; 端末記録時刻）
    started_at          TEXT NOT NULL,
    -- 最終更新日時（ステップ完了・中断の度に更新）
    last_updated_at     TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    -- サーバー同期済みフラグ（0=未同期, 1=同期済）
    -- 同期完了後はサーバーの work_executions が権威となる
    synced_to_server    INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_local_work_executions_case_id ON local_work_executions (case_id);
-- 未同期レコードを効率的に抽出するためのインデックス
CREATE INDEX IF NOT EXISTS idx_local_work_executions_sync ON local_work_executions (synced_to_server, last_updated_at);

-- ============================================================
-- local_work_steps — ローカル作業ステップ実行
--   PG 対応: work_events (TBL-001; Append-only) の派生ビューに相当
--   各ステップの実行状態をキャッシュし、Offline-First 表示に使用する
-- ============================================================
CREATE TABLE IF NOT EXISTS local_work_steps (
    -- ステップ実行レコード ID（UUID v7; アプリ側で生成）
    id                  TEXT PRIMARY KEY,
    -- 所属する作業実行セッション ID（FK）
    work_execution_id   TEXT NOT NULL REFERENCES local_work_executions(id),
    -- 対象ステップの ID（ミラー FK）
    step_id             TEXT NOT NULL REFERENCES steps(id),
    -- ステップ実行状態（'pending' | 'in_progress' | 'completed' | 'skipped'）
    -- スキップも記録する（ALCOA+ Complete 原則; src/CLAUDE.md §XES互換イベント必須属性）
    status              TEXT NOT NULL,
    -- ステップ開始日時（null = pending 状態）
    started_at          TEXT,
    -- ステップ完了日時（null = 未完了）
    completed_at        TEXT,
    -- 実施作業者 ID（null = 未着手）
    operator_id         TEXT,
    -- サーバー同期済みフラグ（0=未同期, 1=同期済）
    synced_to_server    INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_local_work_steps_execution_id ON local_work_steps (work_execution_id);
CREATE INDEX IF NOT EXISTS idx_local_work_steps_step_id ON local_work_steps (step_id);
-- 未同期ステップの抽出用インデックス
CREATE INDEX IF NOT EXISTS idx_local_work_steps_sync ON local_work_steps (synced_to_server, work_execution_id);

-- ============================================================
-- local_work_assignments — ローカル作業割当（Push 型）
--   PG 対応: 割当はサーバー主導（SSE で端末に Push; PG の割当管理テーブルは PG-only）
--   SSE（Server-Sent Events）から受信した割当を端末ローカルに保存する
--   ネットワーク切断中でも直前に受信した割当を参照可能にする
-- ============================================================
CREATE TABLE IF NOT EXISTS local_work_assignments (
    -- 割当 ID（サーバーから受信した ID をそのまま使用）
    id                  TEXT PRIMARY KEY,
    -- 割り当てられた SOP の ID（ミラー FK）
    sop_id              TEXT NOT NULL REFERENCES sops(id),
    -- 作業指示 ID（XES Case ID）
    case_id             TEXT NOT NULL,
    -- ロット ID（null = ロット指定なし）
    lot_id              TEXT,
    -- 割当優先度（高い値 = 高優先度）
    priority            INTEGER NOT NULL DEFAULT 0,
    -- 割当状態（'pending' | 'accepted' | 'started' | 'done'）
    status              TEXT NOT NULL DEFAULT 'pending',
    -- サーバーから割当を受信した日時（SSE 受信時刻; 端末ローカル記録）
    received_at         TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    -- 割当者の ID（null = システム自動割当）
    assigned_by         TEXT
);

CREATE INDEX IF NOT EXISTS idx_local_work_assignments_case_id ON local_work_assignments (case_id);
CREATE INDEX IF NOT EXISTS idx_local_work_assignments_status ON local_work_assignments (status, priority);
