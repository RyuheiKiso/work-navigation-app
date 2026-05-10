#!/usr/bin/env bash
# 対応 §: ロードマップ §2.2 §18.5
# 「決定語」を含む新規追記行に出典（§番号または URL）が無い場合に警告する。
# git diff の追加行を対象とし、PR 文脈で実行する想定。

set -euo pipefail

# ベースブランチ（環境変数で上書き可）
BASE="${BASE_REF:-origin/main}"

# 決定語のパターン
DECISION_RE='(初期決定|採用する|不採用|禁止する|必須とする)'

# 出典っぽいパターン
CITE_RE='(§[0-9]+(\.[0-9]+)*|https?://)'

# 対象拡張子（Markdown／コード）
EXTENSIONS_RE='\.(md|rs|ts|tsx|sh)$'

# 違反件数
violations=0

# git が利用不可なときはスキップ
if ! command -v git >/dev/null 2>&1; then
  echo "lint-rationale: git が見つからないためスキップ"
  exit 0
fi

# diff を取得（追加行のみ）
# `|| true` で 0 行 diff を許容する
mapfile -t lines < <(git diff --unified=0 "$BASE"...HEAD 2>/dev/null | \
  awk '
    /^\+\+\+/ { file = substr($0, 7); next }
    /^@@/ { next }
    /^\+/ {
      # 先頭の + を除いた本文
      content = substr($0, 2)
      printf("%s\t%s\n", file, content)
    }
  ' || true)

# 追加行を確認する
for entry in "${lines[@]}"; do
  file="${entry%%	*}"
  text="${entry#*	}"
  # 拡張子で対象を絞る
  if ! [[ "$file" =~ $EXTENSIONS_RE ]]; then
    continue
  fi
  # 決定語を含む？
  if ! echo "$text" | grep -Eq "$DECISION_RE"; then
    continue
  fi
  # 出典が同じ行にある？
  if echo "$text" | grep -Eq "$CITE_RE"; then
    continue
  fi
  # 違反として出力
  echo "::warning file=${file}::決定語を含む追加行に出典がありません: ${text}"
  violations=$((violations + 1))
done

# 集計
if [ "$violations" -gt 0 ]; then
  echo "lint-rationale: ${violations} 件の警告"
  if [ "${STRICT:-0}" = "1" ]; then
    exit 1
  fi
else
  echo "lint-rationale: OK"
fi
exit 0
