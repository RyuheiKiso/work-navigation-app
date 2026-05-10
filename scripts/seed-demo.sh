#!/usr/bin/env bash
# 対応 §: ロードマップ §14.2 §10.2.1
# デモシード一括投入スクリプト（旧版は psql 直接投入。現在は wna-backend CLI を経由する）。
# 内部で adapter 層の upsert を呼ぶため、ドメイン不変条件と同じ径路を通る。

set -euo pipefail

# プリセット指定（minimal / showcase）。既定は showcase（マスタ＋タスクまで投入）。
PRESET="${1:-showcase}"
# バックエンドコンテナ名（compose 既定）
BACKEND_CONTAINER="${BACKEND_CONTAINER:-wna-backend}"

echo "== work-navigation-app デモシード (preset=${PRESET}) =="

# 1. compose を起動（postgres + backend）
echo "[1/4] docker compose up -d"
docker compose up -d

# 2. backend の readiness を待機（/readyz が 200）
echo "[2/4] backend readiness 待機"
bash "$(dirname "$0")/wait-backend-healthy.sh"

# 3. シードを投入（compose 経由で wna-backend を実行）
echo "[3/4] wna-backend seed --preset ${PRESET}"
docker compose exec -T backend wna-backend seed --preset "${PRESET}"

# 4. ペアリング QR の生成（任意。スクリプト未生成なら警告のみ）
echo "[4/4] 端末ペアリング QR を出力"
bash "$(dirname "$0")/qr-pair.sh" terminal-001 || echo "（qr-pair.sh が未生成。スキップ）"

cat <<'EOF'

== ✓ デモシード完了 ==
ログイン情報:
  alice / hello-world   (オペレータ)
  bob / hello-world     (班長)
  charlie / hello-world (生産技術)

URL:
  端末:    http://localhost:1420
  設定UI:  http://localhost:1421
  バック:  http://localhost:8080

EOF
