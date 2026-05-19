-- 01_create_login_roles.sql — ログインロール作成スクリプト
-- 権威ドキュメント:
--   ADR-015（アプリロール名統一）
--   docs/08_移行/導入手順/03_PostgreSQL初期化とマイグレーション手順.md §2-2
--
-- このスクリプトは PostgreSQL コンテナの初回起動時にのみ実行される。
-- パスワードは 'CHANGE_ME' のプレースホルダを設定しており、
-- 本番環境では必ず ALTER ROLE コマンドまたは Docker Secret で上書きすること。
--
-- 注意: パスワードは CHANGE_ME のままでは本番環境で使用しないこと。
--       .env.prod の WNAV_DB_PASSWORD で強固なパスワードを設定すること。

-- wnav_admin: DB オーナー・マイグレーション実行用（CREATEDB / CREATEROLE 権限を付与する）
DO $$ BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'wnav_admin') THEN
        CREATE ROLE wnav_admin WITH LOGIN CREATEDB CREATEROLE PASSWORD 'CHANGE_ME';
    END IF;
END $$;

-- wnav_backup: バックアップ専用（pg_read_all_data により全テーブルを SELECT 可能にする）
DO $$ BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'wnav_backup') THEN
        CREATE ROLE wnav_backup WITH LOGIN PASSWORD 'CHANGE_ME';
        GRANT pg_read_all_data TO wnav_backup;
    END IF;
END $$;

-- wnav_write: app_read_write グループロールのメンバー（マスタ CRUD 用）
-- グループへの追加はマイグレーション 02_create_app_roles.sql で実施する
DO $$ BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'wnav_write') THEN
        CREATE ROLE wnav_write WITH LOGIN PASSWORD 'CHANGE_ME';
    END IF;
END $$;

-- wnav_event_insert: app_event_writer のメンバー（作業ログ記録用）
-- グループへの追加はマイグレーション 02_create_app_roles.sql で実施する
DO $$ BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'wnav_event_insert') THEN
        CREATE ROLE wnav_event_insert WITH LOGIN PASSWORD 'CHANGE_ME';
    END IF;
END $$;

-- wnav_read: 読み取り権限のみ（監査・ダッシュボード閲覧用）
-- グループへの追加はマイグレーション 02_create_app_roles.sql で実施する
DO $$ BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'wnav_read') THEN
        CREATE ROLE wnav_read WITH LOGIN PASSWORD 'CHANGE_ME';
    END IF;
END $$;

-- wnav_replication: Standby レプリケーション専用ユーザー
-- pg_hba.conf の replication エントリ・postgresql.standby.conf の primary_conninfo で使用する。
-- REPLICATION 権限を付与する（ストリーミングレプリケーションに必要）。
-- 権威: docs/04_概要設計/01_システム方式設計/03_配置設計（Active_Standby・単一建屋内冗長）.md
DO $$ BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'wnav_replication') THEN
        CREATE ROLE wnav_replication WITH LOGIN REPLICATION PASSWORD 'CHANGE_ME';
    END IF;
END $$;
