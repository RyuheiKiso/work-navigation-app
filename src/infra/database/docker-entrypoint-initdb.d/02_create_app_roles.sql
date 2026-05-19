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

-- app_event_writer: Append-only テーブルへの INSERT/SELECT のみ許可するグループロール
-- UPDATE / DELETE は Append-only 原則（src/CLAUDE.md §2）により物理禁止する
DO $$ BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'app_event_writer') THEN
        CREATE ROLE app_event_writer NOLOGIN;
    END IF;
END $$;

-- app_read_write: 業務テーブルへの INSERT / SELECT / UPDATE を許可するグループロール
-- DELETE は原則禁止（論理削除のみ許容、idempotency_keys は TTL 削除の例外）
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

-- app_event_insert: case_locks 制御テーブルへの完全操作を許可するグループロール
-- ADR-009 端末占有アルゴリズムに必要な排他制御のための例外ロール
DO $$ BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'app_event_insert') THEN
        CREATE ROLE app_event_insert NOLOGIN;
    END IF;
END $$;

-- app_read_only: SELECT 専用グループロール（監査・ダッシュボード・IDE 接続用）
-- wnav_read はこのグループに所属する。app_read_write には追加しない（INSERT/UPDATE 禁止）。
-- テーブルレベルの GRANT SELECT はマイグレーション V70 で付与する。
DO $$ BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'app_read_only') THEN
        CREATE ROLE app_read_only NOLOGIN;
    END IF;
END $$;

-- =============================================================================
-- ログインロールをグループロールに追加する
-- =============================================================================

-- wnav_write: マスタ CRUD 担当（app_read_write メンバー）
GRANT app_read_write TO wnav_write;

-- wnav_event_insert: 作業ログ記録 + 排他制御・冪等性保証の両方を担当
GRANT app_event_writer TO wnav_event_insert;
GRANT app_event_insert TO wnav_event_insert;

-- wnav_read: 読み取り専用（監査・ダッシュボード閲覧用）
-- app_read_only に追加する（app_read_write に追加すると INSERT/UPDATE を継承してしまうため禁止）
GRANT app_read_only TO wnav_read;
