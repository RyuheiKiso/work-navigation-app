#!/bin/bash
# evidence_rsync.sh — 証拠ファイル NAS 同期スクリプト（日次 03:00 JST）
# docs/04_概要設計/08_運用方式設計/04_バックアップ・リストア方式.md 参照
set -euo pipefail

EVIDENCE_SOURCE="${EVIDENCE_SOURCE:-/data/evidence}"
NAS_TARGET="${NAS_TARGET:?NAS_TARGET 環境変数を設定してください（例: nas.internal:/backup/evidence）}"
LOG_FILE="${LOG_FILE:-/var/log/wnav/evidence_rsync.log}"

echo "[$(date -u +%Y-%m-%dT%H:%M:%SZ)] 証拠ファイル rsync 同期開始"
echo "  ソース: $EVIDENCE_SOURCE"
echo "  ターゲット: $NAS_TARGET"

# rsync で差分同期する（--checksum で変更確認、--archive でパーミッション保持）
rsync \
    --archive \
    --checksum \
    --verbose \
    --log-file="$LOG_FILE" \
    --stats \
    "$EVIDENCE_SOURCE/" \
    "$NAS_TARGET/"

echo "[$(date -u +%Y-%m-%dT%H:%M:%SZ)] 証拠ファイル rsync 同期完了"
