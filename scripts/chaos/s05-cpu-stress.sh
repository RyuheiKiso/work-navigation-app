#!/usr/bin/env bash
# 対応 §: ロードマップ §13.4.2 S-05 §5.2
# シナリオ S-05: 高 CPU 負荷

set -euo pipefail

DURATION="${1:-60s}"
WORKERS="${2:-4}"

echo "[S-05] stress-ng で CPU 負荷 ${WORKERS} workers × ${DURATION}"

if command -v stress-ng >/dev/null 2>&1; then
  stress-ng --cpu "$WORKERS" --cpu-load 90 --timeout "$DURATION"
else
  echo "[S-05] stress-ng 未導入。yes >/dev/null で代替"
  for _ in $(seq 1 "$WORKERS"); do
    yes >/dev/null &
  done
  trap 'pkill yes 2>/dev/null || true' EXIT INT TERM
  sleep "$DURATION"
  pkill yes 2>/dev/null || true
fi

# 検証: タッチ応答 ≤ 100ms / 画面遷移 ≤ 200ms（§5.2）
echo "[S-05] 検証: §5.2／§5.4 受入観点"
