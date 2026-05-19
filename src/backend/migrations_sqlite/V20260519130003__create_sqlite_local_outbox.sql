-- V20260519130003__create_sqlite_local_outbox.sql
--
-- 端末ローカル Outbox テーブル — 未同期イベントのバッファリング
--
-- ペア・マイグレーション規則（ADR-006）:
--   PG 対応: outbox_events テーブル（TBL-003 ミラー; Append-only + status UPDATE）
--   SQLite: 本テーブル（local_outbox）はクライアント発の Outbox Pattern 実装
--
-- 設計意図（Offline-First 原則; src/CLAUDE.md §1）:
--   端末がネットワーク切断状態でも作業記録を継続できるよう、
--   サーバーへの送信待ちイベントをローカルにバッファする。
--   接続復旧後にバックグラウンド同期プロセスが idempotency_key を使って
--   冪等に再送する（Idempotent API 原則; src/CLAUDE.md §3）。
--
-- 冪等性保証:
--   idempotency_key (UNIQUE) によりサーバー側の重複記録を防止する。
--   同一 idempotency_key の再送時はサーバーがキャッシュからレスポンスを返す（TTL 24h）。

PRAGMA journal_mode=WAL;
PRAGMA foreign_keys=ON;

-- ============================================================
-- local_outbox — 端末ローカル Outbox テーブル
-- ============================================================
CREATE TABLE IF NOT EXISTS local_outbox (
    -- 端末ローカルの Outbox レコード ID（UUID v7; アプリ側で生成）
    id                  TEXT PRIMARY KEY,
    -- イベント種別（列挙型; 自由文字列禁止）
    -- 例: 'work_started' | 'step_completed' | 'step_skipped' | 'work_suspended' | 'work_resumed'
    --     | 'andon_raised' | 'nonconformity_recorded' | 'electronic_signed'
    event_type          TEXT NOT NULL,
    -- イベントペイロード（JSON; PG への送信内容と同一構造）
    payload             TEXT NOT NULL CHECK (json_valid(payload)),
    -- 作業指示 ID または製品ロット ID（XES 互換 Case ID）
    case_id             TEXT NOT NULL,
    -- 端末での記録時刻（ISO 8601 UTC; Local-First 原則: 端末申告時刻を保持）
    client_recorded_at  TEXT NOT NULL,
    -- 冪等性キー（UUID v4; サーバーへの再送時に重複防止のために使用; TTL 24h）
    idempotency_key     TEXT NOT NULL UNIQUE,
    -- 送信リトライ回数（バックオフ制御に使用）
    retry_count         INTEGER NOT NULL DEFAULT 0,
    -- 最終送信試行日時（指数バックオフの計算基点）
    last_attempted_at   TEXT,
    -- 最終エラーメッセージ（デバッグ・運用監視用）
    error_message       TEXT,
    -- レコード生成日時
    created_at          TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

-- 再送キュー処理用インデックス: 未送信（retry_count 昇順）× 作成日時で優先度付け
CREATE INDEX IF NOT EXISTS idx_local_outbox_status ON local_outbox (retry_count, created_at);

-- case_id 単位での Outbox 件数確認・表示用インデックス
CREATE INDEX IF NOT EXISTS idx_local_outbox_case_id ON local_outbox (case_id);
