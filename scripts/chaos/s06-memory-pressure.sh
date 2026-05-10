#!/usr/bin/env bash
# 対応 §: ロードマップ §13.4.2 S-06 §11.6
# シナリオ S-06: メモリ逼迫

set -euo pipefail

LIMIT_MB="${1:-256}"
DURATION="${2:-60s}"

echo "[S-06] メモリ ${LIMIT_MB}MB に逼迫させる（${DURATION}）"

if command -v stress-ng >/dev/null 2>&1; then
  stress-ng --vm 1 --vm-bytes "${LIMIT_MB}M" --vm-keep --timeout "$DURATION"
else
  echo "[S-06] stress-ng 未導入。Python で代替"
  python3 -c "
import time
buf = bytearray(${LIMIT_MB} * 1024 * 1024)
time.sleep(${DURATION%s})
" || true
fi

# 検証: OOM 後の自動再起動と直前状態への復帰（§11.6）
echo "[S-06] 検証: §11.6／F-003"
