#!/usr/bin/env bash
# 対応 §: ロードマップ §9.4 §9.6 §13.2 ／ ルート CLAUDE.md
# CLAUDE.md「コメント方針（WHY を一行で）」を機械的に検証する最小 lint。
# 本スクリプトの設計意図:
# - WHY/WHAT 弁別は意味解析を要し、機械化は誤検知の温床になる。
#   よって厳格な意味検査ではなく「最低限のドキュメンテーション衛生」を担保する。
#   つまり、ソース冒頭に doc コメントが在ること、明らかな placeholder が
#   残っていないことを確認する。
# - 既存リポジトリ全体に対し誤検知ゼロで通ることを設計目標とする。
# - 対象は Rust / TypeScript / TSX / JS / Shell。テスト・ビルド・依存物は除外。

set -euo pipefail

EXTENSIONS="rs|ts|tsx|jsx|js|sh"
violations=0

# 検査対象ファイルを列挙（自動生成・依存・ビルド成果物を除外）
mapfile -t files < <(find . \
  \( -path './node_modules' -o -path './**/node_modules' \
     -o -path './target' -o -path './**/target' \
     -o -path './dist' -o -path './**/dist' \
     -o -path './**/build' \
     -o -path './.git' \) -prune -o \
  -type f -regextype posix-extended -regex ".*\.(${EXTENSIONS})$" -print)

# 先頭 5 行のいずれかが doc コメント (非空) かを判定する。
# - Shell: `#` 始まり
# - Rust/TS/JS/TSX: `//` 始まり、または Rust の `/*!` `///` `//!`
# - shebang (#!) はカウントしない
has_top_doc() {
  local file="$1"
  awk '
    NR<=5 {
      if (substr($0,1,2)=="#!") next
      gsub(/^[[:space:]]+/, "", $0)
      if ($0 ~ /^\/\// || $0 ~ /^\/\*/ || $0 ~ /^#/) {
        body=$0
        sub(/^[\/#*!]+[[:space:]]*/, "", body)
        if (length(body) > 0) { found=1 }
      }
    }
    END { exit (found ? 0 : 1) }
  ' "$file"
}

# placeholder / no-information の代表例。
# 現状 codebase で一致しないパターンに限定（誤検知ゼロを維持）。
forbidden_re='^(\s*//|\s*#)\s*(TBD|TODO 何か書く|FIXME later|placeholder)\s*$'

while IFS= read -r f; do
  # テストファイルは doc コメント要件の対象外
  case "$f" in
    *.test.ts|*.test.tsx|*.test.js|*.test.jsx)
      ;;
    *)
      if ! has_top_doc "$f"; then
        echo "::error file=${f}::module-level doc comment missing in first 5 lines (CLAUDE.md)"
        violations=$((violations + 1))
      fi
      ;;
  esac

  # placeholder コメントの検出
  if grep -nE "$forbidden_re" "$f" >/dev/null 2>&1; then
    while IFS= read -r m; do
      echo "::error file=${f},line=${m%%:*}::placeholder comment forbidden (CLAUDE.md)"
      violations=$((violations + 1))
    done < <(grep -nE "$forbidden_re" "$f")
  fi
done < <(printf "%s\n" "${files[@]}")

if [ "$violations" -gt 0 ]; then
  echo "lint-line-comments: ${violations} violation(s) detected"
  exit 1
fi

echo "lint-line-comments: OK (files=${#files[@]})"
exit 0
