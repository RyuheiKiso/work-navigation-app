#!/usr/bin/env bash
# 対応 §: ロードマップ §13.4.2 S-01 §10.6
# シナリオ S-01: 端末ネットワーク断（10s〜24h）

set -euo pipefail

# パラメータ（既定: 10 秒）
DURATION="${1:-10s}"
IFACE="${WNA_IFACE:-eth0}"

echo "[S-01] iface=${IFACE} に 100% パケットロスを注入（${DURATION}）"

# tc が存在するなら 100% loss を注入
if command -v tc >/dev/null 2>&1; then
  sudo tc qdisc add dev "$IFACE" root netem loss 100%
  trap 'sudo tc qdisc del dev '"$IFACE"' root 2>/dev/null || true' EXIT INT TERM
  sleep "$DURATION"
  sudo tc qdisc del dev "$IFACE" root
  echo "[S-01] 復旧完了"
else
  echo "[S-01] tc 未導入。Wi-Fi トグル等の手動操作で代替（手順は §13.4.2 参照）"
  exit 1
fi

# 検証ポイント:
# - 端末→サーバ同期再送が成功する（INV-01 NoEventLoss）
# - 同期遅延が §5.2 ≤ 5s で復旧する
echo "[S-01] 検証: §10.6.2 受入観点／INV-01"
