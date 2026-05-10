#!/usr/bin/env bash
# 対応 §: ロードマップ §13.4.2 S-02 §10.3.1
# シナリオ S-02: サーバ DB 接続断

set -euo pipefail

DURATION="${1:-30s}"

echo "[S-02] PostgreSQL コンテナを pause（${DURATION}）"

if command -v docker >/dev/null 2>&1; then
  docker pause wna-postgres
  trap 'docker unpause wna-postgres 2>/dev/null || true' EXIT INT TERM
  sleep "$DURATION"
  docker unpause wna-postgres
  echo "[S-02] 復旧完了"
else
  echo "[S-02] docker 未導入。iptables ルール等で代替"
  exit 1
fi

# 検証ポイント:
# - 指数バックオフでリトライされる
# - 24h で DLQ 遷移する
# - §10.3.5 受入観点
echo "[S-02] 検証: §10.3.5／F-011"
