# 07 四半期依存更新レビュー手順（OPS-PROC-007）

本手順書の責務はシステム全体の依存ライブラリ・OS・コンテナイメージの脆弱性状態と更新要否を確定することである。上流要件 NFR-OPS-040/041・MNT-017〜019（`docs/04_概要設計/08_運用方式設計/07_アカウント・変更管理と運用手順.md`）を手順に具体化する。IPA 共通フレーム 2013「4.2.1.c 業務及びシステムの運用」に準拠する。

---

## 1. 目的と上流要件

| 属性 | 内容 |
|---|---|
| **手順 ID** | OPS-PROC-007 |
| **頻度** | 四半期（1 月・4 月・7 月・10 月の第 1 営業日） |
| **想定所要時間** | P50: 45 分 / P95: 90 分 |
| **実施権限** | system_admin（必須） |

上流要件:
- NFR-OPS-040: 四半期ごとに全依存ライブラリの脆弱性スキャンを実施し結果を記録すること
- NFR-OPS-041: CVE 重大度別の対応 SLO を遵守すること（Critical 3 日・High 30 日・Medium 次回四半期・Low 年次）
- MNT-017: Rust crate の脆弱性は `cargo audit` で検出すること
- MNT-018: Node.js 依存の脆弱性は `pnpm audit` で検出すること
- MNT-019: Docker ベースイメージおよび OS パッチを四半期で確認すること

**本節で確定した方針**
- 四半期レビューは §4 の対象スキャンをすべて実施することを確定する。
- Critical（CVSS 9.0+）が検出された場合は即時 OPS-PROC-009 に委任することを確定する。
- スキャン結果はファイルに保存し `maintenance_log` に記録することを確定する。

---

## 2. 前提条件チェックリスト

以下をすべて確認してから手順を開始する。1 つでも NG なら手順を開始しない。

- [ ] system_admin として認証済みのターミナルセッションが確立されている
- [ ] インターネット接続が確立されている（CVE データベースの取得に必要）
- [ ] `cargo`・`pnpm`・`docker`・`Get-WindowsUpdate` コマンドが使用可能である
- [ ] スキャン結果の保存ディレクトリ `/opt/wnav/reports/audit/` が存在する
- [ ] 前回の四半期レビュー記録が `maintenance_log` に存在する

**本節で確定した方針**
- 前提条件チェックリストに 1 つでも NG がある場合は手順を開始しないことを確定する。

---

## 3. 事前準備

- [CMD]
  ```bash
  export QUARTER_DATE=$(date +%Y-Q$(( ($(date +%-m)-1)/3+1 )))
  export AUDIT_DIR="/opt/wnav/reports/audit"
  mkdir -p "${AUDIT_DIR}"
  export AUDIT_LOG="${AUDIT_DIR}/${QUARTER_DATE}-dependency-review.log"
  echo "=== Dependency Review: ${QUARTER_DATE} ===" | tee "${AUDIT_LOG}"
  echo "Start: $(date)" | tee -a "${AUDIT_LOG}"
  ```

**本節で確定した方針**
- 全スキャン結果を単一ログファイルに集約することを確定する。

---

## 4. 実施手順

以下の操作タグを使用する。
- `[CMD]` シェルコマンド（WSL2 + bash）
- `[SQL]` PostgreSQL クエリ（psql 経由）
- `[PS]` PowerShell（IIS / Windows Server 操作）
- `[GUI]` ブラウザ / Grafana / 管理 UI 操作
- `[CHECK]` 確認・検証操作

### 4.1 ステップ 1: Rust — cargo audit

- [CMD]
  ```bash
  echo "=== cargo audit ===" | tee -a "${AUDIT_LOG}"
  cd /opt/wnav/backend

  # CVE アドバイザリ DB を最新化
  cargo audit fetch 2>&1 | tee -a "${AUDIT_LOG}"

  # JSON 形式でスキャン実行
  cargo audit --json 2>&1 | tee "${AUDIT_DIR}/${QUARTER_DATE}-cargo-audit.json"

  # サマリを表示
  cargo audit 2>&1 | tee -a "${AUDIT_LOG}"
  echo "cargo audit exit: $?" | tee -a "${AUDIT_LOG}"
  ```

- [CMD] 最新バージョンとの比較
  ```bash
  echo "=== cargo outdated ===" | tee -a "${AUDIT_LOG}"
  cargo outdated --root-deps-only 2>&1 | tee -a "${AUDIT_LOG}"
  ```

- [CHECK] `cargo audit` の出力で `vulnerabilities.count > 0` の場合は §4 末尾の対応フロー（§4.8）を確認する。
  Critical（CVSS 9.0+）がある場合は即時 OPS-PROC-009 を起動する。

### 4.2 ステップ 2: React Native — pnpm audit（ハンディ APP）

- [CMD]
  ```bash
  echo "=== pnpm audit (handy-app) ===" | tee -a "${AUDIT_LOG}"
  cd /opt/wnav/handy-app

  pnpm audit --json 2>&1 | tee "${AUDIT_DIR}/${QUARTER_DATE}-pnpm-handy-audit.json"
  pnpm audit 2>&1 | tee -a "${AUDIT_LOG}"
  echo "pnpm handy audit exit: $?" | tee -a "${AUDIT_LOG}"
  ```

- [CMD]
  ```bash
  echo "=== pnpm outdated (handy-app) ===" | tee -a "${AUDIT_LOG}"
  pnpm outdated 2>&1 | tee -a "${AUDIT_LOG}"
  ```

### 4.3 ステップ 3: React — pnpm audit（マスタメンテナンス UI）

- [CMD]
  ```bash
  echo "=== pnpm audit (master-ui) ===" | tee -a "${AUDIT_LOG}"
  cd /opt/wnav/master-ui

  pnpm audit --json 2>&1 | tee "${AUDIT_DIR}/${QUARTER_DATE}-pnpm-ui-audit.json"
  pnpm audit 2>&1 | tee -a "${AUDIT_LOG}"
  echo "pnpm ui audit exit: $?" | tee -a "${AUDIT_LOG}"
  ```

- [CMD]
  ```bash
  echo "=== pnpm outdated (master-ui) ===" | tee -a "${AUDIT_LOG}"
  pnpm outdated 2>&1 | tee -a "${AUDIT_LOG}"
  ```

### 4.4 ステップ 4: Docker ベースイメージ確認

- [CMD]
  ```bash
  echo "=== Docker image pull (latest check) ===" | tee -a "${AUDIT_LOG}"
  cd /opt/wnav

  # pull して更新が存在するか確認（実際の更新は §4.5 で実施）
  docker compose pull --dry-run 2>&1 | tee -a "${AUDIT_LOG}" || \
    docker compose pull 2>&1 | tee -a "${AUDIT_LOG}"

  # 現在使用中のイメージバージョン
  docker images | grep -E 'work-nav|postgres|nginx|grafana|prom' | tee -a "${AUDIT_LOG}"
  ```

- [CHECK] ベースイメージに更新が存在する場合は §4.5 の更新手順を実施する。

### 4.5 ステップ 5: Docker イメージ更新（更新存在時のみ）

- [CMD]
  ```bash
  echo "=== Docker compose pull ===" | tee -a "${AUDIT_LOG}"
  cd /opt/wnav

  # 更新を実際に取得
  docker compose pull 2>&1 | tee -a "${AUDIT_LOG}"

  # 再ビルド（アプリケーションイメージ）
  docker compose build --no-cache 2>&1 | tee -a "${AUDIT_LOG}"
  ```

- [CHECK] ビルドが exit 0 で完了すること。

### 4.6 ステップ 6: Windows Server / OS パッチ確認

- [PS]
  ```powershell
  Write-Host "=== Windows Update Check ==="
  Get-WindowsUpdate -MicrosoftUpdate | Where-Object {
    $_.Title -like "*Security*"
  } | Select-Object Title, KB, Severity, ReleaseDate |
    Format-Table -AutoSize |
    Tee-Object -FilePath "C:\Logs\wnav\quarterly-wu-check.log"
  ```

- [PS]
  ```powershell
  # WSL2 カーネル更新確認
  wsl --update --check
  wsl --version
  ```

- [CHECK] Severity が `Critical` または `Important` の未適用パッチが存在する場合は OPS-PROC-009 に委任する。

### 4.7 ステップ 7: PostgreSQL マイナーバージョン確認

- [CMD]
  ```bash
  echo "=== PostgreSQL version check ===" | tee -a "${AUDIT_LOG}"
  docker compose exec postgres psql -U work_nav -c "SELECT version();" 2>&1 | tee -a "${AUDIT_LOG}"

  # Docker Hub で最新 16.x の stable タグを確認
  docker pull postgres:16 --dry-run 2>/dev/null || \
    echo "Check https://hub.docker.com/_/postgres for latest 16.x" | tee -a "${AUDIT_LOG}"
  ```

### 4.8 ステップ 8: CVE 重大度別対応フロー判定

全スキャン結果を集計し対応要否を判定する。

- [CMD]
  ```bash
  echo "=== CVE Severity Summary ===" | tee -a "${AUDIT_LOG}"

  # cargo audit の Critical/High を抽出
  CRITICAL_RUST=$(cat "${AUDIT_DIR}/${QUARTER_DATE}-cargo-audit.json" \
    | jq '[.vulnerabilities.list[] | select(.advisory.cvss_v3_base_score >= 9.0)] | length' 2>/dev/null || echo 0)
  HIGH_RUST=$(cat "${AUDIT_DIR}/${QUARTER_DATE}-cargo-audit.json" \
    | jq '[.vulnerabilities.list[] | select(.advisory.cvss_v3_base_score >= 7.0 and .advisory.cvss_v3_base_score < 9.0)] | length' 2>/dev/null || echo 0)

  echo "Rust Critical: ${CRITICAL_RUST}" | tee -a "${AUDIT_LOG}"
  echo "Rust High: ${HIGH_RUST}" | tee -a "${AUDIT_LOG}"

  if [ "${CRITICAL_RUST}" -gt 0 ]; then
    echo "ACTION REQUIRED: Critical CVEs found. Escalate to OPS-PROC-009 immediately." | tee -a "${AUDIT_LOG}"
  fi
  ```

対応 SLO（NFR-OPS-041 による）:

| CVSS | 重大度 | 対応期限 | 委任先 |
|---|---|---|---|
| 9.0+ | Critical | 3 日以内 | OPS-PROC-009（緊急パッチ） |
| 7.0〜8.9 | High | 30 日以内 | 次回月次メンテナンス |
| 4.0〜6.9 | Medium | 次回四半期更新 | OPS-PROC-007 次回 |
| 0.1〜3.9 | Low | 年次 | OPS-PROC-008（年次監査） |

**本節で確定した方針**
- Critical が 1 件以上検出された場合は即時 OPS-PROC-009 を起動することを確定する。
- 全スキャン結果はファイルに保存し `maintenance_log` で参照可能にすることを確定する。

---

## 5. 合格基準

| CHK-ID | 基準 | 合否 |
|---|---|---|
| DEP-1 | `cargo audit` が exit 0 またはスキャン完了（脆弱性ゼロ） | ☐ |
| DEP-2 | `pnpm audit` (handy-app / master-ui) が exit 0 またはスキャン完了 | ☐ |
| DEP-3 | Docker イメージに Critical セキュリティパッチがない または適用済み | ☐ |
| DEP-4 | Windows Security Update に Critical/Important 未適用がない または委任済み | ☐ |
| DEP-5 | スキャン結果ファイルが `/opt/wnav/reports/audit/` に保存されている | ☐ |

**本節で確定した方針**
- Critical CVE が存在する場合は DEP-1〜4 の合否にかかわらず OPS-PROC-009 を起動することを確定する。

---

## 6. 異常時の判断

| 事象 | 打ち切り条件 | 通知先 | 代替手順 |
|---|---|---|---|
| Critical CVE 検出 | 当該スキャン後に即時 OPS-PROC-009 起動 | system_admin | OPS-PROC-009（緊急パッチ適用） |
| cargo audit がネットワークエラーで失敗 | 継続（オフライン DB で再試行） | system_admin | `cargo audit --no-fetch` でオフライン実行 |
| Docker pull が失敗 | 継続（記録必須） | system_admin | Docker Hub の状態確認・翌日再試行 |

**本節で確定した方針**
- Critical CVE はスキャン完了を待たず即時 OPS-PROC-009 を起動することを確定する。

---

## 7. 終了条件と記録

- [SQL] maintenance_log への INSERT
  ```sql
  INSERT INTO maintenance_log (log_type, executed_at, executed_by, detail)
  VALUES (
    'quarterly_dependency_review',
    NOW(),
    'system_admin',
    '{"result": "pass", "quarter": "2026-Q2", "rust_critical": 0, "rust_high": 0, "node_critical": 0, "node_high": 0, "docker_updated": false, "windows_critical": 0, "report_path": "/opt/wnav/reports/audit/2026-Q2-dependency-review.log"}'
  );
  ```

**本節で確定した方針**
- `maintenance_log` への記録なしに四半期依存更新レビュー完了と見なさないことを確定する。

---

## 8. ロールバック / 代替手順

Docker イメージを更新後に API が起動しない場合は前バージョンに戻す。

- [CMD]
  ```bash
  # 前のイメージタグを確認
  docker images --format "{{.Repository}}:{{.Tag}} {{.CreatedAt}}" | grep work-nav

  # 前バージョンに戻す（タグを指定）
  docker compose stop api
  docker tag work-nav-api:previous work-nav-api:latest
  docker compose up -d api
  curl -fsS http://localhost:8080/health
  ```

**本節で確定した方針**
- Docker イメージ更新のロールバックは前バージョンのイメージタグへの差し戻しで対応することを確定する。

---

## 9. 関連識別子・改訂履歴

| 属性 | 内容 |
|---|---|
| **関連 BAT** | — |
| **関連 ALERT** | — |
| **関連 ERR** | — |
| **関連 KEY** | — |
| **関連 ADR-IMPL** | — |
| **初版** | 2026-05-18 RyuheiKiso |

---

## 参照業界分析

### 必須
- IPA 共通フレーム 2013 SLCP-JCF2013 4.2.1.c（業務及びシステムの運用）

### 関連
- NIST NVD（National Vulnerability Database）https://nvd.nist.gov/
- RustSec Advisory Database https://rustsec.org/
- GitHub Advisory Database（pnpm audit のデータソース）
- CWE/CVSSv3 スコアリング基準（FIRST.org）
- NFR-OPS-040/041、MNT-017〜019（本プロジェクト要件定義）
