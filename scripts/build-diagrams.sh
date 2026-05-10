#!/usr/bin/env bash
# 対応 §: ロードマップ §18.3 §23
# drawio ファイル（*.drawio）から PNG を書き出す。
# `drawio` CLI（drawio-desktop の `drawio --export` または `@drawio/cli`）が必要。
# 未導入時は警告のみで exit 0。

set -euo pipefail

# 入力ディレクトリ（複数）
INPUT_DIRS=(
  "docs/01_企画"
  "docs/02_設計"
  "docs/03_設計"
  "docs/04_運用"
)
# 違反件数
errors=0

# drawio CLI の存在確認
if ! command -v drawio >/dev/null 2>&1; then
  echo "build-diagrams: drawio CLI 未導入のためスキップ（§18.3 のとおり手動エクスポート）"
  exit 0
fi

# 各入力ディレクトリで *.drawio を処理
for dir in "${INPUT_DIRS[@]}"; do
  [ -d "$dir" ] || continue
  while IFS= read -r src; do
    # 出力ファイル
    out="${src%.drawio}.png"
    # ヘッドレスエクスポート
    if drawio --export --format png --output "$out" "$src" 2>/dev/null; then
      echo "build-diagrams: ${src} -> ${out}"
    else
      echo "::error file=${src}::drawio エクスポートに失敗しました"
      errors=$((errors + 1))
    fi
  done < <(find "$dir" -type f -name '*.drawio')
done

# 集計
if [ "$errors" -gt 0 ]; then
  echo "build-diagrams: ${errors} 件の失敗"
  exit 1
fi
echo "build-diagrams: OK"
exit 0
