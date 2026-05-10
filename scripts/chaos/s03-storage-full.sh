#!/usr/bin/env bash
# 対応 §: ロードマップ §13.4.2 S-03 §10.4.4
# シナリオ S-03: ストレージ満杯

set -euo pipefail

# 縮減サイズ（既定: 残容量 - 1MB）
FILE="${WNA_BALLAST:-/tmp/wna-ballast}"
SIZE="${1:-1G}"

echo "[S-03] バラスト ${FILE}（${SIZE}）を作成しストレージを縮減"

if command -v fallocate >/dev/null 2>&1; then
  fallocate -l "$SIZE" "$FILE"
  trap 'rm -f '"$FILE"'' EXIT INT TERM
  echo "[S-03] バラスト確保完了。アプリでメディア取得を試行してください"
  echo "[S-03] 80% で警告／90% でブロックされることを目視確認"
  read -r -p "確認後 Enter で復旧: " _ || true
  rm -f "$FILE"
else
  echo "[S-03] fallocate 未導入。dd if=/dev/zero of=... で代替"
  exit 1
fi

echo "[S-03] 検証: §10.4.6／F-011"
