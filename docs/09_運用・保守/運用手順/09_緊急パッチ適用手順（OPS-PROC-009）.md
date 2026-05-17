# 09 緊急パッチ適用手順（OPS-PROC-009）

本手順書の責務は CVSS 9.0 以上の Critical 脆弱性に対し 3 日以内のパッチ適用を確定することである。上流要件 NFR-OPS-041・NFR-AVL-004（`docs/04_概要設計/08_運用方式設計/07_アカウント・変更管理と運用手順.md`）を手順に具体化する。IPA 共通フレーム 2013「4.2.1.c 業務及びシステムの運用」に準拠する。

---

## 1. 目的と上流要件

| 属性 | 内容 |
|---|---|
| **手順 ID** | OPS-PROC-009 |
| **頻度** | イベント駆動（Critical CVE 検出時随時） |
| **想定所要時間** | P50: 60 分 / P95: 120 分（停止時間は最大 15 分以内） |
| **実施権限** | system_admin（必須） |

上流要件:
- NFR-OPS-041: CVSS 9.0+ の Critical 脆弱性は検出から 3 日以内にパッチを適用すること
- NFR-AVL-004: 緊急パッチ適用による計画停止は 06:00〜22:00 の作業時間帯で最大 15 分を上限とすること

**本節で確定した方針**
- CVSS 9.0+ を検出してから 3 日（72 時間）以内にパッチ適用を完了することを確定する。
- 停止を伴うパッチ適用は 06:00〜22:00 に限定し停止時間は 15 分以内に抑えることを確定する。
- パッチ適用の前日（少なくとも 24 時間前）に現場監督および quality_admin に通知することを確定する。

---

## 2. 前提条件チェックリスト

以下をすべて確認してから手順を開始する。1 つでも NG なら手順を開始しない。

- [ ] system_admin として認証済みのターミナルセッションが確立されている
- [ ] 現場監督および quality_admin への停止通知が 24 時間前以上に完了している
- [ ] 最新バックアップが BAT-001 により正常に完了している（CHK-007 相当）
- [ ] 作業時間帯が 06:00〜22:00 の範囲内である
- [ ] ステージング環境でパッチ適用済みの動作確認が完了している
- [ ] ロールバック用の現行イメージタグが記録されている

**本節で確定した方針**
- 前提条件チェックリストに 1 つでも NG がある場合は手順を開始しないことを確定する。
- ステージング確認未完了の場合は本番適用を行わないことを確定する。

---

## 3. 事前準備

### 3.1 脆弱性の特定と影響評価

- [CMD]
  ```bash
  # Critical CVE の詳細確認
  cd /opt/wnav/backend
  cargo audit --json | jq '.vulnerabilities.list[] | select(.advisory.cvss_v3_base_score >= 9.0) | {
    id: .advisory.id,
    package: .package.name,
    version: .package.version,
    cvss: .advisory.cvss_v3_base_score,
    description: .advisory.description,
    patched_versions: .versions.patched
  }'
  ```

- [CMD] Node.js の Critical CVE 確認
  ```bash
  cd /opt/wnav/handy-app && pnpm audit --json | jq '.advisories | to_entries[] | select(.value.severity == "critical") | .value | {title, module_name, cvss_score: .findings[].paths}'
  ```

### 3.2 ロールバック情報の記録

- [CMD]
  ```bash
  # 現行の依存バージョンを記録
  cp /opt/wnav/backend/Cargo.lock /tmp/Cargo.lock.bak-$(date +%Y%m%d)
  cp /opt/wnav/handy-app/pnpm-lock.yaml /tmp/pnpm-lock.yaml.bak-$(date +%Y%m%d)

  # 現行の Docker イメージ ID を記録
  docker images --format "{{.Repository}}:{{.Tag}} {{.ID}}" | grep work-nav | tee /tmp/docker-images-bak-$(date +%Y%m%d).txt
  ```

### 3.3 現場監督への停止通知

- [CMD]
  ```bash
  # nxlog 経由で Windows Event Log に停止予告を記録
  /opt/wnav/bin/notify-maintenance \
    --type "emergency_patch" \
    --window-start "$(date '+%Y-%m-%d') 21:00" \
    --window-end "$(date '+%Y-%m-%d') 21:15" \
    --cve "CVE-YYYY-NNNNN" \
    --message "緊急セキュリティパッチ適用のため 21:00-21:15 にシステムを停止します"
  ```

**本節で確定した方針**
- 事前準備で脆弱性の影響範囲・ロールバック情報・通知記録をすべて完了することを確定する。

---

## 4. 実施手順

以下の操作タグを使用する。
- `[CMD]` シェルコマンド（WSL2 + bash）
- `[SQL]` PostgreSQL クエリ（psql 経由）
- `[PS]` PowerShell（IIS / Windows Server 操作）
- `[GUI]` ブラウザ / Grafana / 管理 UI 操作
- `[CHECK]` 確認・検証操作

### 4.1 ステップ 0: 停止前バックアップ（本番実施直前）

- [SQL]
  ```sql
  -- 手動バックアップジョブを即時起動
  INSERT INTO job_executions (job_id, status, started_at, triggered_by)
  VALUES ('BAT-001-EMERGENCY', 'running', NOW(), 'system_admin');
  ```

- [CMD]
  ```bash
  # 緊急バックアップ実行
  /opt/wnav/bin/run-backup.sh --type emergency | tee /tmp/emergency-backup-$(date +%Y%m%d).log
  ```

- [CHECK] バックアップが完了し SHA256 チェックが通ること。

### 4.2 ステップ 1: maintenance_mode フラグの設定

- [SQL]
  ```sql
  UPDATE system_settings
  SET value = 'true', updated_at = NOW(), updated_by = 'system_admin'
  WHERE key = 'maintenance_mode';
  ```

- [CHECK] ハンディ端末が「メンテナンス中」表示に切り替わること。
  新規 Step イベントの POST が 503 を返すこと。

### 4.3 ステップ 2: Rust crate の更新（Backend）

- [CMD]
  ```bash
  cd /opt/wnav/backend

  # 対象 crate を更新
  cargo update -p <affected_crate>

  # 更新後の脆弱性確認
  cargo audit
  echo "cargo audit exit: $?"

  # ビルドとテスト
  cargo build --release 2>&1 | tee /tmp/cargo-build-$(date +%Y%m%d).log
  cargo test --release 2>&1 | tee -a /tmp/cargo-build-$(date +%Y%m%d).log
  echo "cargo test exit: $?"
  ```

- [CHECK] `cargo audit` が exit 0 かつ Critical が 0 件であること。
  `cargo test` が全テスト PASS であること。

### 4.4 ステップ 3: Node.js 依存の更新（該当時のみ）

- [CMD]
  ```bash
  # handy-app
  cd /opt/wnav/handy-app
  pnpm update <affected_package>
  pnpm audit
  echo "pnpm handy audit exit: $?"
  pnpm test 2>&1 | tee /tmp/pnpm-test-$(date +%Y%m%d).log

  # master-ui
  cd /opt/wnav/master-ui
  pnpm update <affected_package>
  pnpm audit
  pnpm build 2>&1 | tee -a /tmp/pnpm-test-$(date +%Y%m%d).log
  ```

- [CHECK] 両プロジェクトとも `pnpm audit` で Critical が 0 件であること。

### 4.5 ステップ 4: Docker ビルドと本番適用（停止タイマー開始）

- [CMD]
  ```bash
  PATCH_START=$(date +%s)
  echo "PATCH_START=$(date)" | tee /tmp/patch-timer-$(date +%Y%m%d).log

  cd /opt/wnav

  # サービス停止
  docker compose down

  # 新イメージのビルド
  docker compose build --no-cache 2>&1 | tee -a /tmp/patch-timer-$(date +%Y%m%d).log

  # 起動
  docker compose up -d

  # ヘルスチェックが通るまで待機（最大 120 秒）
  for i in $(seq 1 24); do
    curl -fsS http://localhost:8080/health && break
    echo "Waiting... ${i}/24"
    sleep 5
  done

  PATCH_END=$(date +%s)
  PATCH_DURATION=$((PATCH_END - PATCH_START))
  echo "Patch duration: ${PATCH_DURATION}s" | tee -a /tmp/patch-timer-$(date +%Y%m%d).log
  ```

- [CHECK] `PATCH_DURATION ≤ 900`（15 分以内）であること。
  API ヘルスチェックが 200 を返すこと。

### 4.6 ステップ 5: maintenance_mode フラグの解除

- [SQL]
  ```sql
  UPDATE system_settings
  SET value = 'false', updated_at = NOW(), updated_by = 'system_admin'
  WHERE key = 'maintenance_mode';
  ```

- [CHECK] ハンディ端末が通常動作に復帰すること。
  Step イベントの POST が 200 を返すこと。

### 4.7 ステップ 6: パッチ後の脆弱性再確認

- [CMD]
  ```bash
  cd /opt/wnav/backend && cargo audit
  cd /opt/wnav/handy-app && pnpm audit
  cd /opt/wnav/master-ui && pnpm audit
  ```

- [CHECK] 全スキャンで Critical が 0 件であること。

**本節で確定した方針**
- ステップ 4 の停止時間を計測し 15 分以内であることを確認することを確定する。
- 15 分を超えた場合は §6 の異常時判断に従うことを確定する。

---

## 5. 合格基準

| CHK-ID | 基準 | 合否 |
|---|---|---|
| PATCH-1 | `cargo audit` / `pnpm audit` で Critical が 0 件 | ☐ |
| PATCH-2 | `cargo test` が全 PASS | ☐ |
| PATCH-3 | 停止時間 ≤ 900 秒（15 分） | ☐ |
| PATCH-4 | API ヘルスチェックが 200 OK を返す | ☐ |
| PATCH-5 | `maintenance_mode = false` に戻っている | ☐ |
| PATCH-6 | パッチ適用が 72 時間（3 日）以内に完了した | ☐ |

**本節で確定した方針**
- 全 6 項目が合格でなければ緊急パッチ適用完了と見なさないことを確定する。

---

## 6. 異常時の判断

| 事象 | 打ち切り条件 | 通知先 | 代替手順 |
|---|---|---|---|
| `cargo test` が FAIL | 即時打ち切り・ロールバック | system_admin | §8 のロールバック手順を実施 |
| 停止時間が 15 分超過 | 停止継続（記録必須） | system_admin・quality_admin | 15 分超過を理由に作業継続し完了後に報告 |
| Docker ビルドが失敗 | 即時ロールバック | system_admin | §8 のロールバック手順を実施 |
| パッチ後も Critical CVE が残存 | 打ち切りなし（追加更新） | system_admin | 追加 crate の更新を繰り返す |

**本節で確定した方針**
- テスト FAIL またはビルド失敗は即時ロールバックとすることを確定する。

---

## 7. 終了条件と記録

- [SQL] maintenance_log への INSERT
  ```sql
  INSERT INTO maintenance_log (log_type, executed_at, executed_by, detail)
  VALUES (
    'emergency_patch',
    NOW(),
    'system_admin',
    '{"result": "pass", "cve": "CVE-YYYY-NNNNN", "cvss": 9.5, "patch_duration_sec": 480, "cargo_test": "pass", "detection_to_patch_h": 48}'
  );
  ```

**本節で確定した方針**
- `maintenance_log` への記録なしに緊急パッチ適用完了と見なさないことを確定する。

---

## 8. ロールバック / 代替手順

**Docker ロールバック手順:**

- [CMD]
  ```bash
  # ロールバック対象のイメージ ID を確認
  cat /tmp/docker-images-bak-$(date +%Y%m%d).txt

  # 前バージョンのイメージで起動
  docker compose down
  docker tag work-nav-api:previous work-nav-api:latest
  docker compose up -d

  curl -fsS http://localhost:8080/health
  echo "Rollback completed"
  ```

**Cargo.lock ロールバック手順:**

- [CMD]
  ```bash
  cp /tmp/Cargo.lock.bak-$(date +%Y%m%d) /opt/wnav/backend/Cargo.lock
  cd /opt/wnav/backend
  cargo build --release
  ```

- [SQL] ロールバック記録
  ```sql
  INSERT INTO maintenance_log (log_type, executed_at, executed_by, detail)
  VALUES (
    'emergency_patch_rollback',
    NOW(),
    'system_admin',
    '{"result": "rollback", "reason": "test_failure", "cve": "CVE-YYYY-NNNNN"}'
  );
  ```

**本節で確定した方針**
- ロールバック後も `maintenance_mode = false` に戻すことを確定する。
- ロールバック後は脆弱性が残存しているため 3 日 SLO のカウントダウンは継続し別の対応策を策定することを確定する。

---

## 9. 関連識別子・改訂履歴

| 属性 | 内容 |
|---|---|
| **関連 BAT** | BAT-001（緊急バックアップ起動） |
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
- NIST SP 800-40 Rev.4「Guide to Enterprise Patch Management Planning」§3（パッチ適用タイムライン）
- CVSSv3.1 Specification Guide（FIRST.org）§3.1（Base Score Severity）
- RustSec Advisory Database（cargo audit のアドバイザリソース）
- NFR-OPS-041、NFR-AVL-004（本プロジェクト要件定義）
