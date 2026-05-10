#!/usr/bin/env bash
# 対応 §: ロードマップ §14.2
# `make demo` から呼ばれる、バックエンドの readiness 待機スクリプト。
# /readyz は DB への SELECT 1 が通った時のみ 200 を返すため、
# ここでの待機が抜けると seed が `connection refused` で落ちる。

set -euo pipefail

# 既定の URL（compose の backend が公開する 8080 を想定）
URL="${WNA_READY_URL:-http://localhost:8080/readyz}"
# 最大待機秒数（既定 60s。重い CI でも余裕がある下限）
TIMEOUT="${WNA_READY_TIMEOUT:-60}"
# ポーリング間隔
INTERVAL="${WNA_READY_INTERVAL:-2}"

echo "== バックエンド readiness 待機 ($URL, timeout ${TIMEOUT}s) =="

# curl の再試行に頼ると進捗が見えないため、自前ループにする
elapsed=0
while [ "$elapsed" -lt "$TIMEOUT" ]; do
  if curl --silent --fail --max-time 3 "$URL" >/dev/null 2>&1; then
    echo "✓ readyz 200 (待機 ${elapsed}s)"
    exit 0
  fi
  sleep "$INTERVAL"
  elapsed=$((elapsed + INTERVAL))
  echo "  ... 待機中 ${elapsed}s / ${TIMEOUT}s"
done

echo "✗ readyz が ${TIMEOUT}s 以内に 200 を返しませんでした" >&2
echo "  ヒント: 'docker compose logs backend' でログを確認してください" >&2
exit 1
