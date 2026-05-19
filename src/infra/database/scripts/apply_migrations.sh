#!/bin/bash
# apply_migrations.sh — sqlx migrate 実行ラッパー
# 権威ドキュメント:
#   docs/08_移行/導入手順/03_PostgreSQL初期化とマイグレーション手順.md §3
#   ADR-013（マイグレーション格納先: src/infra/database/migrations/）
#
# 処理概要:
#   1. dry-run でマイグレーション計画を確認する
#   2. 実際にマイグレーションを適用する
#   3. 実行ログをタイムスタンプ付きで出力する
#
# 必要な環境変数:
#   DATABASE_URL    — sqlx migrate の接続先 URL
#   MIGRATION_DIR   — マイグレーションファイルの格納先（デフォルト: src/infra/database/migrations）
#
# 使用方法:
#   DATABASE_URL="postgresql://wnav_admin:password@localhost:5432/wnav_prod" ./apply_migrations.sh

set -euo pipefail

# マイグレーションファイルのデフォルト格納先（ADR-013 確定: src/infra/database/migrations/）
MIGRATION_DIR="${MIGRATION_DIR:-src/infra/database/migrations}"

# 環境変数の存在確認
: "${DATABASE_URL:?DATABASE_URL が設定されていません}"

echo "[$(date -u +%Y-%m-%dT%H:%M:%SZ)] マイグレーション開始"
echo "  ディレクトリ: ${MIGRATION_DIR}"
echo "  接続先: $(echo "${DATABASE_URL}" | sed 's/:\/\/[^:]*:[^@]*@/:\/\/***:***@/')"

# マイグレーションファイルが存在するか確認する
if [ ! -d "${MIGRATION_DIR}" ]; then
    echo "エラー: マイグレーションディレクトリが存在しません: ${MIGRATION_DIR}"
    exit 1
fi

# dry-run でマイグレーション計画を確認する（実際の変更は行わない）
echo "[$(date -u +%Y-%m-%dT%H:%M:%SZ)] dry-run 確認中..."
sqlx migrate run \
    --source "${MIGRATION_DIR}" \
    --dry-run \
    --database-url "${DATABASE_URL}"

echo "[$(date -u +%Y-%m-%dT%H:%M:%SZ)] dry-run 完了（上記の差分を確認してください）"
echo ""

# 実際にマイグレーションを適用する
echo "[$(date -u +%Y-%m-%dT%H:%M:%SZ)] マイグレーション適用中..."
sqlx migrate run \
    --source "${MIGRATION_DIR}" \
    --database-url "${DATABASE_URL}"

echo "[$(date -u +%Y-%m-%dT%H:%M:%SZ)] マイグレーション完了"
