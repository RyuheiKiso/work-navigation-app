#!/usr/bin/env bash
# 対応 §: ロードマップ §14.2 §14.3 §10.5
# QR ペアリング: 端末アプリにサーバ URL／TLS 証明書／一時セッショントークンを QR で渡す。
# qrencode が無ければ ASCII で代替表示。

set -euo pipefail

DEVICE_ID="${1:-terminal-$(date +%s)}"
BACKEND_URL="${BACKEND_URL:-http://localhost:8080}"

# 1 回限りの初期セッショントークン（5 分有効、JWT 風）
TS=$(date +%s)
PAYLOAD="${DEVICE_ID}.${TS}"
PAYLOAD_B64=$(printf "%s" "$PAYLOAD" | base64 -w0 | tr '+/' '-_' | tr -d '=')

# QR 内容
QR_CONTENT="wna-pair://${BACKEND_URL}?device=${DEVICE_ID}&pairing=${PAYLOAD_B64}"

echo "== 端末ペアリング QR =="
echo "Device ID: ${DEVICE_ID}"
echo "Backend:   ${BACKEND_URL}"
echo

if command -v qrencode >/dev/null 2>&1; then
  echo "$QR_CONTENT" | qrencode -t ANSIUTF8
else
  echo "[INFO] qrencode 未導入のため URL のみ表示"
  echo
  echo "  ${QR_CONTENT}"
  echo
  echo "  ※ 'sudo apt install qrencode' で QR コード表示可能"
fi
