#!/usr/bin/env bash
# 対応 §: ロードマップ §31 §31.5 §27 §29
# `docs/04_運用/slo-dashboard.md` の SLI／SLO ↔ FMEA／risk-register の双方向リンクを検査する。
# 孤児な SLI／SLO（紐付き 0 件）を警告する。

set -euo pipefail

# 対象ファイル
SLO="docs/04_運用/slo-dashboard.md"
FMEA="docs/04_運用/fmea.md"
RISK="docs/04_運用/risk-register.md"

# ファイル存在確認
for f in "$SLO" "$FMEA" "$RISK"; do
  if [ ! -f "$f" ]; then
    echo "observability-link-lint: ${f} が存在しないためスキップ"
    exit 0
  fi
done

# 違反件数
violations=0

# SLO ファイルから SLI-XX の ID を抽出
mapfile -t sli_ids < <(grep -oE 'SLI-[0-9]{2}' "$SLO" | sort -u)

# 各 SLI が FMEA か risk-register のいずれかに登場するか確認する
for sli in "${sli_ids[@]}"; do
  in_fmea=$(grep -c "$sli" "$FMEA" 2>/dev/null || true)
  # bash の grep -c は非マッチで 0 を返す
  in_fmea=${in_fmea:-0}
  in_risk=$(grep -c "$sli" "$RISK" 2>/dev/null || true)
  in_risk=${in_risk:-0}
  if [ "$in_fmea" -eq 0 ] && [ "$in_risk" -eq 0 ]; then
    echo "::warning file=${SLO}::${sli} が FMEA／risk-register に紐付いていません（§31.5）"
    violations=$((violations + 1))
  fi
done

# 集計
if [ "$violations" -gt 0 ]; then
  echo "observability-link-lint: ${violations} 件の孤児 SLI"
  if [ "${STRICT:-0}" = "1" ]; then
    exit 1
  fi
else
  echo "observability-link-lint: OK (${#sli_ids[@]} SLI 検査)"
fi
exit 0
