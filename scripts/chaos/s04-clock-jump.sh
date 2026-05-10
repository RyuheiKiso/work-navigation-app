#!/usr/bin/env bash
# 対応 §: ロードマップ §13.4.2 S-04 §20.2 §21 注 4
# シナリオ S-04: 時刻ジャンプ

set -euo pipefail

OFFSET="${1:-+10}"   # 既定 +10 秒

echo "[S-04] システム時刻を ${OFFSET} 秒ずらす"

if command -v timedatectl >/dev/null 2>&1; then
  # 現在時刻を取得
  NOW=$(date -u +%s)
  TARGET=$((NOW + OFFSET))
  TARGET_STR=$(date -u -d @"$TARGET" "+%Y-%m-%d %H:%M:%S")
  sudo timedatectl set-ntp false
  sudo timedatectl set-time "$TARGET_STR"
  trap 'sudo timedatectl set-ntp true' EXIT INT TERM
  echo "[S-04] 時刻ずれを設定。アプリで NTP ズレ警告（§20.2 5 秒閾値）が発火することを確認"
  read -r -p "確認後 Enter で NTP 復帰: " _ || true
  sudo timedatectl set-ntp true
else
  echo "[S-04] timedatectl 未導入。手動で /etc/localtime をいじって代替"
  exit 1
fi

echo "[S-04] 検証: §20.2／§21 注 4／F-017"
