#!/bin/bash
# wal_archive.sh — WAL アーカイブスクリプト
# 権威ドキュメント:
#   docs/04_概要設計/08_運用方式設計/04_バックアップ・リストア方式.md §3
#
# このスクリプトは postgresql.conf の archive_command として使用する。
# 設定例:
#   archive_command = '/scripts/wal_archive.sh %p %f'
#
# 処理概要:
#   1. WAL ファイルをアーカイブディレクトリにコピーする
#   2. gzip 圧縮してストレージを削減する
#
# 引数:
#   $1 — WAL ファイルのフルパス（%p）
#   $2 — WAL ファイル名（%f）

set -euo pipefail

WAL_FILE="$1"
WAL_NAME="$2"
ARCHIVE_DIR="/wal_archive"

# アーカイブディレクトリが存在しない場合は作成する
mkdir -p "${ARCHIVE_DIR}"

# WAL ファイルをアーカイブディレクトリにコピーする
cp "${WAL_FILE}" "${ARCHIVE_DIR}/${WAL_NAME}"

# gzip 圧縮して WAL アーカイブのストレージ使用量を削減する（-f で既存ファイルを上書きする）
gzip -f "${ARCHIVE_DIR}/${WAL_NAME}"

echo "WAL archived: ${ARCHIVE_DIR}/${WAL_NAME}.gz"
