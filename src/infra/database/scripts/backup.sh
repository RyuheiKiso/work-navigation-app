#!/bin/bash
# backup.sh — 日次フルバックアップスクリプト（BAT-005）
# 権威ドキュメント:
#   docs/05_詳細設計/07_アルゴリズム詳細設計/06_バッチジョブ処理詳細（BAT-001〜010）.md §6 BAT-005
#   docs/04_概要設計/08_運用方式設計/04_バックアップ・リストア方式.md §2
#
# 処理概要:
#   1. pg_dump --format=custom --compress=9 でダンプ取得
#   2. gzip 外層圧縮でファイルサイズをさらに削減する
#   3. openssl AES-256-GCM で暗号化する
#   4. 90 日以上前のバックアップを削除する（BAT-005 仕様: 90 日保持）
#   5. /backup/db/ に保存する
#
# 必要な環境変数:
#   DATABASE_URL          — pg_dump 接続先 URL
#   BACKUP_ENCRYPTION_KEY — AES-256-GCM 暗号化キー（16 進数 64 文字）
#   BACKUP_ENCRYPTION_IV  — AES-256-GCM 初期化ベクトル（16 進数 32 文字）
#   STANDBY_HOST          — Standby ノードの rsync 転送先（省略可。例: standby.internal:/backup/db）
#
# 実行スケジュール: 毎日 02:00 JST（BAT-005）
# Standby rsync:   毎日 02:15 JST（BAT-005 完了後に STANDBY_HOST が設定されている場合のみ）

set -euo pipefail

# バックアップ保存先ディレクトリ
BACKUP_DIR="/backup/db"

# バックアップファイル名（命名規則: work_nav_YYYYMMDD.dump.gz.enc）
DATE="$(date +%Y%m%d)"
DUMP_FILE="${BACKUP_DIR}/work_nav_${DATE}.dump"
GZ_FILE="${DUMP_FILE}.gz"
ENC_FILE="${GZ_FILE}.enc"

# 保持日数（BAT-005 仕様: 90 日以上前のバックアップを削除する）
RETENTION_DAYS=90

# 環境変数の存在確認
: "${DATABASE_URL:?環境変数 DATABASE_URL が設定されていません}"
: "${BACKUP_ENCRYPTION_KEY:?環境変数 BACKUP_ENCRYPTION_KEY が設定されていません}"
: "${BACKUP_ENCRYPTION_IV:?環境変数 BACKUP_ENCRYPTION_IV が設定されていません}"

echo "[$(date -u +%Y-%m-%dT%H:%M:%SZ)] BAT-005 バックアップ開始"
echo "  保存先: ${ENC_FILE}"

# バックアップディレクトリが存在しない場合は作成する
mkdir -p "${BACKUP_DIR}"

# Step 1: pg_dump でダンプを取得する（--format=custom は内部圧縮を含む）
echo "[$(date -u +%Y-%m-%dT%H:%M:%SZ)] pg_dump 実行中..."
pg_dump \
    --format=custom \
    --compress=9 \
    --dbname="${DATABASE_URL}" \
    --file="${DUMP_FILE}"

# Step 2: gzip 外層圧縮で転送サイズをさらに削減する
# --format=custom は既に圧縮を内包するが、外層 gzip でさらに削減する
echo "[$(date -u +%Y-%m-%dT%H:%M:%SZ)] gzip 圧縮中..."
gzip -k "${DUMP_FILE}"
# 元の .dump ファイルを削除して .gz のみを保持する
rm -f "${DUMP_FILE}"

# Step 3: openssl AES-256-GCM で暗号化する（改ざん検知付き）
echo "[$(date -u +%Y-%m-%dT%H:%M:%SZ)] AES-256-GCM 暗号化中..."
openssl enc -aes-256-gcm \
    -K "${BACKUP_ENCRYPTION_KEY}" \
    -iv "${BACKUP_ENCRYPTION_IV}" \
    -in "${GZ_FILE}" \
    -out "${ENC_FILE}"
# 暗号化前の .gz ファイルを削除する
rm -f "${GZ_FILE}"

# チェックサムファイルを生成して整合性検証に備える
sha256sum "${ENC_FILE}" > "${ENC_FILE}.sha256"
echo "[$(date -u +%Y-%m-%dT%H:%M:%SZ)] SHA-256 チェックサム生成完了: ${ENC_FILE}.sha256"

# Step 4: 90 日以上前のバックアップを削除する（BAT-005 仕様: 90 日保持）
# find -mtime +90 で 90 日以上前のファイルを検索して削除する
echo "[$(date -u +%Y-%m-%dT%H:%M:%SZ)] 古いバックアップを削除中（${RETENTION_DAYS} 日保持）..."
find "${BACKUP_DIR}" -maxdepth 1 -name 'work_nav_*.dump.gz.enc' \
    -mtime "+${RETENTION_DAYS}" -exec rm -f {} \;

# 対応するチェックサムファイルも削除する
find "${BACKUP_DIR}" -maxdepth 1 -name 'work_nav_*.dump.gz.enc.sha256' \
    -mtime "+${RETENTION_DAYS}" -exec rm -f {} \;

echo "[$(date -u +%Y-%m-%dT%H:%M:%SZ)] BAT-005 バックアップ完了"
echo "  ファイル: ${ENC_FILE}"
echo "  サイズ: $(du -sh "${ENC_FILE}" | cut -f1)"

# Step 5: Standby ノードへ rsync 転送する（STANDBY_HOST が設定されている場合のみ）
# 権威: docs/04_概要設計/08_運用方式設計/04_バックアップ・リストア方式.md §2.1
if [[ -n "${STANDBY_HOST:-}" ]]; then
    echo "[$(date -u +%Y-%m-%dT%H:%M:%SZ)] Standby への rsync 転送中: ${STANDBY_HOST}"
    rsync \
        --archive \
        --checksum \
        --verbose \
        "${BACKUP_DIR}"/work_nav_*.dump.gz.enc \
        "${BACKUP_DIR}"/work_nav_*.dump.gz.enc.sha256 \
        "${STANDBY_HOST}/"
    echo "[$(date -u +%Y-%m-%dT%H:%M:%SZ)] Standby rsync 転送完了"
else
    echo "[$(date -u +%Y-%m-%dT%H:%M:%SZ)] STANDBY_HOST 未設定のため rsync をスキップ"
fi
