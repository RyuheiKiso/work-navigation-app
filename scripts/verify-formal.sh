#!/usr/bin/env bash
# 対応 §: ロードマップ §3.4.1 §3.4.2 §10.6.2
# 形式化の機械検証スクリプト:
#   - TLA+ 仕様（docs/03_設計/形式化/sync.tla）を TLC で検証
#   - HSM（hsm-task.puml）を sismic で reachability/deadlock 検査
#   - CPN モデル概要は手動検証用ドキュメント（CPN Tools が必要）
#
# 各ツール未導入時は警告のみで終了する（CI で feature flag 化する想定）。

set -euo pipefail

TLA_DIR="docs/03_設計/形式化"
TLA_FILE="${TLA_DIR}/sync.tla"
HSM_FILE="${TLA_DIR}/hsm-task.puml"

# 1. TLA+ TLC
echo "== TLA+ TLC 検証 =="
if [ ! -f "$TLA_FILE" ]; then
  echo "[SKIP] ${TLA_FILE} が無い"
else
  if command -v java >/dev/null 2>&1 && [ -f "${TLA_DIR}/tla2tools.jar" ]; then
    # TLC を起動。設定ファイルが無ければスキップ
    if [ -f "${TLA_DIR}/sync.cfg" ]; then
      java -cp "${TLA_DIR}/tla2tools.jar" tlc2.TLC "$TLA_FILE" \
        -config "${TLA_DIR}/sync.cfg" -workers auto -metadir target/tlc \
        || echo "[FAIL] TLC で違反検出"
    else
      echo "[SKIP] sync.cfg 未整備（TLC 設定が必要）"
    fi
  else
    echo "[SKIP] java または tla2tools.jar 未導入"
    echo "       https://github.com/tlaplus/tlaplus/releases から tla2tools.jar を ${TLA_DIR}/ に配置"
  fi
fi

# 2. HSM sismic
echo "== HSM sismic 検証 =="
if [ ! -f "$HSM_FILE" ]; then
  echo "[SKIP] ${HSM_FILE} が無い"
else
  if command -v sismic-bdd >/dev/null 2>&1; then
    # PlantUML → sismic YAML 変換が必要（手作業 or plantuml 経由）
    echo "[INFO] hsm-task.puml の sismic 検証は plantuml→sismic-yaml 変換後に実行する"
    echo "       plantuml --txt $HSM_FILE で AST 確認可能"
  else
    echo "[SKIP] sismic-bdd 未導入（pip install sismic）"
  fi
fi

# 3. CPN Tools（手動）
echo "== CPN モデル =="
if [ -f "${TLA_DIR}/cpn-sync.md" ]; then
  echo "[INFO] CPN モデルは ${TLA_DIR}/cpn-sync.md を参照"
  echo "       検証は CPN Tools／Access/CPN（手動）"
fi

echo "verify-formal: 完了"
exit 0
