#!/usr/bin/env bash
# 対応 §: ロードマップ §11.4.3 §19.3
# CycloneDX 形式の SBOM を生成する。Rust／Node 双方を対象。

set -euo pipefail

OUT_DIR="dist/sbom"
mkdir -p "$OUT_DIR"

echo "== Rust SBOM =="
if command -v cargo-cyclonedx >/dev/null 2>&1; then
  cargo cyclonedx --all --format json --output-pattern "${OUT_DIR}/rust-{name}.cdx.json"
  echo "Rust SBOM 出力: ${OUT_DIR}/rust-*.cdx.json"
else
  echo "cargo-cyclonedx 未導入。'cargo install cargo-cyclonedx' でインストール"
fi

echo "== Node SBOM =="
if command -v cyclonedx-npm >/dev/null 2>&1; then
  cyclonedx-npm --output-format json --output-file "${OUT_DIR}/node.cdx.json"
  echo "Node SBOM 出力: ${OUT_DIR}/node.cdx.json"
elif command -v npx >/dev/null 2>&1; then
  npx --yes @cyclonedx/cyclonedx-npm --output-format json --output-file "${OUT_DIR}/node.cdx.json" || \
    echo "Node SBOM 生成失敗（@cyclonedx/cyclonedx-npm のインストールを検討）"
else
  echo "npm 系ツール未導入"
fi

echo "== Container SBOM（trivy）=="
if command -v trivy >/dev/null 2>&1; then
  trivy image --format cyclonedx --output "${OUT_DIR}/container.cdx.json" "${WNA_IMAGE:-wna-backend:latest}" || \
    echo "trivy SBOM 生成失敗（コンテナイメージ未ビルド？）"
else
  echo "trivy 未導入。コンテナ SBOM はスキップ"
fi

echo "generate-sbom: 完了 → ${OUT_DIR}/"
