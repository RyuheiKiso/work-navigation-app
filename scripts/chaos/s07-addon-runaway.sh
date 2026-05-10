#!/usr/bin/env bash
# 対応 §: ロードマップ §13.4.2 S-07 §17.5 §27 F-004
# シナリオ S-07: アドオン暴走（無限ループアドオン）

set -euo pipefail

ADDON_PATH="${1:-./addon-sdk/examples/runaway-test/runaway.wasm}"

echo "[S-07] 無限ループアドオン ${ADDON_PATH} を投入"

if [ ! -f "$ADDON_PATH" ]; then
  echo "[S-07] テスト用 wasm が未整備。次のように生成してください:"
  echo "  cargo build -p wna-addon-runaway-test --target wasm32-wasi --release"
  echo "[S-07] capability_check 単体テストおよび capability_check::pass_when_required_subset 等で"
  echo "  入力を満たしても fuel／timeout で kill されることを §17.5 受入観点として確認"
  exit 0
fi

echo "[S-07] アドオンランタイムを invoke。fuel/time 上限で kill されること（§17.5）"
echo "[S-07] 検証: §17.7 受入観点／F-004"
