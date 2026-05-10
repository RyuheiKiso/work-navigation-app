#!/usr/bin/env bash
# 対応 §: ロードマップ §13.4.1 §9.6
# ミューテーションテスト実行スクリプト。Rust ドメイン層（cargo-mutants）と
# TypeScript フロント（stryker）の両方を実行する。

set -euo pipefail

# 出力先
REPORT_DIR="docs/04_運用"
YEAR_MONTH=$(date -u +%Y-%m)
RUST_REPORT="${REPORT_DIR}/mutation-report-${YEAR_MONTH}.md"

# Rust ミューテーションテスト
echo "== Rust ミューテーションテスト =="
if command -v cargo-mutants >/dev/null 2>&1; then
  cargo mutants \
    --workspace \
    --package wna-domain \
    --package wna-usecase \
    --output "${REPORT_DIR}/mutants-${YEAR_MONTH}/" \
    --timeout 60 \
    --json \
    || echo "cargo-mutants が一部 mutation を生存させました（要テスト追加）"
  echo "Rust mutation report: ${REPORT_DIR}/mutants-${YEAR_MONTH}/"
else
  echo "cargo-mutants 未導入。'cargo install cargo-mutants' でインストールしてください"
fi

# TypeScript ミューテーションテスト
echo "== TypeScript ミューテーションテスト =="
if [ -f "stryker.config.json" ]; then
  pnpm exec stryker run || echo "Stryker が一部 mutation を生存させました（要テスト追加）"
else
  echo "stryker.config.json 未整備。手順は §13.4.1 を参照"
fi

# 雛形レポートを書き出す
mkdir -p "${REPORT_DIR}"
if [ ! -f "${RUST_REPORT}" ]; then
  cat > "${RUST_REPORT}" <<EOF
# ミューテーションレポート（${YEAR_MONTH}）

> 対応 §: ロードマップ §13.4.1 §9.6
> 実行: $(date -u +%Y-%m-%dT%H:%M:%SZ)

## サマリ

| 対象 | スコア | 目標 | 合否 |
| --- | --- | --- | --- |
| Rust ドメイン層 | （cargo-mutants 結果） | ≥ 80% | TBD |
| TypeScript フロント | （stryker 結果） | ≥ 70% | TBD |

詳細: \`mutants-${YEAR_MONTH}/\` 配下の JSON／HTML を参照。
EOF
fi

echo "mutation-test: 完了（${RUST_REPORT}）"
