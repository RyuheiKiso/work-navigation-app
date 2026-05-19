-- V20260517120000__create_extensions.sql
-- PostgreSQL 拡張機能の有効化。冪等性確保のため IF NOT EXISTS を使用する。

-- pgcrypto: gen_random_uuid() / SHA-256 ハッシュ計算に使用する
CREATE EXTENSION IF NOT EXISTS pgcrypto;

-- uuid-ossp: uuid_generate_v4() 等の UUID 生成関数を提供する（pgcrypto の補完用）
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- pg_stat_statements: クエリ統計情報の収集に使用する（パフォーマンス監視）
CREATE EXTENSION IF NOT EXISTS pg_stat_statements;

-- ロール定義: NOLOGIN グループロール。冪等性確保のため DO ブロックで存在チェックを行う。

-- app_event_writer: Append-only テーブルへの INSERT/SELECT のみ許可するロール
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'app_event_writer') THEN
        CREATE ROLE app_event_writer NOLOGIN;
    END IF;
END;
$$;

-- app_read_write: 通常の SELECT/INSERT/UPDATE を許可するロール（DELETE は禁止）
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'app_read_write') THEN
        CREATE ROLE app_read_write NOLOGIN;
    END IF;
END;
$$;

-- app_admin: DDL 操作・GRANT 操作を許可する管理者ロール
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'app_admin') THEN
        CREATE ROLE app_admin NOLOGIN;
    END IF;
END;
$$;

-- app_event_insert: case_locks 等の制御テーブルに INSERT/UPDATE/DELETE を許可するロール
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'app_event_insert') THEN
        CREATE ROLE app_event_insert NOLOGIN;
    END IF;
END;
$$;
