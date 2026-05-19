-- V20260519130002__create_sqlite_app_settings.sql
--
-- 端末固有の設定テーブル — AppSettings（端末専用・PG 非同期）
--
-- ペア・マイグレーション規則（ADR-006）:
--   本テーブルは端末専用（PG-only ではなく端末専用）のため PG 対応マイグレーションは存在しない
--   07a_PG_SQLiteスキーマ同期戦略.md §1「端末専用テーブル」として定義
--
-- 設計意図:
--   キー・バリュー形式でアプリケーション設定を保存する。
--   schema_version でマイグレーション状態を追跡し、前進修正（Backward-compatible DDL）を保証する。

PRAGMA journal_mode=WAL;
PRAGMA foreign_keys=ON;

-- ============================================================
-- app_settings — 端末固有設定テーブル
--   key: 設定キー名（例: 'terminal_id', 'server_url', 'last_sync_at'）
--   value: 設定値（TEXT として格納; 数値・JSON も文字列として保存）
-- ============================================================
CREATE TABLE IF NOT EXISTS app_settings (
    key                 TEXT PRIMARY KEY,
    value               TEXT NOT NULL,
    -- 設定最終更新日時（SQLite 組み込み関数 strftime で UTC ISO 8601 形式を生成）
    updated_at          TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

-- 初期設定値の投入（OR IGNORE で冪等に実行可能）
INSERT OR IGNORE INTO app_settings (key, value) VALUES
    -- SQLite マイグレーション適用済みバージョン（前進修正の状態管理）
    ('schema_version', '1'),
    -- 最終サーバー同期日時（ISO 8601 UTC; 未同期の場合は空文字）
    ('last_sync_at', ''),
    -- この端末の端末 ID（TBL-021 terminals.id; 初回起動時にサーバーから取得して設定）
    ('terminal_id', ''),
    -- 接続先バックエンド API の URL（例: 'http://192.168.10.100:8080'）
    ('server_url', '');
