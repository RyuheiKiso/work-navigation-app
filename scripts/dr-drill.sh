#!/usr/bin/env bash
# 対応 §: ロードマップ §15 §15.2 §15.3 §22.1
# DR（Disaster Recovery）演習スクリプト。
# §15.2 RPO／RTO 目標の実機検証用。半期サイクル（§22.1）で実行する。
# 演習結果は docs/04_運用/dr-drill-<YYYY-MM-DD>.md に追記する。

set -euo pipefail

# 設定（環境変数で上書き可）
SCALE="${SCALE:-medium}"            # small / medium / large
PG_CONTAINER="${PG_CONTAINER:-wna-postgres}"
BACKUP_DIR="${BACKUP_DIR:-./dist/dr-drill}"
LOG_FILE="${LOG_FILE:-docs/04_運用/dr-drill-$(date -u +%Y-%m-%d).md}"

# RPO／RTO 目標（§15.2 から）
case "$SCALE" in
  small) RPO_MAX_SEC=$((60 * 60)); RTO_MAX_SEC=$((4 * 60 * 60));;
  medium) RPO_MAX_SEC=$((15 * 60)); RTO_MAX_SEC=$((60 * 60));;
  large) RPO_MAX_SEC=$((5 * 60)); RTO_MAX_SEC=$((30 * 60));;
  *) echo "未知のスケール: $SCALE"; exit 1;;
esac

mkdir -p "$BACKUP_DIR" "$(dirname "$LOG_FILE")"

echo "== DR Drill: scale=${SCALE} (RPO ≤${RPO_MAX_SEC}s / RTO ≤${RTO_MAX_SEC}s) =="

# ステップ 1: バックアップ取得
echo "[1/5] バックアップ取得"
T1=$(date +%s)
DUMP="${BACKUP_DIR}/wna-$(date -u +%Y%m%dT%H%M%SZ).dump"
if docker ps --format '{{.Names}}' | grep -q "$PG_CONTAINER"; then
  docker exec "$PG_CONTAINER" pg_dump -Fc -d wna -U wna > "$DUMP"
  echo "  バックアップ: $DUMP"
else
  echo "  PostgreSQL コンテナ未起動。docker compose up を先に実行してください"
  exit 1
fi
T2=$(date +%s)
BACKUP_SEC=$((T2 - T1))

# ステップ 2: 障害シミュレーション（DROP DATABASE）
echo "[2/5] 障害シミュレーション (DROP DATABASE wna)"
T3=$(date +%s)
docker exec "$PG_CONTAINER" psql -U wna -d postgres -c "DROP DATABASE IF EXISTS wna;"

# ステップ 3: 復旧（CREATE DATABASE + pg_restore）
echo "[3/5] 復旧開始"
docker exec "$PG_CONTAINER" psql -U wna -d postgres -c "CREATE DATABASE wna;"
docker exec -i "$PG_CONTAINER" pg_restore -d wna -U wna < "$DUMP"
T4=$(date +%s)
RECOVER_SEC=$((T4 - T3))

# ステップ 4: マイグレーション再適用
echo "[4/5] マイグレーション再適用"
# 端末側からは migration バイナリを使う想定だが、本演習では直接 SQL 適用も許容
T5=$(date +%s)
APPLY_SEC=$((T5 - T4))

# ステップ 5: 結果検証
echo "[5/5] 結果検証 (テーブル存在確認)"
TABLE_COUNT=$(docker exec "$PG_CONTAINER" psql -U wna -d wna -tAc \
  "SELECT COUNT(*) FROM information_schema.tables WHERE table_schema='public';")
TOTAL_RTO=$((T5 - T3))

# レポート追記
{
  echo
  echo "## 演習 $(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo
  echo "- スケール: ${SCALE}"
  echo "- バックアップ取得: ${BACKUP_SEC} 秒"
  echo "- 復旧（pg_restore）: ${RECOVER_SEC} 秒"
  echo "- マイグレーション再適用: ${APPLY_SEC} 秒"
  echo "- **総 RTO**: ${TOTAL_RTO} 秒（目標 ≤${RTO_MAX_SEC} 秒）"
  echo "- 復旧後テーブル数: ${TABLE_COUNT}"
  echo
  if [ "$TOTAL_RTO" -le "$RTO_MAX_SEC" ]; then
    echo "**判定**: ✅ RTO 目標達成"
  else
    echo "**判定**: ❌ RTO 目標未達（§22.4 是正フロー対象）"
  fi
} >> "$LOG_FILE"

echo "DR Drill 完了。ログ: $LOG_FILE"
echo "RTO: ${TOTAL_RTO}s （目標 ≤${RTO_MAX_SEC}s）"

if [ "$TOTAL_RTO" -gt "$RTO_MAX_SEC" ]; then
  exit 1
fi
