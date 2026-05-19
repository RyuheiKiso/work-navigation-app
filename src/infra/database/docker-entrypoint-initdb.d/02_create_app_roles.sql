-- 02_create_app_roles.sql — アプリケーション用グループロール作成スクリプト
-- 権威ドキュメント:
--   ADR-015（アプリロール名統一）
--   src/CLAUDE.md §2「Append-only Event Sourcing」原則
--
-- このスクリプトは PostgreSQL コンテナの初回起動時にのみ実行される。
-- グループロール（NOLOGIN）を先に作成し、ログインロールをグループに追加する。
--
-- マイグレーション V20260517120058 でも冪等作成するため、
-- ここでの作成はコンテナ初期化時点でロールが利用できる状態にするための前準備である。

-- app_event_writer: work_events への INSERT のみ許可するグループロール
-- Append-only 原則により UPDATE / DELETE は禁止する
DO $$ BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'app_event_writer') THEN
        CREATE ROLE app_event_writer NOLOGIN;
    END IF;
END $$;

-- app_read_write: マスタテーブルの SELECT / INSERT / UPDATE / DELETE を許可するグループロール
DO $$ BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'app_read_write') THEN
        CREATE ROLE app_read_write NOLOGIN;
    END IF;
END $$;

-- app_admin: スキーマ変更・GRANT 実行用グループロール（マイグレーション実行時に使用する）
DO $$ BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'app_admin') THEN
        CREATE ROLE app_admin NOLOGIN CREATEROLE;
    END IF;
END $$;

-- app_event_insert: case_locks / idempotency_keys への INSERT / UPDATE を許可するグループロール
-- これらのテーブルは Append-only の例外として排他制御・冪等性保証のために更新が必要
DO $$ BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'app_event_insert') THEN
        CREATE ROLE app_event_insert NOLOGIN;
    END IF;
END $$;

-- ログインロールをグループロールに追加する
-- wnav_write は app_read_write グループのメンバーとしてマスタ CRUD を実行する
GRANT app_read_write TO wnav_write;

-- wnav_event_insert は app_event_writer と app_event_insert 両グループのメンバーとする
-- 作業ログ記録と排他制御・冪等性保証の両方を担当する
GRANT app_event_writer TO wnav_event_insert;
GRANT app_event_insert TO wnav_event_insert;

-- wnav_read は app_read_write グループのメンバーとして読み取りのみを実行する
-- マイグレーションで SELECT 専用の権限に絞り込む
GRANT app_read_write TO wnav_read;
