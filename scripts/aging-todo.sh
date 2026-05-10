#!/usr/bin/env bash
# 対応 §: ロードマップ §9.6
# TODO／FIXME の経過日数を git blame ベースで検出し、90 日超を警告する。

set -euo pipefail

# 上限日数
LIMIT_DAYS="${LIMIT_DAYS:-90}"

# 検出パターン
PATTERN='TODO|FIXME|XXX'

# 対象拡張子
EXTENSIONS_RE='\.(rs|ts|tsx|jsx|js|sh|md)$'

# 違反件数
violations=0

# git の有無
if ! command -v git >/dev/null 2>&1; then
  echo "aging-todo: git が見つからないためスキップ"
  exit 0
fi

# 対象ファイルを列挙
mapfile -t files < <(find . \
  \( -path './node_modules' -o -path './target' -o -path './dist' -o -path './.git' \) -prune -o \
  -type f -regextype posix-extended -regex ".*${EXTENSIONS_RE}" -print)

# 各ファイルで TODO/FIXME 行を検出
for f in "${files[@]}"; do
  while IFS=: read -r lineno _; do
    [ -z "$lineno" ] && continue
    # blame で当該行の最終変更日（YYYY-MM-DD）を取得
    blame_line=$(git blame -L "${lineno},${lineno}" --date=short --line-porcelain "$f" 2>/dev/null | \
      awk '/^author-time/ { t=$2 } END { print t+0 }')
    [ -z "$blame_line" ] && continue
    # 経過秒
    now=$(date +%s)
    age=$(( (now - blame_line) / 86400 ))
    if [ "$age" -gt "$LIMIT_DAYS" ]; then
      echo "::warning file=${f},line=${lineno}::TODO/FIXME が ${age} 日経過しています（上限 ${LIMIT_DAYS} 日、§9.6）"
      violations=$((violations + 1))
    fi
  done < <(grep -nE "$PATTERN" "$f" || true)
done

# 集計
if [ "$violations" -gt 0 ]; then
  echo "aging-todo: ${violations} 件の経過超過"
  if [ "${STRICT:-0}" = "1" ]; then
    exit 1
  fi
else
  echo "aging-todo: OK"
fi
exit 0
