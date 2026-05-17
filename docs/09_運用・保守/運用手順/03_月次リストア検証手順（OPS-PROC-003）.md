# 03 月次リストア検証手順（OPS-PROC-003）

本手順書の責務は月次リストア検証を実施し RTO を計測・確定することである。上流要件 NFR-OPS-046・NFR-AVL-015/016/020（`docs/04_概要設計/08_運用方式設計/07_アカウント・変更管理と運用手順.md`）を手順に具体化する。IPA 共通フレーム 2013「4.2.1.c 業務及びシステムの運用」に準拠する。

---

## 1. 目的と上流要件

| 属性 | 内容 |
|---|---|
| **手順 ID** | OPS-PROC-003 |
| **頻度** | 月次（月初第 1 営業日、09:00 開始） |
| **想定所要時間** | P50: 40 分 / P95: 60 分（RTO 上限） |
| **実施権限** | system_admin（必須） |

上流要件:
- NFR-OPS-046: 月次でバックアップからのリストア検証を実施し RTO を記録すること
- NFR-AVL-015: RTO（Recovery Time Objective）≤ 60 分を保証すること
- NFR-AVL-016: RPO（Recovery Point Objective）≤ 5 分（WAL PITR による）を保証すること
- NFR-AVL-020: ハッシュチェーン整合性を復元後に必ず確認すること

**本節で確定した方針**
- 月次リストア検証は本番環境と分離した検証環境（別プロジェクト名）で実施することを確定する。
- RTO 計測は本手順書 §4.1 の `T0` セットから §4.9 の `T1` セットまでを計測範囲とすることを確定する。
- RTO > 60 分は検証失敗とし即時 system_admin が原因調査を実施することを確定する。

---

## 2. 前提条件チェックリスト

以下をすべて確認してから手順を開始する。1 つでも NG なら手順を開始しない。

- [ ] 本番環境（`wnav-prod`）の Docker Compose が稼働中である（検証は別プロジェクトで行うため本番影響なし）
- [ ] 最新バックアップファイル `/backup/db/daily/*.dump.gz.enc` が存在する
- [ ] SHA256 チェックファイル `/backup/db/latest.sha256` が存在する
- [ ] KEY-002（バックアップ復号鍵）の参照先パスが利用可能である
- [ ] 検証用ポート（5433 番）が未使用である
- [ ] `/tmp/wnav-restore-YYYYMMDD/` を作成するための空き容量が 10GB 以上ある

**本節で確定した方針**
- 前提条件チェックリストに 1 つでも NG がある場合は手順を開始しないことを確定する。

---

## 3. 事前準備

- [CMD]
  ```bash
  export PROJECT=wnav-restore-$(date +%Y%m%d)
  export RESTORE_DIR="/tmp/${PROJECT}"
  mkdir -p "${RESTORE_DIR}"
  echo "Restore project: ${PROJECT}"
  echo "Restore dir: ${RESTORE_DIR}"
  ```

- [CMD] 検証用 Docker Compose ファイルをコピーして準備
  ```bash
  cp /opt/wnav/docker-compose.restore.yml "${RESTORE_DIR}/docker-compose.yml"
  # ポートを 5433 に変更して本番と分離
  sed -i 's/5432:5432/5433:5432/g' "${RESTORE_DIR}/docker-compose.yml"
  ```

**本節で確定した方針**
- 検証環境は本番環境とポート・プロジェクト名を必ず分離することを確定する。

---

## 4. 実施手順

以下の操作タグを使用する。
- `[CMD]` シェルコマンド（WSL2 + bash）
- `[SQL]` PostgreSQL クエリ（psql 経由）
- `[PS]` PowerShell（IIS / Windows Server 操作）
- `[GUI]` ブラウザ / Grafana / 管理 UI 操作
- `[CHECK]` 確認・検証操作

### 4.1 ステップ 1: RTO 計測開始

- [CMD]
  ```bash
  T0=$(date +%s)
  echo "RTO_START=$(date -d @${T0} '+%Y-%m-%d %H:%M:%S')" | tee "${RESTORE_DIR}/rto.log"
  ```

### 4.2 ステップ 2: 検証環境セットアップ

- [CMD]
  ```bash
  cd "${RESTORE_DIR}"
  docker compose up -d postgres
  # PostgreSQL 起動待ち（最大 60 秒）
  for i in $(seq 1 12); do
    pg_isready -h localhost -p 5433 -U work_nav && break
    echo "Waiting for PostgreSQL... ${i}/12"
    sleep 5
  done
  ```

- [CHECK] `pg_isready` が `accepting connections` を返すこと。

### 4.3 ステップ 3: バックアップ復号

- [CMD]
  ```bash
  BACKUP_ENC=$(ls -t /backup/db/daily/*.dump.gz.enc | head -1)
  echo "Restoring from: ${BACKUP_ENC}"

  # SHA256 チェック
  sha256sum -c /backup/db/latest.sha256 && echo "SHA256: OK"

  # AES-256-GCM 復号
  openssl enc -d -aes-256-gcm \
    -kfile /etc/wnav/keys/backup-key.bin \
    -in "${BACKUP_ENC}" \
    -out "${RESTORE_DIR}/restore.dump.gz"

  # 解凍
  gunzip -c "${RESTORE_DIR}/restore.dump.gz" > "${RESTORE_DIR}/restore.dump"
  echo "Decryption complete: $(ls -lh ${RESTORE_DIR}/restore.dump)"
  ```

- [CHECK] exit 0 かつ `restore.dump` のサイズが 0 より大きいこと。

### 4.4 ステップ 4: PostgreSQL 起動と pg_restore

- [CMD]
  ```bash
  # 空データベースを作成
  docker compose exec postgres \
    psql -U work_nav -c "CREATE DATABASE work_navigation_restore;"

  # リストア実行
  docker compose exec -T postgres \
    pg_restore -U work_nav -d work_navigation_restore \
    --no-owner --no-acl -v < "${RESTORE_DIR}/restore.dump" \
    2>&1 | tee "${RESTORE_DIR}/pg_restore.log"
  ```

- [CHECK] `pg_restore.log` に `error:` が含まれないこと（`warning:` は許容）。

### 4.5 ステップ 5: CHK-010 — PITR 適用（オプション）

PITR による RPO 検証が必要な場合のみ実施する。

- [CMD]
  ```bash
  # WAL アーカイブのコピー
  cp -r /backup/wal/ "${RESTORE_DIR}/wal/"

  # PostgreSQL recovery.conf を設定（コンテナ内）
  docker compose exec postgres bash -c "
    cat > /var/lib/postgresql/data/recovery.conf << 'EOF'
restore_command = 'cp /wal/%f %p'
recovery_target_time = '$(date -u '+%Y-%m-%d %H:%M:%S') UTC'
recovery_target_action = 'promote'
EOF
  "
  docker compose restart postgres
  ```

- [CHECK] `pg_isready` が再度 `accepting connections` を返すこと。

### 4.6 ステップ 6: CHK-011 — ハッシュチェーン整合性検証

- [SQL]
  ```sql
  -- psql で検証環境（port 5433）に接続
  -- \c work_navigation_restore
  SELECT count(*) AS total,
         sum(CASE WHEN hash_chain_valid THEN 1 ELSE 0 END) AS valid_count,
         sum(CASE WHEN NOT hash_chain_valid THEN 1 ELSE 0 END) AS invalid_count
  FROM (
    SELECT
      id,
      prev_hash,
      LAG(content_hash) OVER (PARTITION BY entity_type ORDER BY created_at) AS expected_prev_hash,
      (prev_hash = LAG(content_hash) OVER (PARTITION BY entity_type ORDER BY created_at)
       OR LAG(content_hash) OVER (PARTITION BY entity_type ORDER BY created_at) IS NULL) AS hash_chain_valid
    FROM audit_logs
    ORDER BY created_at
  ) sub;
  ```

- [CHECK] `invalid_count = 0` であること。

### 4.7 ステップ 7: CHK-012 — 代表 SOP ナビゲーション動作確認

- [CMD]
  ```bash
  # API を検証環境 DB に向けて起動
  cd "${RESTORE_DIR}"
  DATABASE_URL="postgres://work_nav:password@localhost:5433/work_navigation_restore" \
    docker compose up -d api

  sleep 10
  curl -fsS http://localhost:8081/health | jq .
  ```

- [SQL]
  ```sql
  -- 代表 SOP が存在することを確認
  SELECT sop_id, title, version, status
  FROM sop_master
  WHERE status = 'active'
  LIMIT 5;
  ```

- [CHECK] API ヘルスチェックが `{"status":"ok"}` を返すこと。
  SOP レコードが 1 件以上存在すること。

### 4.8 ステップ 8: CHK-013 — XES エクスポート確認

- [SQL]
  ```sql
  -- 監査ログのエクスポート用データが揃っているか確認
  SELECT count(*) AS audit_count,
         min(created_at) AS oldest,
         max(created_at) AS newest
  FROM audit_logs;
  ```

- [CHECK] `audit_count > 0` かつ `oldest` と `newest` の範囲がバックアップ日時と整合することを確認する。

### 4.9 ステップ 9: RTO 計測終了

- [CMD]
  ```bash
  T1=$(date +%s)
  RTO_SEC=$((T1 - T0))
  RTO_MIN=$(echo "scale=1; ${RTO_SEC}/60" | bc)
  echo "RTO_END=$(date -d @${T1} '+%Y-%m-%d %H:%M:%S')" | tee -a "${RESTORE_DIR}/rto.log"
  echo "RTO_SEC=${RTO_SEC}" | tee -a "${RESTORE_DIR}/rto.log"
  echo "RTO_MIN=${RTO_MIN}" | tee -a "${RESTORE_DIR}/rto.log"
  echo "RESULT=$([ ${RTO_SEC} -le 3600 ] && echo PASS || echo FAIL)" | tee -a "${RESTORE_DIR}/rto.log"
  ```

- [CHECK] `RTO_SEC ≤ 3600`（60 分以内）であること。

**本節で確定した方針**
- ステップ 1〜9 を順序通りに実施することを確定する。
- RTO が 3600 秒（60 分）を超えた場合は即時 §6 の判断に従うことを確定する。

---

## 5. 合格基準

| CHK-ID | 基準 | 合否 |
|---|---|---|
| CHK-010 | pg_restore が error なしで完了する | ☐ |
| CHK-011 | ハッシュチェーン `invalid_count = 0` | ☐ |
| CHK-012 | API ヘルスチェック 200 OK かつ SOP 1 件以上存在 | ☐ |
| CHK-013 | 監査ログ `audit_count > 0` かつ 日時範囲整合 | ☐ |
| CHK-014 | RTO ≤ 3600 秒（60 分） | ☐ |

全 CHK が合格でリストア検証完了とする。

**本節で確定した方針**
- 全 5 項目が合格でなければリストア検証完了と見なさないことを確定する。

---

## 6. 異常時の判断

| 事象 | 打ち切り条件 | 通知先 | 代替手順 |
|---|---|---|---|
| SHA256 ミスマッチ | 即時打ち切り | system_admin | バックアップファイルを隔離し OPS-PROC-002 §6.1 で再 dump |
| pg_restore エラー | 即時打ち切り | system_admin | pg_restore ログ解析・別世代バックアップで再試行 |
| ハッシュチェーン不整合 | 即時打ち切り | system_admin・quality_admin | 監査ログ改ざん調査を開始 |
| RTO > 3600 秒 | 打ち切りなし（結果 FAIL で記録） | system_admin | 原因分析（ネットワーク・ストレージ I/O）を実施 |

**本節で確定した方針**
- SHA256 ミスマッチ・pg_restore エラー・ハッシュチェーン不整合は即時打ち切りとすることを確定する。

---

## 7. 終了条件と記録

- [CMD] 検証環境クリーンアップ
  ```bash
  cd "${RESTORE_DIR}"
  docker compose down -v
  cd /
  rm -rf "${RESTORE_DIR}"
  echo "Cleanup complete"
  ```

- [SQL] maintenance_log への INSERT（本番環境の psql で実行）
  ```sql
  INSERT INTO maintenance_log (log_type, executed_at, executed_by, detail)
  VALUES (
    'restore_verification',
    NOW(),
    'system_admin',
    '{"result": "pass", "rto_sec": 1800, "chk_010": "pass", "chk_011": "pass", "chk_012": "pass", "chk_013": "pass", "chk_014": "pass", "backup_date": "2026-05-01"}'
  );
  ```

  実際の `rto_sec` には `${RESTORE_DIR}/rto.log` の `RTO_SEC` 値を入れること。

**本節で確定した方針**
- 検証環境は必ず手順完了後にクリーンアップすることを確定する。
- `maintenance_log` への記録なしに検証完了と見なさないことを確定する。

---

## 8. ロールバック / 代替手順

本手順書の検証は本番環境に影響を与えない独立環境で実施するためロールバックは不要である。
検証中に本番 DB への誤接続を防ぐため、検証環境の `DATABASE_URL` にポート 5433 を指定することを徹底する。

**本節で確定した方針**
- 検証環境は本番環境とネットワーク・ポートレベルで分離し、誤接続を構造的に防止することを確定する。

---

## 9. 関連識別子・改訂履歴

| 属性 | 内容 |
|---|---|
| **関連 BAT** | BAT-001（日次 pg_dump）、BAT-002（WAL アーカイブ） |
| **関連 ALERT** | — |
| **関連 ERR** | — |
| **関連 KEY** | KEY-002（バックアップ復号鍵） |
| **関連 ADR-IMPL** | — |
| **初版** | 2026-05-18 RyuheiKiso |

---

## 参照業界分析

### 必須
- IPA 共通フレーム 2013 SLCP-JCF2013 4.2.1.c（業務及びシステムの運用）

### 関連
- PostgreSQL 公式ドキュメント「25.3. Continuous Archiving and Point-in-Time Recovery (PITR)」
- NIST SP 800-34 Rev.1「Contingency Planning Guide」§3.4.2（リストアテスト）
- ISO/IEC 27001:2022 Annex A 8.13（情報のバックアップ）
- NFR-OPS-046、NFR-AVL-015/016/020（本プロジェクト要件定義）
