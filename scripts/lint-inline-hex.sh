#!/usr/bin/env bash
# 対応 §: ロードマップ §9.5.1 §11.2 / docs/02_設計/設定UI監査.md §1.11
# JSX/TSX 内のインライン 16 進カラーを禁止する lint。
# 配色は tokens/access の palette / tone() / elevation を経由する規約 (§9.5.1) を機械的に強制。
# 逸脱すると dark/outdoor テーマ追従と「色のみ依存しない」設計 (SC 1.4.1) が崩れる。
# 例外: tokens/ 配下 (トークン定義そのもの)、テストファイル、コメント、cva-fallback の hex リテラル (`var(--…, #XXX)` は許可)。
#
# 運用方針: 現状トークン化が完了したファイル群 (flow-canvas / master-editor /
# lead-dashboard / audit-viewer / app-shell / confirm-dialog) のみを CI ゲートとし、
# 残ファイル (login-screen / error-panel / error-boundary / flow-editor / states/* /
# utils/flow-rf-mapping / terminal 側全般) は逐次トークン化してから CI へ追加する。
# `LINT_INLINE_HEX_SCOPE` 環境変数で対象ディレクトリを絞れる。未指定なら全 apps を走査。

set -uo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

# 走査対象。LINT_INLINE_HEX_SCOPE 未指定なら全 apps、指定時はそのパス配下のみ。
SCOPE="${LINT_INLINE_HEX_SCOPE:-apps}"

# 検出本体:
#   - .ts/.tsx 限定 (.css は別途 stylelint 想定)
#   - tokens/, node_modules/, dist/, coverage/ は除外
#   - .test.* は除外
#   - コメント行 (`//` `*` 始まり) は除外
#   - var(--…, #XXX) のフォールバックは暫定で許容
matches=$(
  grep -rEn '#[0-9A-Fa-f]{3,6}\b' \
    --include='*.ts' --include='*.tsx' \
    --exclude='*.test.ts' --exclude='*.test.tsx' \
    --exclude-dir=tokens --exclude-dir=node_modules --exclude-dir=dist --exclude-dir=coverage \
    "$SCOPE" 2>/dev/null \
  | grep -v -E ':[[:space:]]*//' \
  | grep -v -E ':[[:space:]]*\*' \
  | grep -v -E 'var\(--[^,)]+,[[:space:]]*#[0-9A-Fa-f]{3,6}' \
  || true
)

if [ -n "$matches" ]; then
  echo "$matches"
  echo ""
  echo "lint-inline-hex: 上記箇所で hex 直書きを検出しました。"
  echo "  apps/<app>/src/tokens/access の palette / tone() / elevation を経由してください。"
  echo "  CSS 変数フォールバック (var(--…, #XXX)) はテーマ未注入時の保険として暫定許可しています。"
  exit 1
fi

echo "lint-inline-hex: OK"
exit 0
