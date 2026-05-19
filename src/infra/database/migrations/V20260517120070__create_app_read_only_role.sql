-- V20260517120070__create_app_read_only_role.sql
-- app_read_only グループロールの作成と wnav_read のグループ所属修正。
--
-- 背景（修正理由）:
--   02_create_app_roles.sql の旧バージョンでは wnav_read を app_read_write に追加していた。
--   app_read_write は INSERT / UPDATE 権限を保有するため、
--   wnav_read がこれを継承すると「読み取り専用」の設計意図に反する。
--
--   本マイグレーションで:
--     1. app_read_only（SELECT 専用グループロール）を作成する
--     2. 全テーブル・全シーケンスへの SELECT を付与する
--     3. wnav_read を app_read_only に追加する
--
-- 実行ユーザー: wnav_admin（CREATEROLE 保有 → CREATE ROLE / GRANT ROLE が可能）
--              テーブルは wnav_admin が V01〜V68 で作成しているため GRANT SELECT も可能
--
-- 対象ドキュメント:
--   docs/06_実装/06_開発環境構築手順.md §4（GRANT app_read TO wnav_read 記述）
--   docs/06_実装/12_環境変数とシークレット一覧.md（WNAV_DB_USER_READ=wnav_read）
--   ADR-015（アプリロール名統一）

-- =============================================================================
-- Step 1: app_read_only グループロールの冪等作成
-- =============================================================================
DO $$ BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'app_read_only') THEN
        CREATE ROLE app_read_only NOLOGIN;
    END IF;
END $$;

-- =============================================================================
-- Step 2: SELECT 権限の付与（全テーブル・全シーケンス）
-- このマイグレーション実行時点で存在するすべてのテーブルが対象となる。
-- 以降に追加されるテーブル（将来のマイグレーション）は個別に GRANT SELECT を付与すること。
-- =============================================================================
GRANT SELECT ON ALL TABLES    IN SCHEMA public TO app_read_only;
GRANT SELECT ON ALL SEQUENCES IN SCHEMA public TO app_read_only;

-- =============================================================================
-- Step 3: wnav_read を app_read_only に追加する
-- wnav_admin は CREATEROLE を保有するため非スーパーユーザーロールへの GRANT ROLE が可能。
-- 03_schema_grants.sh で既に追加済みの場合も冪等（PostgreSQL は重複 GRANT を無視する）。
-- =============================================================================
GRANT app_read_only TO wnav_read;
