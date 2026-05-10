#!/usr/bin/env bash
# 対応 §: ロードマップ §9.4 §9.6 ／ ルート CLAUDE.md
# 「1 ファイル 500 行上限」を強制する CI lint。
# Rust／TypeScript／TSX／シェルスクリプトを対象とし、
# 自動生成ファイル（target／node_modules／dist 等）を除外する。

# シェルオプション: 失敗即時停止／未定義変数検出／パイプ失敗検出
set -euo pipefail

# 対象拡張子（CLAUDE.md「プログラムのソースファイル」に該当するもの）
EXTENSIONS="rs|ts|tsx|jsx|js|sh"
# 上限行数（CLAUDE.md / §9.4）
LIMIT=500
# 違反件数の集計用
violations=0

# 検査対象ファイルを列挙する
# - 自動生成・依存ディレクトリは除外する
# - シンボリックリンクを辿らない
mapfile -t files < <(find . \
  \( -path './node_modules' -o -path './**/node_modules' \
     -o -path './target' -o -path './**/target' \
     -o -path './dist' -o -path './**/dist' \
     -o -path './.git' \) -prune -o \
  -type f -regextype posix-extended -regex ".*\.(${EXTENSIONS})$" -print)

# 各ファイルの行数を確認する
for f in "${files[@]}"; do
  # 行数を取得する
  lines=$(wc -l < "$f")
  # 上限超を検出
  if [ "$lines" -gt "$LIMIT" ]; then
    # 違反を出力
    echo "::error file=${f}::${lines} lines exceeds ${LIMIT}-line limit (CLAUDE.md / §9.4)"
    # カウンタを進める
    violations=$((violations + 1))
  fi
done

# 集計結果
if [ "$violations" -gt 0 ]; then
  # 違反あり
  echo "lint-file-size: ${violations} violation(s) detected"
  exit 1
fi

# 違反なし
echo "lint-file-size: OK (limit=${LIMIT})"
exit 0
