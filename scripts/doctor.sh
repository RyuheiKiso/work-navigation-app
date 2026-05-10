#!/usr/bin/env bash
# 対応 §: ロードマップ §14.2
# `make doctor` のスクリプト実装版。make が無い環境でも実行可能。
# §14.2 自己診断: OS／Docker／空きポート／メモリ／時刻同期 を 30 秒以内に判定。

set -euo pipefail

echo "== work-navigation-app doctor =="
echo "OS: $(uname -srm)"
echo "Docker: $(docker --version 2>/dev/null || echo 'not installed')"
echo "Docker Compose: $(docker compose version 2>/dev/null || echo 'not installed')"
echo "Cargo: $(cargo --version 2>/dev/null || echo 'not installed')"
echo "Node: $(node --version 2>/dev/null || echo 'not installed')"
echo "pnpm: $(pnpm --version 2>/dev/null || echo 'not installed')"
echo "Free memory (MiB): $(free -m 2>/dev/null | awk '/^Mem:/{print $7}' || echo 'unknown')"
echo "NTP sync: $(timedatectl 2>/dev/null | grep -E 'synchronized' || echo 'unknown')"

# ポート占有
for port in 5432 8080 1420 1421; do
  if ss -ltn 2>/dev/null | awk -v p="$port" '$4 ~ ":"p"$" {found=1} END {exit !found}'; then
    echo "Port ${port}: busy"
  else
    echo "Port ${port}: free"
  fi
done

# 必要ディレクトリ
for d in services/backend apps/terminal apps/config-ui addon-sdk scripts; do
  if [ -d "$d" ]; then
    echo "Dir ${d}: ok"
  else
    echo "Dir ${d}: missing"
  fi
done

# 設定ファイル
for f in Cargo.toml package.json docker-compose.yml; do
  if [ -f "$f" ]; then
    echo "File ${f}: ok"
  else
    echo "File ${f}: missing"
  fi
done

echo "Done."
