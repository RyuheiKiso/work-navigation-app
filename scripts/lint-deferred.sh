#!/usr/bin/env bash
# 対応 §: ロードマップ §33 §33.4
# `docs/adr/deferred/` 配下の DEFER- 保留 ID の再評価期日超過を検出する。
# 各ファイルは「再評価期日」行を含むものとし、ISO 8601（YYYY-MM-DD）日付をパースする。

set -euo pipefail

# 対象ディレクトリ
DEFER_DIR="docs/adr/deferred"

# 違反件数
violations=0

# ディレクトリが存在しなければ終了
if [ ! -d "$DEFER_DIR" ]; then
  echo "lint-deferred: ${DEFER_DIR} が存在しないためスキップ"
  exit 0
fi

# 現在日（UTC ISO 8601）
today=$(date -u +%Y-%m-%d)

# DEFER- ファイルを検出
for f in "$DEFER_DIR"/DEFER-*.md; do
  [ -e "$f" ] || continue
  # 再評価期日の行を抽出（書式: `再評価期日: YYYY-MM-DD`）
  due=$(grep -E '再評価期日[:：]' "$f" | head -n 1 | grep -oE '[0-9]{4}-[0-9]{2}-[0-9]{2}' || true)
  if [ -z "$due" ]; then
    echo "::warning file=${f}::再評価期日が記載されていません（§33.2）"
    violations=$((violations + 1))
    continue
  fi
  # 文字列比較で日付の前後を判定（ISO 8601 は辞書順で比較可能）
  if [[ "$due" < "$today" ]]; then
    echo "::error file=${f}::再評価期日（${due}）が現在日（${today}）を超過しています（§33.4）"
    violations=$((violations + 1))
  fi
done

# 集計
if [ "$violations" -gt 0 ]; then
  echo "lint-deferred: ${violations} 件の問題"
  exit 1
fi
echo "lint-deferred: OK"
exit 0
