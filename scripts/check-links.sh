#!/usr/bin/env bash
# 対応 §: ロードマップ §18.5.2
# Markdown 内の相対リンク／脚注リンクの切れを検出する。
# 外部 URL（http/https）は §18.5.2「リンク切れ」検出として記録のみで、ネットワーク失敗は致命扱いしない。

set -euo pipefail

# 違反件数
violations=0
# 外部リンク警告件数
warnings=0

# 対象 Markdown ファイル
mapfile -t files < <(find . \
  \( -path './node_modules' -o -path './target' -o -path './.git' \) -prune -o \
  -type f -name '*.md' -print)

for f in "${files[@]}"; do
  # 行ごとにリンクを抽出
  while IFS=: read -r lineno content; do
    [ -z "$content" ] && continue
    # 一行から複数のリンクを抜く（簡易）
    while read -r link; do
      [ -z "$link" ] && continue
      # 外部リンクの場合は警告のみ
      if [[ "$link" =~ ^https?:// ]]; then
        # ネットワーク到達は確認しない（CI 高速化）
        # §18.5.2 で別途 daily ジョブで実施想定
        warnings=$((warnings + 1))
        continue
      fi
      # フラグメント／メールリンクなど対象外
      if [[ "$link" =~ ^# ]] || [[ "$link" =~ ^mailto: ]]; then
        continue
      fi
      # ファイルパス先頭を切り出す（# 以降は除去）
      target="${link%%#*}"
      [ -z "$target" ] && continue
      # 相対パスの場合は所在ファイルからの相対を解決
      dir=$(dirname "$f")
      if [[ "$target" =~ ^/ ]]; then
        # ルート絶対参照（リポジトリルート起点と解釈）
        resolved=".${target}"
      else
        resolved="${dir}/${target}"
      fi
      # 存在チェック
      if ! [ -e "$resolved" ]; then
        echo "::error file=${f},line=${lineno}::相対リンク切れ: ${link} (resolved=${resolved})"
        violations=$((violations + 1))
      fi
    done < <(echo "$content" | grep -oE '\]\([^)]+\)' | sed -E 's/^\]\(([^)]+)\)$/\1/' || true)
  done < <(grep -nE '\]\([^)]+\)' "$f" || true)
done

# 集計
if [ "$violations" -gt 0 ]; then
  echo "check-links: ${violations} 件のリンク切れ"
  exit 1
fi
echo "check-links: OK (外部リンク警告 ${warnings} 件は確認のみ)"
exit 0
