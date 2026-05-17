# 01 週次ヘルスチェック手順（OPS-PROC-001）

本手順書の責務は毎週月曜 09:00 に実施するシステム全体の健全性を確定することである。上流要件 NFR-OPS-002/052・NFR-AVL-008（`docs/04_概要設計/08_運用方式設計/07_アカウント・変更管理と運用手順.md`）を手順に具体化する。IPA 共通フレーム 2013「4.2.1.c 業務及びシステムの運用」に準拠する。

---

## 1. 目的と上流要件

| 属性 | 内容 |
|---|---|
| **手順 ID** | OPS-PROC-001 |
| **頻度** | 週次（毎週月曜 09:00） |
| **想定所要時間** | P50: 15 分 / P95: 30 分 |
| **実施権限** | system_admin（必須） |

上流要件:
- NFR-OPS-002: システムの稼働状態を週次で確認し記録すること
- NFR-OPS-052: 監視ダッシュボードの異常を週次レビューすること
- NFR-AVL-008: API エンドポイントの可用性を継続的に監視すること
- ALERT-001〜004: 以下のアラートが発火していないことを確認すること

**本節で確定した方針**
- 週次ヘルスチェックは毎週月曜 09:00 に必ず実施することを確定する。
- CHK-001〜006 の全項目を完了しなければ週次チェック完了と見なさないことを確定する。
- 異常検出時は本手順書 §6 の判断基準に従い即時対応することを確定する。

---

## 2. 前提条件チェックリスト

以下をすべて確認してから手順を開始する。1 つでも NG なら手順を開始しない。

- [ ] system_admin として認証済みのターミナルセッションが確立されている
- [ ] WSL2 環境が起動しており Docker Compose が稼働中である
- [ ] Grafana（http://localhost:3000）にアクセス可能である
- [ ] Prometheus（http://localhost:9090）にアクセス可能である
- [ ] psql コマンドが使用可能であり PostgreSQL コンテナが起動している
- [ ] 前回のヘルスチェック記録（`maintenance_log`）が存在する

**本節で確定した方針**
- 前提条件チェックリストに 1 つでも NG がある場合は手順を開始しないことを確定する。
- Grafana / Prometheus が起動していない場合は `docker compose up -d` で先に起動することを確定する。

---

## 3. 事前準備

作業記録用のログファイルを準備する。

- [CMD]
  ```bash
  export WNAV_CHECK_DATE=$(date +%Y-%m-%d)
  export WNAV_CHECK_LOG="/tmp/wnav-health-${WNAV_CHECK_DATE}.log"
  echo "=== Weekly Health Check: ${WNAV_CHECK_DATE} ===" | tee "${WNAV_CHECK_LOG}"
  ```

- [CMD] Docker コンテナ稼働確認
  ```bash
  docker compose -f /opt/wnav/docker-compose.yml ps | tee -a "${WNAV_CHECK_LOG}"
  ```

- [CHECK] 全コンテナの Status が `running` または `Up` であることを確認する。

**本節で確定した方針**
- 作業ログは `/tmp/wnav-health-YYYY-MM-DD.log` に出力し終了後に `maintenance_log` と照合することを確定する。

---

## 4. 実施手順

以下の操作タグを使用する。
- `[CMD]` シェルコマンド（WSL2 + bash）
- `[SQL]` PostgreSQL クエリ（psql 経由）
- `[PS]` PowerShell（IIS / Windows Server 操作）
- `[GUI]` ブラウザ / Grafana / 管理 UI 操作
- `[CHECK]` 確認・検証操作

### 4.1 ステップ 1: CHK-001 — API ヘルスチェック

- [CMD]
  ```bash
  curl -fsS http://localhost:8080/health | tee -a "${WNAV_CHECK_LOG}"
  echo "EXIT: $?" | tee -a "${WNAV_CHECK_LOG}"
  ```

- [CHECK] exit 0 かつ レスポンスボディに `{"status":"ok"}` が含まれること。
  HTTP ステータスコードが 200 であることを確認する。

### 4.2 ステップ 2: CHK-002 — PostgreSQL 接続確認

- [CMD]
  ```bash
  pg_isready -h localhost -p 5432 -U work_nav | tee -a "${WNAV_CHECK_LOG}"
  ```

- [CHECK] 出力に `accepting connections` が含まれること。

### 4.3 ステップ 3: CHK-003 — バックアップ完了確認

- [SQL]
  ```sql
  SELECT job_id, status, started_at, finished_at, error_detail
  FROM job_executions
  WHERE job_id = 'BAT-001'
  ORDER BY started_at DESC
  LIMIT 1;
  ```

- [CHECK] `status = 'SUCCESS'` かつ `finished_at` が過去 25 時間以内であること。
  `error_detail` が NULL であること。

### 4.4 ステップ 4: CHK-004 — Outbox 滞留確認

- [SQL]
  ```sql
  -- Outbox pending 件数確認
  SELECT count(*) AS pending FROM outbox_events WHERE status = 'pending';
  -- DLQ 件数確認
  SELECT count(*) AS dlq FROM outbox_dlq;
  ```

- [CHECK] `pending` が 100 件以下であること（閾値: NFR-OPS-052 参照）。
  `dlq` が 0 件であること。

### 4.5 ステップ 5: CHK-005 — ディスク使用率確認

- [CMD]
  ```bash
  df -h /opt/wnav /backup | tee -a "${WNAV_CHECK_LOG}"
  ```

- [CHECK] `/opt/wnav` および `/backup` のいずれも使用率が 85% 以下であること。

- [CMD] Windows ホスト側のディスク確認
  ```bash
  df -h /mnt/c | tee -a "${WNAV_CHECK_LOG}"
  ```

### 4.6 ステップ 6: CHK-006 — 認証失敗ログ確認

- [CMD]
  ```bash
  # 過去 24 時間の認証失敗イベント件数
  docker compose -f /opt/wnav/docker-compose.yml logs --since 24h api 2>/dev/null \
    | grep -E 'ERR-AUTH-00[1-4]' \
    | wc -l \
    | tee -a "${WNAV_CHECK_LOG}"
  ```

- [SQL]
  ```sql
  SELECT event_code, count(*) AS cnt
  FROM audit_logs
  WHERE event_code LIKE 'ERR-AUTH-%'
    AND created_at >= NOW() - INTERVAL '24 hours'
  GROUP BY event_code
  ORDER BY cnt DESC;
  ```

- [CHECK] ERR-AUTH-001〜004 の合計件数が 50 件以下であること（閾値超過時は ALERT-002 相当として記録）。

### 4.7 ステップ 7: Grafana ダッシュボード確認

- [GUI] http://localhost:3000 → Work Navigation Overview ダッシュボードを開く
  - API レイテンシ（P95）が 200ms 以下であることを確認する
  - エラーレート（5xx）が 0.1% 以下であることを確認する
  - メモリ使用量が上限の 80% 以下であることを確認する

### 4.8 ステップ 8: Windows Event Log 確認

- [PS]
  ```powershell
  Get-WinEvent -LogName Application -MaxEvents 200 |
    Where-Object { $_.LevelDisplayName -in @('Critical','Error') -and $_.TimeCreated -gt (Get-Date).AddHours(-168) } |
    Select-Object TimeCreated, LevelDisplayName, Message |
    Format-Table -AutoSize
  ```

- [CHECK] CRITICAL イベントが 0 件であること。ERROR イベントが既知のもの以外でないことを確認する。

### 4.9 ステップ 9: 脆弱性スキャン結果確認

- [CMD]
  ```bash
  # 最新の cargo audit 結果ファイルを確認
  ls -lt /opt/wnav/reports/audit/ | head -5
  cat /opt/wnav/reports/audit/latest-cargo-audit.json \
    | jq '.vulnerabilities.count' 2>/dev/null || echo "No recent audit found"
  ```

- [CHECK] `vulnerabilities.count` が 0 であること。0 でない場合は OPS-PROC-007 または OPS-PROC-009 に委任する。

**本節で確定した方針**
- CHK-001〜006 の順序を変えずに実施することを確定する。
- いずれかのチェックが NG の場合は §6 の異常時判断に従うことを確定する。

---

## 5. 合格基準

| CHK-ID | 基準 | 合否 |
|---|---|---|
| CHK-001 | API `/health` が HTTP 200 かつ `{"status":"ok"}` を返す | ☐ |
| CHK-002 | `pg_isready` が `accepting connections` を返す | ☐ |
| CHK-003 | BAT-001 最新実行が `SUCCESS` かつ 25h 以内に完了 | ☐ |
| CHK-004 | `outbox_events.pending ≤ 100` かつ `outbox_dlq = 0` | ☐ |
| CHK-005 | 全マウントポイントのディスク使用率 ≤ 85% | ☐ |
| CHK-006 | 24h 以内の ERR-AUTH-001〜004 合計 ≤ 50 件 | ☐ |

全 CHK が合格で週次ヘルスチェック完了とする。

**本節で確定した方針**
- 全 6 項目が合格でなければ週次ヘルスチェック完了と見なさないことを確定する。

---

## 6. 異常時の判断

| 事象 | 打ち切り条件 | 通知先 | 代替手順 |
|---|---|---|---|
| API が 200 を返さない | 即時打ち切り | system_admin | `docs/09_運用・保守/障害対応/` の API 障害手順を参照 |
| PostgreSQL が `accepting connections` でない | 即時打ち切り | system_admin | DB コンテナ再起動 → 改善なければ OPS-PROC-010 を起動 |
| BAT-001 が FAILED または実行されていない | 即時打ち切り | system_admin | OPS-PROC-002 §4 の手動バックアップ手順を実施 |
| Outbox pending > 100 または DLQ > 0 | 継続（ただし記録必須） | system_admin | Outbox フラッシュバッチ手動起動 |
| ディスク使用率 > 85% | 継続（ただし記録必須） | system_admin | 古いログ・バックアップの整理（NFR-OPS-045 参照） |
| 認証失敗 > 50 件 / 24h | 継続（ただし記録必須） | system_admin・quality_admin | ERR-AUTH-* 調査、ブルートフォース判定時は IP ブロック |

**本節で確定した方針**
- API 障害・DB 障害は即時打ち切りとし後続手順に委任することを確定する。
- Outbox / ディスク異常は打ち切りなしで継続するが必ず記録することを確定する。

---

## 7. 終了条件と記録

全 CHK 合格後に以下を実施する。

- [SQL] maintenance_log への INSERT
  ```sql
  INSERT INTO maintenance_log (log_type, executed_at, executed_by, detail)
  VALUES (
    'weekly_health_check',
    NOW(),
    'system_admin',
    '{"result": "pass", "chk_001": "pass", "chk_002": "pass", "chk_003": "pass", "chk_004": "pass", "chk_005": "pass", "chk_006": "pass"}'
  );
  ```

- [CMD] ログファイルを保存先にアーカイブする
  ```bash
  cp "${WNAV_CHECK_LOG}" /opt/wnav/logs/health-checks/
  ```

- [CHECK] `maintenance_log` に正常 INSERT されていることを確認する。

**本節で確定した方針**
- `maintenance_log` への記録なしに手順完了と見なさないことを確定する。
- CHK に NG があった場合は `result: "partial"` と個別 CHK の合否を記録することを確定する。

---

## 8. ロールバック / 代替手順

本手順書はヘルスチェック（読み取り専用操作）であるためロールバックは発生しない。
異常が検出された場合は §6 の「代替手順」列に記載の手順を起動する。

**本節で確定した方針**
- 週次ヘルスチェックはロールバック不要の読み取り専用手順であることを確定する。

---

## 9. 関連識別子・改訂履歴

| 属性 | 内容 |
|---|---|
| **関連 BAT** | BAT-001（日次バックアップ） |
| **関連 ALERT** | ALERT-001（API ダウン）、ALERT-002（認証失敗多発）、ALERT-003（ディスク満杯）、ALERT-004（Outbox 滞留） |
| **関連 ERR** | ERR-AUTH-001〜004 |
| **関連 KEY** | — |
| **関連 ADR-IMPL** | — |
| **初版** | 2026-05-18 RyuheiKiso |

---

## 参照業界分析

### 必須
- IPA 共通フレーム 2013 SLCP-JCF2013 4.2.1.c（業務及びシステムの運用）

### 関連
- IPA「情報システムの信頼性向上に関するガイドライン」第 2 版 3.3 節（定期ヘルスチェック）
- Prometheus Operator Alert Manager ベストプラクティス（Grafana Labs 公式ドキュメント）
- NFR-OPS-002、NFR-OPS-052、NFR-AVL-008（本プロジェクト要件定義）
