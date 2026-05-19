#!/bin/bash
# 00_create_databases.sh — 初期データベース作成スクリプト
# 権威ドキュメント:
#   docs/08_移行/導入手順/03_PostgreSQL初期化とマイグレーション手順.md §2
#   ADR-012（PostgreSQL 16 採用）
#
# このスクリプトは PostgreSQL コンテナの初回起動時にのみ実行される。
# docker-entrypoint-initdb.d/ 配下のスクリプトはデータディレクトリが空の場合に自動実行される。
#
# 使用方法:
#   環境変数 WNAV_DATABASES にスペース区切りでデータベース名を指定する。
#   デフォルト: wnav_dev
#   例: WNAV_DATABASES="wnav_dev wnav_test" の場合、2 つの DB を作成する。

set -e

# 環境変数 WNAV_DATABASES に指定されたデータベースを作成する
for db in ${WNAV_DATABASES:-wnav_dev}; do
    psql -v ON_ERROR_STOP=1 --username "${POSTGRES_USER}" --dbname postgres <<-EOSQL
        SELECT 'CREATE DATABASE ${db} WITH ENCODING ''UTF8'' LC_COLLATE ''ja_JP.UTF-8'' LC_CTYPE ''ja_JP.UTF-8'' TEMPLATE template0'
        WHERE NOT EXISTS (SELECT FROM pg_database WHERE datname = '${db}')
        \gexec
EOSQL
    echo "Database '${db}' created (or already exists)."
done
