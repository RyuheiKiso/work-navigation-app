#!/usr/bin/env bash
# 対応 §: ロードマップ §19.3 §32.5
# リリース成果物の cosign 署名（OIDC keyless）。
# 本スクリプトは GitHub Actions の OIDC トークンを利用する想定。
# ローカルで実行する場合は cosign sign-blob の対話モードを使う。

set -euo pipefail

# 入力アーティファクト
ARTIFACTS=(
  "dist/wna-backend-*.tar.gz"
  "dist/work-navigation-app-*.apk"
  "dist/work-navigation-app-*.msi"
  "dist/sbom/*.cdx.json"
)

# 署名ディレクトリ
SIG_DIR="dist/signatures"
mkdir -p "$SIG_DIR"

if ! command -v cosign >/dev/null 2>&1; then
  echo "cosign 未導入。https://github.com/sigstore/cosign からインストールしてください"
  exit 1
fi

echo "== cosign 署名（OIDC keyless）=="

# 各アーティファクトを署名
for pattern in "${ARTIFACTS[@]}"; do
  for f in $pattern; do
    [ -e "$f" ] || continue
    base=$(basename "$f")
    sig="${SIG_DIR}/${base}.sig"
    cert="${SIG_DIR}/${base}.cert"
    echo "[sign] ${f}"
    # OIDC keyless 署名（GitHub Actions 環境で動く）
    COSIGN_EXPERIMENTAL=1 cosign sign-blob \
      --yes \
      --output-signature "$sig" \
      --output-certificate "$cert" \
      "$f"
    echo "  -> ${sig}"
  done
done

echo "== 署名検証ヘルパ（受領側用）=="
cat > "${SIG_DIR}/VERIFY.md" <<'EOF'
# 署名検証手順

```bash
# 各アーティファクトに対して:
COSIGN_EXPERIMENTAL=1 cosign verify-blob \
  --signature signatures/<artifact>.sig \
  --certificate signatures/<artifact>.cert \
  --certificate-identity "https://github.com/RyuheiKiso/work-navigation-app/.github/workflows/release.yml@refs/tags/<TAG>" \
  --certificate-oidc-issuer "https://token.actions.githubusercontent.com" \
  <artifact>
```

OIDC issuer と identity は §19.3 のリリース署名鍵管理方針と整合する。
EOF

echo "sign-release: 完了 → ${SIG_DIR}/"
