#!/bin/bash
# 03_schema_grants.sh — スキーマ権限・ロールメンバーシップ補正スクリプト
# 権威ドキュメント:
#   docs/08_移行/導入手順/03_PostgreSQL初期化とマイグレーション手順.md §2-3
#   ADR-015（アプリロール名統一）
#
# このスクリプトは 01_create_login_roles.sql / 02_create_app_roles.sql の実行後に
# 実行される（docker-entrypoint-initdb.d は名前の昇順で実行される）。
#
# 処理概要:
#   1. WNAV_DATABASES に列挙した各 DB の public スキーマに対して
#      wnav_admin へ CREATE 権限を付与する
#      （PostgreSQL 15+ でデフォルト CREATE が PUBLIC から剥奪されたため必須）
#   2. wnav_read の誤ったグループ所属（app_read_write）を補正する
#      02_create_app_roles.sql では app_read_only を使用しているが、
#      旧バージョンのコンテナイメージで初期化された環境の救済処置。
#
# 注意: このスクリプトはスーパーユーザー（POSTGRES_USER）として実行される。

set -euo pipefail

# =============================================================================
# Step 1: クラスタ共通 — wnav_read のグループ所属補正
# ロールメンバーシップはクラスタ全体に適用されるため、postgres DB に接続して実行する
# =============================================================================
psql -v ON_ERROR_STOP=1 --username "${POSTGRES_USER}" --dbname postgres <<-EOSQL

    -- wnav_read が app_read_write に属している場合は解除する（旧バージョン救済）
    DO \$\$ BEGIN
        IF EXISTS (
            SELECT FROM pg_auth_members m
            JOIN pg_roles g ON g.oid = m.roleid
            JOIN pg_roles r ON r.oid = m.member
            WHERE g.rolname = 'app_read_write' AND r.rolname = 'wnav_read'
        ) THEN
            REVOKE app_read_write FROM wnav_read;
        END IF;
    END \$\$;

    -- wnav_read を app_read_only に追加する（未追加の場合のみ）
    DO \$\$ BEGIN
        IF NOT EXISTS (
            SELECT FROM pg_auth_members m
            JOIN pg_roles g ON g.oid = m.roleid
            JOIN pg_roles r ON r.oid = m.member
            WHERE g.rolname = 'app_read_only' AND r.rolname = 'wnav_read'
        ) THEN
            GRANT app_read_only TO wnav_read;
        END IF;
    END \$\$;

EOSQL

echo "[03_schema_grants] wnav_read のグループ所属を app_read_only に確定しました。"

# =============================================================================
# Step 2: 各 DB — public スキーマの CREATE 権限を wnav_admin に付与する
# wnav_admin は sqlx migrate run でテーブル・インデックスを作成するため必要
# PostgreSQL 15+ はデフォルトで PUBLIC からの CREATE を剥奪しているため明示的付与が必須
# =============================================================================
for db in ${WNAV_DATABASES:-wnav_dev}; do
    psql -v ON_ERROR_STOP=1 --username "${POSTGRES_USER}" --dbname "${db}" <<-EOSQL
        -- wnav_admin が CREATE TABLE / CREATE INDEX 等の DDL を実行できるようにする
        GRANT CREATE ON SCHEMA public TO wnav_admin;
EOSQL
    echo "[03_schema_grants] GRANT CREATE ON SCHEMA public TO wnav_admin — DB: '${db}'"
done
