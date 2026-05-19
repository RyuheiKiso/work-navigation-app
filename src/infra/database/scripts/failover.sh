#!/bin/bash
# failover.sh — Active-Standby フェイルオーバースクリプト
# 権威ドキュメント:
#   docs/09_運用・保守/運用手順/10_ActiveStandby切替手順（OPS-PROC-010）.md
#   docs/04_概要設計/01_システム方式設計/03_配置設計（Active_Standby・単一建屋内冗長）.md §4-2
#
# 処理概要:
#   Step 1: プライマリ停止を確認する
#   Step 2: standby.signal ファイルを削除してリカバリモードを終了する
#   Step 3: recovery.conf.template を postgresql.conf に追記する
#   Step 4: pg_ctl promote でプライマリに昇格させる
#   Step 5: 昇格確認（pg_isready + 書き込み可否テスト）
#
# 使用方法:
#   ./failover.sh
#
# 必要な環境変数:
#   PGDATA              — PostgreSQL データディレクトリ（デフォルト: /var/lib/postgresql/data）
#   POSTGRES_USER       — PostgreSQL 管理ユーザー（デフォルト: postgres）
#   POSTGRES_DB         — 疎通確認に使用するデータベース名（デフォルト: postgres）
#   PRIMARY_HOST        — プライマリホスト名（停止確認用）
#
# 前提条件:
#   OPS-PROC-010 §2 の前提条件チェックリストを事前に確認すること。
#   本スクリプトは system_admin のみが実行できること。

set -euo pipefail

# 設定値（環境変数で上書き可能）
PGDATA="${PGDATA:-/var/lib/postgresql/data}"
POSTGRES_USER="${POSTGRES_USER:-postgres}"
POSTGRES_DB="${POSTGRES_DB:-postgres}"
PRIMARY_HOST="${PRIMARY_HOST:-wnav-primary}"
SCRIPT_DIR="$(dirname "$0")"
CONFIG_DIR="${SCRIPT_DIR}/../config"

# フェイルオーバー開始時刻を記録する（RTO 計測用）
T0=$(date +%s)
FAILOVER_LOG="/tmp/failover-$(date +%Y%m%d%H%M%S).log"
echo "FAILOVER_START=$(date -d @${T0} '+%Y-%m-%d %H:%M:%S')" | tee "${FAILOVER_LOG}"
echo "[$(date -u +%Y-%m-%dT%H:%M:%SZ)] OPS-PROC-010: フェイルオーバー開始" | tee -a "${FAILOVER_LOG}"

# ----- Step 1: プライマリ停止を確認する -----
echo "[$(date -u +%Y-%m-%dT%H:%M:%SZ)] Step 1: プライマリ停止確認中..." | tee -a "${FAILOVER_LOG}"
if ping -c 3 "${PRIMARY_HOST}" > /dev/null 2>&1; then
    echo "エラー: プライマリ ${PRIMARY_HOST} に到達可能です。" | tee -a "${FAILOVER_LOG}"
    echo "  プライマリが停止していない場合はフェイルオーバーを実施しないでください。" | tee -a "${FAILOVER_LOG}"
    echo "  プライマリが実際に停止していることを確認してから再実行してください。" | tee -a "${FAILOVER_LOG}"
    exit 1
fi
echo "[$(date -u +%Y-%m-%dT%H:%M:%SZ)] Step 1: プライマリ停止確認 OK（ping 失敗）" | tee -a "${FAILOVER_LOG}"

# ----- Step 2: standby.signal を削除してリカバリモードを終了する -----
echo "[$(date -u +%Y-%m-%dT%H:%M:%SZ)] Step 2: standby.signal 削除中..." | tee -a "${FAILOVER_LOG}"
STANDBY_SIGNAL="${PGDATA}/standby.signal"
if [ -f "${STANDBY_SIGNAL}" ]; then
    rm -f "${STANDBY_SIGNAL}"
    echo "[$(date -u +%Y-%m-%dT%H:%M:%SZ)] Step 2: standby.signal 削除完了" | tee -a "${FAILOVER_LOG}"
else
    echo "[$(date -u +%Y-%m-%dT%H:%M:%SZ)] Step 2: standby.signal が存在しません（既にプライマリ状態の可能性）" | tee -a "${FAILOVER_LOG}"
fi

# ----- Step 3: recovery.conf.template を postgresql.conf に追記する -----
echo "[$(date -u +%Y-%m-%dT%H:%M:%SZ)] Step 3: リカバリ設定を postgresql.conf に適用中..." | tee -a "${FAILOVER_LOG}"
RECOVERY_TEMPLATE="${CONFIG_DIR}/recovery.conf.template"
POSTGRESQL_CONF="${PGDATA}/postgresql.conf"

if [ ! -f "${RECOVERY_TEMPLATE}" ]; then
    echo "エラー: recovery.conf.template が見つかりません: ${RECOVERY_TEMPLATE}" | tee -a "${FAILOVER_LOG}"
    exit 1
fi

# 既にリカバリ設定が追記されている場合は重複追記を避ける
if grep -q "restore_command" "${POSTGRESQL_CONF}" 2>/dev/null; then
    echo "[$(date -u +%Y-%m-%dT%H:%M:%SZ)] Step 3: リカバリ設定は既に postgresql.conf に存在します（スキップ）" | tee -a "${FAILOVER_LOG}"
else
    # recovery.conf.template の内容を postgresql.conf に追記する
    echo "" >> "${POSTGRESQL_CONF}"
    echo "# フェイルオーバー設定（$(date -u +%Y-%m-%dT%H:%M:%SZ) に追記）" >> "${POSTGRESQL_CONF}"
    grep -v '^#' "${RECOVERY_TEMPLATE}" | grep -v '^$' >> "${POSTGRESQL_CONF}"
    echo "[$(date -u +%Y-%m-%dT%H:%M:%SZ)] Step 3: リカバリ設定の追記完了" | tee -a "${FAILOVER_LOG}"
fi

# ----- Step 4: pg_ctl promote でプライマリに昇格させる -----
echo "[$(date -u +%Y-%m-%dT%H:%M:%SZ)] Step 4: pg_ctl promote 実行中..." | tee -a "${FAILOVER_LOG}"
pg_ctl promote -D "${PGDATA}" -W 2>&1 | tee -a "${FAILOVER_LOG}"
echo "[$(date -u +%Y-%m-%dT%H:%M:%SZ)] Step 4: pg_ctl promote 完了" | tee -a "${FAILOVER_LOG}"

# ----- Step 5: 昇格確認（pg_isready + 書き込み可否テスト） -----
echo "[$(date -u +%Y-%m-%dT%H:%M:%SZ)] Step 5: 昇格確認中..." | tee -a "${FAILOVER_LOG}"

# pg_isready でデータベースが接続を受け付けているか確認する（最大 60 秒待機する）
for i in $(seq 1 12); do
    if pg_isready -U "${POSTGRES_USER}" -d "${POSTGRES_DB}" > /dev/null 2>&1; then
        echo "[$(date -u +%Y-%m-%dT%H:%M:%SZ)] Step 5: pg_isready OK（${i} 回目）" | tee -a "${FAILOVER_LOG}"
        break
    fi
    if [ "${i}" -eq 12 ]; then
        echo "エラー: データベースが 60 秒以内に起動しませんでした。" | tee -a "${FAILOVER_LOG}"
        exit 1
    fi
    sleep 5
done

# 書き込み可否テスト（Standby は読み取り専用のため、書き込みが可能になっていることを確認する）
WRITE_TEST=$(psql -U "${POSTGRES_USER}" -d "${POSTGRES_DB}" \
    --tuples-only --no-align \
    -c "SELECT pg_is_in_recovery();" 2>/dev/null || echo "error")

if [ "${WRITE_TEST}" = "f" ]; then
    echo "[$(date -u +%Y-%m-%dT%H:%M:%SZ)] Step 5: 昇格確認 OK（pg_is_in_recovery = false: プライマリ状態）" | tee -a "${FAILOVER_LOG}"
else
    echo "エラー: pg_is_in_recovery が true のままです。昇格に失敗しました。" | tee -a "${FAILOVER_LOG}"
    exit 1
fi

# RTO 計測終了
T1=$(date +%s)
RTO_SEC=$((T1 - T0))
RTO_MIN=$(echo "scale=1; ${RTO_SEC}/60" | bc)
RTO_RESULT=$([ "${RTO_SEC}" -le 3600 ] && echo "PASS" || echo "FAIL")

echo "FAILOVER_END=$(date -d @${T1} '+%Y-%m-%d %H:%M:%S')" | tee -a "${FAILOVER_LOG}"
echo "RTO_SEC=${RTO_SEC}" | tee -a "${FAILOVER_LOG}"
echo "RTO_MIN=${RTO_MIN}" | tee -a "${FAILOVER_LOG}"
echo "RTO_RESULT=${RTO_RESULT}" | tee -a "${FAILOVER_LOG}"

echo "[$(date -u +%Y-%m-%dT%H:%M:%SZ)] フェイルオーバー完了"
echo "  RTO: ${RTO_MIN} 分（${RTO_RESULT}）"
echo "  ログ: ${FAILOVER_LOG}"
echo ""
echo "次のステップ（OPS-PROC-010 §4.4〜§4.9）:"
echo "  1. API コンテナを起動する: docker compose up -d terminal-api master-api"
echo "  2. IIS upstream を Standby に切り替える（PowerShell で実施）"
echo "  3. ハンディ端末の疎通確認"
echo "  4. ハッシュチェーン整合性確認（verify_hash_chain.sql を実行する）"
echo "  5. maintenance_log に記録する"
