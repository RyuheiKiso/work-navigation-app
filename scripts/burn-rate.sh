#!/usr/bin/env bash
# 対応 §: ロードマップ §31.3 §31.4
# エラー予算バーンレート計算スクリプト。
# Google SRE Workbook 6.5 章のマルチウィンドウ・マルチバーンレート手法。
#
# 使用法:
#   PROM_URL=http://localhost:9090 SLO=0.995 bash scripts/burn-rate.sh
#
# SLO は 30 日窓のサービスレベル目標（例: 0.995 = 99.5%）
# 出力: 短期／中期／長期バーンレートと閾値判定

set -euo pipefail

PROM_URL="${PROM_URL:-http://localhost:9090}"
SLO="${SLO:-0.995}"
SLI_QUERY="${SLI_QUERY:-wna_terminal_screen_transition_success_ratio}"

# Prometheus 即値 query
prom_query() {
  local q="$1"
  curl -fsS -G --data-urlencode "query=${q}" "${PROM_URL}/api/v1/query" 2>/dev/null \
    | python3 -c "import sys, json; d=json.load(sys.stdin); r=d['data']['result']; print(r[0]['value'][1] if r else '0')"
}

# エラー率 = 1 - SLI 達成率
calc_error_rate() {
  local window="$1"
  local sli
  # 当該ウィンドウでの達成率を取得（avg_over_time）
  sli=$(prom_query "avg_over_time(${SLI_QUERY}[${window}])")
  python3 -c "print(1 - ${sli:-0})"
}

# バーンレート = エラー率 / (1 - SLO)
calc_burn_rate() {
  local error_rate="$1"
  local budget
  budget=$(python3 -c "print(1 - ${SLO})")
  python3 -c "br = ${error_rate} / ${budget} if ${budget} > 0 else 0; print(br)"
}

echo "== バーンレート計算（SLO=${SLO}, SLI=${SLI_QUERY}）=="

# 短期（5 分／1 時間）— 14.4× で短期アラート
e_5m=$(calc_error_rate "5m")
e_1h=$(calc_error_rate "1h")
br_5m=$(calc_burn_rate "$e_5m")
br_1h=$(calc_burn_rate "$e_1h")
echo "短期 5m バーンレート: ${br_5m}"
echo "短期 1h バーンレート: ${br_1h}"

# 中期（30 分／6 時間）— 6× で中期アラート
e_30m=$(calc_error_rate "30m")
e_6h=$(calc_error_rate "6h")
br_30m=$(calc_burn_rate "$e_30m")
br_6h=$(calc_burn_rate "$e_6h")
echo "中期 30m バーンレート: ${br_30m}"
echo "中期 6h バーンレート: ${br_6h}"

# 長期（3 日）— 1× で長期アラート
e_3d=$(calc_error_rate "3d")
br_3d=$(calc_burn_rate "$e_3d")
echo "長期 3d バーンレート: ${br_3d}"

# 判定
echo "== アラート判定 =="

# 短期: 5m AND 1h が共に 14.4 以上
if python3 -c "exit(0 if (${br_5m} >= 14.4 and ${br_1h} >= 14.4) else 1)"; then
  echo "[CRITICAL] 短期バーンレートアラート発火（§22.2 即時トリガー）"
  exit 2
fi

# 中期: 30m AND 6h が共に 6 以上
if python3 -c "exit(0 if (${br_30m} >= 6 and ${br_6h} >= 6) else 1)"; then
  echo "[WARNING] 中期バーンレートアラート発火（§22.4 是正フロー）"
  exit 1
fi

# 長期: 3d が 1 以上
if python3 -c "exit(0 if (${br_3d} >= 1) else 1)"; then
  echo "[INFO] 長期バーンレートアラート（エラー予算 50% 超過の予兆）"
  exit 0
fi

echo "[OK] 全バーンレートが閾値以下"
exit 0
