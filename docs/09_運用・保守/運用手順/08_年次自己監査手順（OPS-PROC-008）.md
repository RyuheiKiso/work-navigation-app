# 08 年次自己監査手順（OPS-PROC-008）

本手順書の責務は年次自己監査を実施しシステムの運用品質を確定することである。上流要件 NFR-OPS-042（`docs/04_概要設計/08_運用方式設計/07_アカウント・変更管理と運用手順.md`）を手順に具体化する。IPA 共通フレーム 2013「4.2.1.c 業務及びシステムの運用」に準拠する。

---

## 1. 目的と上流要件

| 属性 | 内容 |
|---|---|
| **手順 ID** | OPS-PROC-008 |
| **頻度** | 年次（毎年 12 月第 2 週） |
| **想定所要時間** | P50: 120 分 / P95: 180 分 |
| **実施権限** | system_admin（実施）・quality_admin（独立レビュー・必須） |

上流要件:
- NFR-OPS-042: 年次で自己監査を実施し 28 項目チェックリストに基づく評価結果を記録すること

**本節で確定した方針**
- 年次自己監査は system_admin が実施し quality_admin が独立レビューを行うことを確定する。
- CHK-015〜042 の全 28 項目を評価し、FAIL 項目は翌月末までに改善計画を作成することを確定する。
- 年次監査レポートは quality_admin の署名付きで保存することを確定する。

---

## 2. 前提条件チェックリスト

以下をすべて確認してから手順を開始する。1 つでも NG なら手順を開始しない。

- [ ] system_admin として認証済みのターミナルセッションが確立されている
- [ ] quality_admin が独立レビューに参加することを事前確認している
- [ ] 直近 12 ヶ月分の `maintenance_log` が参照可能である
- [ ] 直近 4 回分の四半期依存更新レビュー結果が `/opt/wnav/reports/audit/` に保存されている
- [ ] 直近 12 回分の月次 SLO レポートが `/opt/wnav/reports/slo/` に保存されている

**本節で確定した方針**
- 前提条件チェックリストに 1 つでも NG がある場合は手順を開始しないことを確定する。

---

## 3. 事前準備

- [CMD]
  ```bash
  export AUDIT_YEAR=$(date +%Y)
  export AUDIT_DIR="/opt/wnav/reports/annual-audit"
  mkdir -p "${AUDIT_DIR}"
  export AUDIT_REPORT="${AUDIT_DIR}/annual-audit-${AUDIT_YEAR}.md"
  echo "=== Annual Self-Audit: ${AUDIT_YEAR} ===" | tee "${AUDIT_REPORT}"
  echo "Executed by: system_admin" | tee -a "${AUDIT_REPORT}"
  echo "Date: $(date)" | tee -a "${AUDIT_REPORT}"
  ```

**本節で確定した方針**
- 監査レポートは年次フォルダに保存し quality_admin によるレビュー完了後に確定とすることを確定する。

---

## 4. 実施手順

以下の操作タグを使用する。
- `[CMD]` シェルコマンド（WSL2 + bash）
- `[SQL]` PostgreSQL クエリ（psql 経由）
- `[PS]` PowerShell（IIS / Windows Server 操作）
- `[GUI]` ブラウザ / Grafana / 管理 UI 操作
- `[CHECK]` 確認・検証操作

### 4.1 カテゴリ 1: 可用性 SLO 達成状況（CHK-015〜018）

#### CHK-015: 年間稼働率実績

- [CMD]
  ```bash
  # 直近 12 ヶ月の SLO レポートから可用性を集計
  grep -h "可用性" /opt/wnav/reports/slo/slo-report-*.md | tee -a "${AUDIT_REPORT}"
  ```

- [SQL]
  ```sql
  SELECT
    date_trunc('month', executed_at) AS month,
    (detail->>'availability')::numeric AS availability
  FROM maintenance_log
  WHERE log_type = 'slo_report'
    AND executed_at >= NOW() - INTERVAL '12 months'
  ORDER BY month;
  ```

- [CHECK] 全月の可用性が 99.5% 以上であること（CHK-015 PASS）。

#### CHK-016: RTO 実績（月次リストア検証）

- [SQL]
  ```sql
  SELECT
    date_trunc('month', executed_at) AS month,
    (detail->>'rto_sec')::integer AS rto_sec
  FROM maintenance_log
  WHERE log_type = 'restore_verification'
    AND executed_at >= NOW() - INTERVAL '12 months'
  ORDER BY month;
  ```

- [CHECK] 全 12 回の RTO が 3600 秒（60 分）以下であること（CHK-016 PASS）。

#### CHK-017: RPO 実績（WAL アーカイブ遅延）

- [SQL]
  ```sql
  SELECT count(*) AS months_checked,
         max((detail->>'outbox_max_delay_min')::numeric) AS max_delay
  FROM maintenance_log
  WHERE log_type = 'slo_report'
    AND executed_at >= NOW() - INTERVAL '12 months';
  ```

- [CHECK] `max_delay ≤ 5` であること（CHK-017 PASS）。

#### CHK-018: Step 完了 P95 レイテンシ年間最大値

- [SQL]
  ```sql
  SELECT max((detail->>'latency_p95_ms')::numeric) AS max_latency_ms
  FROM maintenance_log
  WHERE log_type = 'slo_report'
    AND executed_at >= NOW() - INTERVAL '12 months';
  ```

- [CHECK] `max_latency_ms ≤ 200` であること（CHK-018 PASS）。

### 4.2 カテゴリ 2: バックアップ完全性（CHK-019〜022）

#### CHK-019: 週次ヘルスチェック実施率

- [SQL]
  ```sql
  SELECT count(*) AS check_count
  FROM maintenance_log
  WHERE log_type = 'weekly_health_check'
    AND executed_at >= NOW() - INTERVAL '12 months';
  ```

- [CHECK] 52 回（年間 52 週）に対して実施率 ≥ 95%（49 回以上）であること（CHK-019 PASS）。

#### CHK-020: バックアップ全世代存在確認

- [CMD]
  ```bash
  echo "=== Backup generation check ===" | tee -a "${AUDIT_REPORT}"
  echo "Daily (expect 7):" && ls /backup/db/daily/*.dump.gz.enc 2>/dev/null | wc -l
  echo "Weekly (expect 13):" && ls /backup/db/weekly/*.dump.gz.enc 2>/dev/null | wc -l
  echo "Monthly (expect 12):" && ls /backup/db/monthly/*.dump.gz.enc 2>/dev/null | wc -l
  ```

- [CHECK] 日次 7・週次 13・月次 12 の全世代が存在すること（CHK-020 PASS）。

#### CHK-021: 月次リストア検証実施率

- [SQL]
  ```sql
  SELECT count(*) AS restore_count
  FROM maintenance_log
  WHERE log_type = 'restore_verification'
    AND executed_at >= NOW() - INTERVAL '12 months';
  ```

- [CHECK] 12 回すべて実施されていること（CHK-021 PASS）。

#### CHK-022: バックアップ暗号化鍵の鍵分離確認

- [CMD]
  ```bash
  # 鍵ファイルがバックアップディレクトリとは別のパスにあることを確認
  ls /etc/wnav/keys/backup-key.bin && echo "Key exists"
  ls /backup/db/*.bin 2>/dev/null && echo "WARNING: Key in backup dir" || echo "OK: No key in backup dir"
  ```

- [CHECK] `/backup/` 配下に鍵ファイルが存在しないこと（CHK-022 PASS）。

### 4.3 カテゴリ 3: ALCOA+ 整合性（CHK-023〜026）

#### CHK-023: 監査ログ連続性確認

- [SQL]
  ```sql
  -- 過去 1 年間の監査ログに欠落がないか確認
  SELECT
    date_trunc('day', created_at) AS log_date,
    count(*) AS event_count
  FROM audit_logs
  WHERE created_at >= NOW() - INTERVAL '12 months'
  GROUP BY log_date
  HAVING count(*) = 0  -- 欠落日を検出
  ORDER BY log_date;
  ```

- [CHECK] 結果が 0 行（欠落日なし）であること（CHK-023 PASS）。

#### CHK-024: ハッシュチェーン整合性（年次全件確認）

- [SQL]
  ```sql
  SELECT
    sum(CASE WHEN NOT hash_chain_valid THEN 1 ELSE 0 END) AS invalid_count,
    count(*) AS total_count
  FROM (
    SELECT
      (prev_hash = LAG(content_hash) OVER (PARTITION BY entity_type ORDER BY created_at)
       OR LAG(content_hash) OVER (PARTITION BY entity_type ORDER BY created_at) IS NULL) AS hash_chain_valid
    FROM audit_logs
  ) sub;
  ```

- [CHECK] `invalid_count = 0` であること（CHK-024 PASS）。

#### CHK-025: Outbox DLQ 年間累積確認

- [SQL]
  ```sql
  SELECT count(*) AS total_dlq_events,
         max(created_at) AS last_dlq_event
  FROM outbox_dlq
  WHERE created_at >= NOW() - INTERVAL '12 months';
  ```

- [CHECK] `total_dlq_events = 0` であること（CHK-025 PASS）。0 でない場合は DLQ イベントの原因を記録する。

#### CHK-026: XES エクスポート実行確認

- [SQL]
  ```sql
  SELECT count(*) AS export_count,
         max(exported_at) AS last_export
  FROM xes_export_log
  WHERE exported_at >= NOW() - INTERVAL '12 months';
  ```

- [CHECK] 少なくとも 1 回の XES エクスポートが実施されていること（CHK-026 PASS）。

### 4.4 カテゴリ 4: 鍵管理（CHK-027〜029）

#### CHK-027: JWT 鍵ローテーション完了確認

- [SQL]
  ```sql
  SELECT count(*) AS rotation_count
  FROM maintenance_log
  WHERE log_type = 'jwt_key_rotation_confirmation'
    AND executed_at >= NOW() - INTERVAL '12 months';
  ```

- [CHECK] 4 回（四半期 × 4）のローテーション確認が記録されていること（CHK-027 PASS）。

#### CHK-028: 有効期限切れ鍵の不存在確認

- [SQL]
  ```sql
  SELECT count(*) AS expired_active_keys
  FROM jwt_keys
  WHERE status = 'active'
    AND expires_at < NOW();
  ```

- [CHECK] `expired_active_keys = 0` であること（CHK-028 PASS）。

#### CHK-029: TLS 証明書有効期限確認

- [CMD]
  ```bash
  # nginx/IIS で使用中の TLS 証明書の有効期限を確認
  openssl s_client -connect localhost:443 -servername wnav.local 2>/dev/null \
    | openssl x509 -noout -dates
  ```

- [CHECK] `notAfter` が現在から 30 日以上先であること（CHK-029 PASS）。
  30 日以内の場合は `12_証明書更新手順.md` を実施する。

### 4.5 カテゴリ 5: 脆弱性管理（CHK-030〜032）

#### CHK-030: 依存ライブラリ脆弱性ゼロ確認

- [CMD]
  ```bash
  cd /opt/wnav/backend && cargo audit --json | jq '.vulnerabilities.count'
  cd /opt/wnav/handy-app && pnpm audit --json | jq '.metadata.vulnerabilities.total'
  cd /opt/wnav/master-ui && pnpm audit --json | jq '.metadata.vulnerabilities.total'
  ```

- [CHECK] 全カウントが 0 であること（CHK-030 PASS）。

#### CHK-031: 四半期依存更新レビュー実施率

- [SQL]
  ```sql
  SELECT count(*) AS review_count
  FROM maintenance_log
  WHERE log_type = 'quarterly_dependency_review'
    AND executed_at >= NOW() - INTERVAL '12 months';
  ```

- [CHECK] 4 回すべて実施されていること（CHK-031 PASS）。

#### CHK-032: OS セキュリティパッチ適用状況

- [PS]
  ```powershell
  Get-HotFix | Where-Object { $_.InstalledOn -gt (Get-Date).AddDays(-90) } |
    Select-Object HotFixID, Description, InstalledOn |
    Sort-Object InstalledOn -Descending |
    Format-Table -AutoSize
  ```

- [CHECK] 直近 90 日以内に Critical Security Update が 1 件以上適用されていること（CHK-032 PASS）。

### 4.6 カテゴリ 6: ADR 棚卸し・SOP 改訂（CHK-033〜036）

#### CHK-033: ADR の有効性確認

- [CMD]
  ```bash
  # ADR ディレクトリの確認
  ls -la /opt/wnav/docs/adr/ | wc -l
  # 廃止 ADR の確認
  grep -l "DEPRECATED\|SUPERSEDED" /opt/wnav/docs/adr/*.md | wc -l
  ```

- [CHECK] 廃止 ADR が現在の実装と矛盾しないことを確認する（CHK-033 PASS）。

#### CHK-034: SOP 改訂履歴確認

- [SQL]
  ```sql
  SELECT sop_id, title, version, updated_at
  FROM sop_master
  WHERE updated_at >= NOW() - INTERVAL '12 months'
  ORDER BY updated_at DESC;
  ```

- [CHECK] SOP 改訂が `maintenance_log` の `proc_revision` 記録と整合していること（CHK-034 PASS）。

#### CHK-035: 本手順書群の改訂管理確認

- [SQL]
  ```sql
  SELECT count(*) AS revision_count,
         max(executed_at) AS last_revision
  FROM maintenance_log
  WHERE log_type = 'proc_revision'
    AND executed_at >= NOW() - INTERVAL '12 months';
  ```

- [CHECK] 手順書改訂が `maintenance_log` に記録されていること（CHK-035 PASS）。

#### CHK-036: 保守記録の完全性確認

- [SQL]
  ```sql
  SELECT log_type, count(*) AS count
  FROM maintenance_log
  WHERE executed_at >= NOW() - INTERVAL '12 months'
  GROUP BY log_type
  ORDER BY count DESC;
  ```

- [CHECK] `weekly_health_check`・`backup_confirmation`・`restore_verification`・`slo_report` の各 log_type が期待件数存在すること（CHK-036 PASS）。

### 4.7 カテゴリ 7: アカウント管理（CHK-037〜039）

#### CHK-037: 全作業者の有効権限確認

- [SQL]
  ```sql
  -- inactive 作業者に有効 JWT が残存していないか確認
  SELECT w.login_id, count(jt.jti) AS active_tokens
  FROM workers w
  JOIN jwt_tokens jt ON w.id = jt.worker_id
  WHERE w.status = 'inactive'
    AND jt.revoked_at IS NULL
    AND jt.expires_at > NOW()
  GROUP BY w.login_id;
  ```

- [CHECK] 結果が 0 行であること（CHK-037 PASS）。

#### CHK-038: PII 匿名化スケジュール未処理確認

- [SQL]
  ```sql
  SELECT task_type, scheduled_at, target_worker_id, status
  FROM maintenance_schedule
  WHERE task_type IN ('pii_anonymize_partial', 'pii_anonymize_complete')
    AND status = 'pending'
    AND scheduled_at < NOW();
  ```

- [CHECK] 期限超過の `pending` タスクが 0 件であること（CHK-038 PASS）。
  存在する場合は OPS-PROC-005 §4.2.5 / §4.2.6 を即時実施する。

#### CHK-039: アカウント入退職記録の完全性

- [SQL]
  ```sql
  SELECT count(*) AS onboarding, log_type
  FROM maintenance_log
  WHERE log_type IN ('account_onboarding', 'account_offboarding')
    AND executed_at >= NOW() - INTERVAL '12 months'
  GROUP BY log_type;
  ```

- [CHECK] 人事部門からの入退職報告件数と `maintenance_log` の件数が一致すること（CHK-039 PASS）。

### 4.8 カテゴリ 8: インフラ・障害対応（CHK-040〜042）

#### CHK-040: インシデント対応記録の完全性

- [SQL]
  ```sql
  SELECT count(*) AS incident_count
  FROM maintenance_log
  WHERE log_type = 'incident'
    AND executed_at >= NOW() - INTERVAL '12 months';
  ```

- [CHECK] `docs/09_運用・保守/障害対応/` のインシデント記録と `maintenance_log` の件数が一致すること（CHK-040 PASS）。

#### CHK-041: 縮退運用（LEVEL-2）の発動記録確認

- [SQL]
  ```sql
  SELECT count(*) AS degraded_count,
         sum(EXTRACT(EPOCH FROM (ended_at - started_at)) / 60) AS total_min
  FROM maintenance_log
  WHERE log_type = 'degraded_operation'
    AND executed_at >= NOW() - INTERVAL '12 months';
  ```

- [CHECK] 縮退運用の合計時間が年間 40 時間（= 99.5% SLO から逆算）以内であること（CHK-041 PASS）。

#### CHK-042: XES エクスポートと保全確認

- [CMD]
  ```bash
  # XES エクスポートファイルの存在確認
  ls -lh /opt/wnav/exports/xes/*.xes.gz 2>/dev/null | tail -5
  ```

- [SQL]
  ```sql
  SELECT exported_at, record_count, file_path, sha256
  FROM xes_export_log
  ORDER BY exported_at DESC
  LIMIT 3;
  ```

- [CHECK] XES ファイルが存在し SHA256 が `xes_export_log` と一致すること（CHK-042 PASS）。

**本節で確定した方針**
- CHK-015〜042 の全 28 項目を評価し判定を記録することを確定する。
- FAIL 項目は翌月末までに改善計画を作成することを確定する。

---

## 5. 合格基準

| CHK-ID | 基準 | 合否 |
|---|---|---|
| CHK-015 | 年間可用性 ≥ 99.5%（全 12 月） | ☐ |
| CHK-016 | 月次 RTO ≤ 3600 秒（全 12 回） | ☐ |
| CHK-017 | Outbox 遅延最大値 ≤ 5 分（年間） | ☐ |
| CHK-018 | Step P95 ≤ 200ms（年間最大値） | ☐ |
| CHK-019 | 週次ヘルスチェック実施率 ≥ 95% | ☐ |
| CHK-020 | バックアップ全世代存在確認 | ☐ |
| CHK-021 | 月次リストア検証 12 回実施 | ☐ |
| CHK-022 | 鍵分離確認（backup dir に鍵なし） | ☐ |
| CHK-023 | 監査ログ連続性（欠落日ゼロ） | ☐ |
| CHK-024 | ハッシュチェーン整合性（invalid=0） | ☐ |
| CHK-025 | Outbox DLQ 年間累積ゼロ | ☐ |
| CHK-026 | XES エクスポート 1 回以上実施 | ☐ |
| CHK-027 | JWT 鍵ローテーション 4 回確認 | ☐ |
| CHK-028 | 有効期限切れ active 鍵ゼロ | ☐ |
| CHK-029 | TLS 証明書有効期限 30 日以上 | ☐ |
| CHK-030 | 依存脆弱性ゼロ（Rust + Node） | ☐ |
| CHK-031 | 四半期依存更新レビュー 4 回実施 | ☐ |
| CHK-032 | OS セキュリティパッチ 90 日以内適用 | ☐ |
| CHK-033 | ADR 有効性確認 | ☐ |
| CHK-034 | SOP 改訂履歴整合 | ☐ |
| CHK-035 | 手順書改訂記録存在 | ☐ |
| CHK-036 | 保守記録完全性（全 log_type 期待件数） | ☐ |
| CHK-037 | inactive 作業者に有効 JWT なし | ☐ |
| CHK-038 | PII 匿名化スケジュール未処理ゼロ | ☐ |
| CHK-039 | 入退職記録 件数整合 | ☐ |
| CHK-040 | インシデント記録件数整合 | ☐ |
| CHK-041 | 縮退運用合計時間 ≤ 40 時間 | ☐ |
| CHK-042 | XES ファイル存在・SHA256 整合 | ☐ |

全 28 項目 PASS で年次自己監査合格とする。

**本節で確定した方針**
- 全 28 項目の評価が完了しなければ年次自己監査完了と見なさないことを確定する。

---

## 6. 異常時の判断

| 事象 | 打ち切り条件 | 通知先 | 代替手順 |
|---|---|---|---|
| ハッシュチェーン不整合（CHK-024 FAIL） | 即時打ち切り | system_admin・quality_admin | 監査ログ改ざん調査を開始 |
| inactive 作業者に有効 JWT 残存（CHK-037 FAIL） | 継続（即時 OPS-PROC-005 §4.2.1 実施） | system_admin | JWT 即時失効 |
| PII 匿名化スケジュール期限超過（CHK-038 FAIL） | 継続（即時 OPS-PROC-005 §4.2.5/6 実施） | system_admin・quality_admin | 即時匿名化実施 |

**本節で確定した方針**
- ハッシュチェーン不整合はセキュリティインシデントとして扱い即時打ち切りとすることを確定する。

---

## 7. 終了条件と記録

- [SQL] maintenance_log への INSERT
  ```sql
  INSERT INTO maintenance_log (log_type, executed_at, executed_by, detail)
  VALUES (
    'annual_audit',
    NOW(),
    'system_admin',
    '{"result": "pass", "year": 2026, "total_checks": 28, "pass": 28, "fail": 0, "report_path": "/opt/wnav/reports/annual-audit/annual-audit-2026.md", "reviewed_by": "quality_admin"}'
  );
  ```

**本節で確定した方針**
- quality_admin のレビュー署名なしに年次監査完了と見なさないことを確定する。

---

## 8. ロールバック / 代替手順

本手順書は評価・記録（読み取り専用）が大半であるためロールバックは発生しない。
即時対応が必要な FAIL 項目は個別の対応手順（OPS-PROC-005 等）に委任する。

**本節で確定した方針**
- 年次自己監査はロールバック不要の評価手順であることを確定する。

---

## 9. 関連識別子・改訂履歴

| 属性 | 内容 |
|---|---|
| **関連 BAT** | BAT-001〜010（全バッチジョブの実績確認対象） |
| **関連 ALERT** | ALERT-001〜005（年間アラート発火状況の確認対象） |
| **関連 ERR** | — |
| **関連 KEY** | KEY-001〜005（鍵管理全体の確認対象） |
| **関連 ADR-IMPL** | — |
| **初版** | 2026-05-18 RyuheiKiso |

---

## 参照業界分析

### 必須
- IPA 共通フレーム 2013 SLCP-JCF2013 4.2.1.c（業務及びシステムの運用）

### 関連
- ISO/IEC 27001:2022 A.5.35（情報セキュリティの独立したレビュー）
- IPA「情報セキュリティ自己点検シート」（中小企業向け）
- NIST SP 800-53 Rev.5 CA-7（継続的な監視）、AU-11（監査ログ保持）
- NFR-OPS-042（本プロジェクト要件定義）
