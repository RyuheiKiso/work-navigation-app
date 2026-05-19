-- V20260519120001__create_roles_and_extensions.sql
-- PostgreSQL 拡張機能の有効化とアプリケーションロールの作成
-- gen_random_uuid() は PG13+ 組み込み済みだが pgcrypto も有効化する
-- ロールは IF NOT EXISTS（DO $$ ブロック）で冪等に作成する

-- =====================================================
-- 拡張機能
-- =====================================================

CREATE EXTENSION IF NOT EXISTS "pgcrypto";        -- gen_random_bytes 等の暗号化関数
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";       -- UUID 生成（バックアップ用・PG13以降は gen_random_uuid() が組み込み）
CREATE EXTENSION IF NOT EXISTS "pg_stat_statements"; -- クエリ性能統計（NFR-PRF 監視用）

-- =====================================================
-- アプリケーションロールの冪等作成
-- =====================================================

-- app_read: 全テーブルへの SELECT のみ（閲覧専用・経営層・監査用途）
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'app_read') THEN
        CREATE ROLE app_read NOLOGIN;
        COMMENT ON ROLE app_read IS
            'app_read — 全テーブルへの SELECT のみを許可する読み取り専用ロール。経営層（executive）・監査レポート用途。';
    END IF;
END
$$;

-- app_write: マスタテーブルへの SELECT/INSERT/UPDATE（DELETE 不可）
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'app_write') THEN
        CREATE ROLE app_write NOLOGIN;
        COMMENT ON ROLE app_write IS
            'app_write — マスタ系テーブル専用ロール。SELECT/INSERT/UPDATE を許可する。DELETE は全テーブルで禁止（物理削除禁止方針）。';
    END IF;
END
$$;

-- app_event_insert: Append-only テーブルへの INSERT のみ（作業ログ・イベント書き込み専用）
-- ADR-010 Append-only 保証: このロールに UPDATE/DELETE を付与しない
-- 例外: case_locks・idempotency_keys は ADR-009・制御テーブルとして INSERT/UPDATE/DELETE を許可（V008 で GRANT）
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'app_event_insert') THEN
        CREATE ROLE app_event_insert NOLOGIN;
        COMMENT ON ROLE app_event_insert IS
            'app_event_insert — Append-only テーブル（work_events / electronic_signs / evidence_files / measurements 等）への INSERT のみを許可するロール。ADR-010 Append-only 保証の実装。UPDATE/DELETE は付与しない。例外: case_locks・idempotency_keys は V008 で別途 GRANT。';
    END IF;
END
$$;
