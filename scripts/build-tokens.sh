#!/usr/bin/env bash
# 対応 §: ロードマップ §9.5.1
# Style Dictionary（Amazon, 2018）形式の design tokens を、
# 各プラットフォームのソースへ展開する。
# 実行には npx style-dictionary が必要（`pnpm add -D style-dictionary` を将来追加）。
# 本セッションでは npx の存在を確認し、未導入時は警告のみ。

set -euo pipefail

# トークン入力ディレクトリ
TOKEN_DIR="docs/02_設計/design-tokens"
# 出力ディレクトリ（端末アプリ／設定 UI が import するため apps/*/src/tokens/ に展開）
OUT_TS_TERMINAL="apps/terminal/src/tokens"
OUT_TS_CONFIG="apps/config-ui/src/tokens"

# 入力存在確認
if [ ! -d "$TOKEN_DIR" ]; then
  echo "build-tokens: ${TOKEN_DIR} が存在しないため終了"
  exit 1
fi

# 出力ディレクトリを作成
mkdir -p "$OUT_TS_TERMINAL" "$OUT_TS_CONFIG"

# JSON ファイルから TS const へ素朴変換する関数（npx 不在時のフォールバック）
# 各 JSON を `tokens.ts` の `export const TOKENS = { ... }` として組み立てる。
fallback_convert() {
  local out_dir="$1"
  local out_file="$out_dir/tokens.ts"
  echo "// 自動生成（scripts/build-tokens.sh フォールバック）— 編集禁止" > "$out_file"
  echo "// 対応 §: ロードマップ §9.5.1" >> "$out_file"
  echo "// 入力: ${TOKEN_DIR}/" >> "$out_file"
  echo "" >> "$out_file"
  echo "export const TOKENS = {" >> "$out_file"
  for json in "$TOKEN_DIR"/*.json; do
    name=$(basename "$json" .json)
    echo "  ${name}:" >> "$out_file"
    cat "$json" >> "$out_file"
    echo "  ," >> "$out_file"
  done
  echo "} as const;" >> "$out_file"
  echo "build-tokens: フォールバックで ${out_file} を生成しました"
}

# npx の存在確認
if command -v npx >/dev/null 2>&1; then
  # Style Dictionary 設定が無い場合はフォールバックを使う
  if [ -f "style-dictionary.config.cjs" ]; then
    npx style-dictionary build --config ./style-dictionary.config.cjs
    echo "build-tokens: Style Dictionary でビルドしました"
  else
    echo "build-tokens: style-dictionary.config.cjs 未整備のためフォールバック実行"
    fallback_convert "$OUT_TS_TERMINAL"
    fallback_convert "$OUT_TS_CONFIG"
  fi
else
  echo "build-tokens: npx 未導入のためフォールバック実行"
  fallback_convert "$OUT_TS_TERMINAL"
  fallback_convert "$OUT_TS_CONFIG"
fi

exit 0
