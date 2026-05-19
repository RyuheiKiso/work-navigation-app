#!/bin/bash
# cold_archive.sh — コールドアーカイブスクリプト（月次メンテナンス・手動トリガ）
# 権威ドキュメント:
#   docs/05_詳細設計/01_データベース詳細設計/06_パーティション・アーカイブ詳細設計.md §3-3
#
# 対象: 61 ヶ月以上経過した work_events パーティション
# 処理概要:
#   1. pg_dump でパーティション単体をダンプする
#   2. gzip 圧縮する
#   3. AES-256-GCM 暗号化する
#   4. パーティションを親テーブルから切り離す（CONCURRENTLY でロックなし）
#   5. DROP TABLE は手動で実施する（ダンプ整合性確認後）
#
# 使用方法:
#   ./cold_archive.sh <partition_name>
#   例: ./cold_archive.sh work_events_y2021m01
#
# 必要な環境変数:
#   DATABASE_URL            — pg_dump / psql 接続先 URL
#   BACKUP_ENCRYPTION_KEY   — AES-256-GCM 暗号化キー（16 進数 64 文字）
#   BACKUP_ENCRYPTION_IV    — AES-256-GCM 初期化ベクトル（16 進数 32 文字）

set -euo pipefail

# 引数の確認
PARTITION="${1:?パーティション名を指定してください（例: work_events_y2021m01）}"

# 環境変数の存在確認
: "${DATABASE_URL:?環境変数 DATABASE_URL が設定されていません}"
: "${BACKUP_ENCRYPTION_KEY:?環境変数 BACKUP_ENCRYPTION_KEY が設定されていません}"
: "${BACKUP_ENCRYPTION_IV:?環境変数 BACKUP_ENCRYPTION_IV が設定されていません}"

# コールドアーカイブ保存先
COLD_DIR="/backup/cold"
DUMP_PATH="${COLD_DIR}/${PARTITION}.dump.gz.enc"

# コールドアーカイブディレクトリが存在しない場合は作成する
mkdir -p "${COLD_DIR}"

echo "[$(date -u +%Y-%m-%dT%H:%M:%SZ)] コールドアーカイブ開始"
echo "  対象パーティション: ${PARTITION}"
echo "  保存先: ${DUMP_PATH}"

# パーティションが存在するか確認する
PARTITION_EXISTS=$(psql "${DATABASE_URL}" \
    --tuples-only --no-align \
    -c "SELECT COUNT(*) FROM pg_tables WHERE tablename = '${PARTITION}';" \
    2>/dev/null || echo "0")

if [ "${PARTITION_EXISTS}" -eq 0 ]; then
    echo "エラー: パーティション ${PARTITION} が存在しません。"
    exit 1
fi

# ダンプ + gzip + AES-256-GCM 暗号化をパイプで実行する（中間ファイルを作成しない）
echo "[$(date -u +%Y-%m-%dT%H:%M:%SZ)] pg_dump + gzip + AES-256-GCM 暗号化中..."
pg_dump \
    --dbname="${DATABASE_URL}" \
    --table="${PARTITION}" \
    --format=custom \
    --compress=9 | \
gzip | \
openssl enc -aes-256-gcm \
    -K "${BACKUP_ENCRYPTION_KEY}" \
    -iv "${BACKUP_ENCRYPTION_IV}" \
    > "${DUMP_PATH}"

echo "[$(date -u +%Y-%m-%dT%H:%M:%SZ)] アーカイブ完了: ${DUMP_PATH}"
echo "  サイズ: $(du -sh "${DUMP_PATH}" | cut -f1)"

# チェックサムファイルを生成する
sha256sum "${DUMP_PATH}" > "${DUMP_PATH}.sha256"
echo "[$(date -u +%Y-%m-%dT%H:%M:%SZ)] SHA-256 チェックサム生成完了: ${DUMP_PATH}.sha256"

# パーティションを親テーブルから切り離す（CONCURRENTLY でロックを最小化する）
echo "[$(date -u +%Y-%m-%dT%H:%M:%SZ)] パーティション切り離し中（CONCURRENTLY）..."
psql "${DATABASE_URL}" \
    -c "ALTER TABLE work_events DETACH PARTITION ${PARTITION} CONCURRENTLY;"
echo "[$(date -u +%Y-%m-%dT%H:%M:%SZ)] パーティション切り離し完了"

echo ""
echo "次のステップ（手動で実施すること）:"
echo "  1. ダンプの整合性を確認する: sha256sum -c ${DUMP_PATH}.sha256"
echo "  2. ダンプが正常であることを確認した後、以下を実行してパーティションを削除する:"
echo "     psql \"\${DATABASE_URL}\" -c \"DROP TABLE ${PARTITION};\""
echo ""
echo "[$(date -u +%Y-%m-%dT%H:%M:%SZ)] コールドアーカイブ完了（DROP TABLE は手動で実施すること）"
