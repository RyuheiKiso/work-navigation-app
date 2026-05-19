#!/bin/bash
# 00_create_databases.sh — 初期データベース作成スクリプト
# 権威ドキュメント:
#   docs/08_移行/導入手順/03_PostgreSQL初期化とマイグレーション手順.md §2
#   ADR-012（PostgreSQL 16-alpine 採用）
#
# このスクリプトは PostgreSQL コンテナの初回起動時にのみ実行される。
# docker-entrypoint-initdb.d/ 配下のスクリプトはデータディレクトリが空の場合に自動実行される。
#
# ロケール方針（ADR-012 補足）:
#   postgres:16-alpine は musl libc を使用するため glibc の ja_JP.UTF-8 ロケールが利用不可。
#   PostgreSQL 16 の ICU ロケールプロバイダ（LOCALE_PROVIDER = 'icu' ICU_LOCALE = 'ja-JP'）を
#   使用することで Alpine 上でも正しい日本語コレーションを実現する。
#
# 拡張機能:
#   V20260517120000（create_extensions マイグレーション）も同一拡張を冪等作成するが、
#   マイグレーションは wnav_admin（非スーパーユーザー）で実行するため、
#   pg_stat_statements のような superuser 要求拡張はここでスーパーユーザーとして事前に作成する。
#
# 使用方法:
#   環境変数 WNAV_DATABASES にスペース区切りでデータベース名を指定する。
#   デフォルト: wnav_dev
#   例: WNAV_DATABASES="wnav_dev wnav_test" の場合、2 つの DB を作成する。

set -euo pipefail

for db in ${WNAV_DATABASES:-wnav_dev}; do

    # ==========================================================================
    # Step 1: データベースの冪等作成
    # LOCALE_PROVIDER = 'icu' + ICU_LOCALE = 'ja-JP' で Alpine 互換の日本語コレーションを設定する
    # ==========================================================================
    psql -v ON_ERROR_STOP=1 --username "${POSTGRES_USER}" --dbname postgres <<-EOSQL
        SELECT 'CREATE DATABASE "${db}" WITH ENCODING ''UTF8'' LOCALE_PROVIDER ''icu'' ICU_LOCALE ''ja-JP'' TEMPLATE template0'
        WHERE NOT EXISTS (SELECT FROM pg_database WHERE datname = '${db}')
        \gexec
EOSQL
    echo "[00_create_databases] Database '${db}' created (or already exists)."

    # ==========================================================================
    # Step 2: 拡張機能の作成（スーパーユーザーとして実行）
    # pg_stat_statements は shared_preload_libraries に設定済みであることが前提。
    # V20260517120000 マイグレーションでも冪等作成するが、
    # wnav_admin（非スーパーユーザー）での実行を保証するためここで先行作成する。
    # ==========================================================================
    psql -v ON_ERROR_STOP=1 --username "${POSTGRES_USER}" --dbname "${db}" <<-EOSQL
        -- pgcrypto: gen_random_uuid() / SHA-256 ハッシュ計算
        CREATE EXTENSION IF NOT EXISTS pgcrypto;

        -- uuid-ossp: UUID 生成関数の補完（pgcrypto の uuid 関数と共存）
        CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

        -- pg_stat_statements: クエリ統計情報の収集（superuser 要求、shared_preload_libraries 設定済み）
        CREATE EXTENSION IF NOT EXISTS pg_stat_statements;
EOSQL
    echo "[00_create_databases] Extensions created in '${db}'."

done
