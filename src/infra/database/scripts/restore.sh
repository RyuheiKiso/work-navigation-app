#!/bin/bash
# restore.sh — フルリストアスクリプト
# 権威ドキュメント:
#   docs/04_概要設計/08_運用方式設計/04_バックアップ・リストア方式.md §5
#   docs/09_運用・保守/運用手順/10_ActiveStandby切替手順（OPS-PROC-010）.md §4.2
#
# 処理概要（5 ステップ）:
#   Step 1: SHA-256 チェックサムでバックアップ整合性を検証する
#   Step 2: openssl AES-256-GCM で復号する
#   Step 3: gunzip で解凍する
#   Step 4: pg_restore でリストアする
#   Step 5: ハッシュチェーン整合性を検証する（verify_hash_chain.sql を実行する）
#
# 使用方法:
#   ./restore.sh <backup_file.dump.gz.enc> <target_database>
#   例: ./restore.sh /backup/db/work_nav_20260519.dump.gz.enc wnav_prod
#
# 必要な環境変数:
#   DATABASE_URL          — pg_restore 接続先 URL（リストア先 DB を含む）
#   BACKUP_ENCRYPTION_KEY — AES-256-GCM 復号キー（16 進数 64 文字）
#   BACKUP_ENCRYPTION_IV  — AES-256-GCM 初期化ベクトル（16 進数 32 文字）

set -euo pipefail

# 引数の確認
BACKUP_ENC="${1:?引数 1: バックアップファイルパスを指定してください（例: /backup/db/work_nav_20260519.dump.gz.enc）}"
TARGET_DB="${2:?引数 2: リストア先データベース名を指定してください（例: wnav_prod）}"

# 環境変数の存在確認
: "${DATABASE_URL:?環境変数 DATABASE_URL が設定されていません}"
: "${BACKUP_ENCRYPTION_KEY:?環境変数 BACKUP_ENCRYPTION_KEY が設定されていません}"
: "${BACKUP_ENCRYPTION_IV:?環境変数 BACKUP_ENCRYPTION_IV が設定されていません}"

# 作業ディレクトリ
WORK_DIR="/tmp/wnav_restore_$(date +%Y%m%d%H%M%S)"
mkdir -p "${WORK_DIR}"

# クリーンアップトラップ（正常終了・エラー終了どちらでも作業ファイルを削除する）
cleanup() {
    echo "[$(date -u +%Y-%m-%dT%H:%M:%SZ)] 作業ファイルをクリーンアップ中..."
    rm -rf "${WORK_DIR}"
}
trap cleanup EXIT

GZ_FILE="${WORK_DIR}/restore.dump.gz"
DUMP_FILE="${WORK_DIR}/restore.dump"

echo "[$(date -u +%Y-%m-%dT%H:%M:%SZ)] リストア開始"
echo "  バックアップファイル: ${BACKUP_ENC}"
echo "  リストア先 DB: ${TARGET_DB}"

# ----- Step 1: SHA-256 チェックサムでバックアップ整合性を検証する -----
echo "[$(date -u +%Y-%m-%dT%H:%M:%SZ)] Step 1: SHA-256 整合性検証..."
CHECKSUM_FILE="${BACKUP_ENC}.sha256"
if [ ! -f "${CHECKSUM_FILE}" ]; then
    echo "エラー: チェックサムファイルが存在しません: ${CHECKSUM_FILE}"
    exit 1
fi
# sha256sum -c でチェックサムを検証する（ミスマッチ時は即時終了する）
sha256sum -c "${CHECKSUM_FILE}" || {
    echo "エラー: SHA-256 チェックサムが一致しません。バックアップが破損している可能性があります。"
    exit 1
}
echo "[$(date -u +%Y-%m-%dT%H:%M:%SZ)] Step 1: SHA-256 検証 OK"

# ----- Step 2: openssl AES-256-GCM で復号する -----
echo "[$(date -u +%Y-%m-%dT%H:%M:%SZ)] Step 2: AES-256-GCM 復号中..."
openssl enc -d -aes-256-gcm \
    -K "${BACKUP_ENCRYPTION_KEY}" \
    -iv "${BACKUP_ENCRYPTION_IV}" \
    -in "${BACKUP_ENC}" \
    -out "${GZ_FILE}"
echo "[$(date -u +%Y-%m-%dT%H:%M:%SZ)] Step 2: 復号完了"

# ----- Step 3: gunzip で解凍する -----
echo "[$(date -u +%Y-%m-%dT%H:%M:%SZ)] Step 3: gunzip 解凍中..."
gunzip -c "${GZ_FILE}" > "${DUMP_FILE}"
echo "[$(date -u +%Y-%m-%dT%H:%M:%SZ)] Step 3: 解凍完了"

# ----- Step 4: pg_restore でリストアする -----
echo "[$(date -u +%Y-%m-%dT%H:%M:%SZ)] Step 4: pg_restore 実行中..."
pg_restore \
    --dbname="${DATABASE_URL}" \
    --clean \
    --if-exists \
    --no-owner \
    --no-acl \
    "${DUMP_FILE}"
echo "[$(date -u +%Y-%m-%dT%H:%M:%SZ)] Step 4: pg_restore 完了"

# ----- Step 5: ハッシュチェーン整合性を検証する -----
echo "[$(date -u +%Y-%m-%dT%H:%M:%SZ)] Step 5: ハッシュチェーン整合性検証中..."
SCRIPT_DIR="$(dirname "$0")"
INVALID_COUNT=$(psql "${DATABASE_URL}" \
    --tuples-only \
    --no-align \
    --file="${SCRIPT_DIR}/verify_hash_chain.sql" | \
    grep -c 'ERR-DB-003' || true)

if [ "${INVALID_COUNT}" -gt 0 ]; then
    echo "エラー: ハッシュチェーン整合性検証に失敗しました。"
    echo "  不整合レコード数: ${INVALID_COUNT}"
    echo "  ERR-DB-003 が検出されました。system_admin に報告してください。"
    exit 1
fi
echo "[$(date -u +%Y-%m-%dT%H:%M:%SZ)] Step 5: ハッシュチェーン整合性 OK（不整合なし）"

echo "[$(date -u +%Y-%m-%dT%H:%M:%SZ)] リストア完了（RTO 15 分以内の完了を確認すること）"
