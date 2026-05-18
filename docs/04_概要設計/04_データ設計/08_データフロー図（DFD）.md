# 08 データフロー図（DFD）

本章の責務は、システム全体のデータフロー（端末 → Outbox → PostgreSQL → NAS・親機 の主要パス）を確定することである。DFD は IPA 2.3「配置設計」タスクのデータ観点での可視化であり、`01_システム方式設計/03_配置設計.md` のインフラ観点と補完関係にある。

バックエンドは **terminal-api（port 8080）** と **master-api（port 8081）** の 2 バイナリ構成である。フロー 1〜2・5（端末起点の書き込み・Outbox 送信）は terminal-api を経由し、フロー 3・6（マスタ同期・管理操作・監査照会）は master-api を経由する。

**図 1: データフロー全体図（Level 0 DFD）**

![図 1 データフロー全体図（Level 0 DFD）](img/fig_des_db_dfd_overall.svg)

> 原本: [`img/fig_des_db_dfd_overall.drawio`](img/fig_des_db_dfd_overall.drawio)

---

## 1. データフロー全体（Level 0 DFD）

主要なデータフローを 6 経路に整理する。

```
[端末（Android/iOS/Windows）]
         │
         │ 1. 作業記録フロー（TBL-001 work_events）
         │    端末 SQLite → Outbox（端末側 TBL-003 相当）
         │    → terminal-api（Rust/axum, 8080） → PostgreSQL
         │
         │ 2. 証拠ファイルフロー（TBL-009 evidence_files）
         │    端末カメラ → SHA-256 計算
         │    → terminal-api（Rust/axum, 8080） → NAS（バイナリ）＆ PostgreSQL（メタデータ）
         │
         │ 3. マスタ同期フロー（端末 → バックエンド）
         │    master-api（Rust/axum, 8081） PostgreSQL 差分抽出 → 端末 SQLite キャッシュ
         │
[terminal-api（Rust/axum, 8080）]  ─── フロー 1・2 担当
│    作業ログ受信 / 証拠記録受信 / Outbox 送信
│
[master-api（Rust/axum, 8081）]    ─── フロー 3・管理操作担当
│    マスタ配信 / 監査照会 / 管理操作
│
         │ 4. 実績送信フロー（子機モード）
         │    PostgreSQL → terminal-api Outbox tokio task → 親機 API（IF-002）
         │
         │ 5. バックアップフロー
         │    PostgreSQL WAL → NAS（PITR）・pg_dump → オフサイト NAS
         ↓
[親機（ERP/MES 等）]
```

---

## 2. データフロー詳細

### 2-1. 作業記録フロー（オンライン時）

担当バイナリ: **terminal-api（8080）**

```
端末操作（Step 完了等）
  → [端末] WorkEvent 生成（UUID v7・timestamp_client 記録）
  → [端末 SQLite] OutboxEvent INSERT（status = PENDING）
  → [Outbox Consumer tokio task（terminal-api 内）] ポーリング検出
  → [terminal-api] POST /api/v1/work-executions/{id}/events（API-step-events-001）
  → [terminal-api] Idempotency Key 検証（TBL-035）
  → [terminal-api] timestamp_server 付与
  → [PostgreSQL TBL-001] work_events INSERT
  → [PostgreSQL TBL-001] ハッシュチェーン更新（prev_hash 計算）
  → [terminal-api] 200 OK レスポンス
  → [端末 SQLite] OutboxEvent.status → SENT
```

### 2-2. 作業記録フロー（オフライン時）

担当バイナリ: **terminal-api（8080）**（回復後に §2-1 へ合流）

```
端末操作（Offline モード）
  → [端末 SQLite TBL-001 相当] WorkEvent INSERT（timestamp_client のみ）
  → [端末 SQLite Outbox] OutboxEvent INSERT（status = PENDING, is_offline = TRUE）
  → ネットワーク回復後、Outbox Consumer tokio task（terminal-api 内）が PENDING を検出し §2-1 フローへ
  ※ timestamp_server はサーバー到着時刻（オフライン記録の Contemporaneous 補足）
```

### 2-3. 証拠ファイルフロー

担当バイナリ: **terminal-api（8080）**

```
端末カメラ撮影
  → [端末] SHA-256 計算（file_hash）
  → [端末] Exif 削除
  → [terminal-api] POST /api/v1/evidences（multipart, API-evidences-001）
  → [terminal-api] NAS への書き込み（/files/{uuid}.{ext}）
  → [PostgreSQL TBL-009] evidence_files INSERT（file_path + file_hash のみ）
  → バイナリファイルは PostgreSQL に格納しない（NAS 専用）
```

### 2-4. マスタ同期フロー（子機初回・差分）

担当バイナリ: **master-api（8081）**

```
端末起動または定期同期（CFG-007: 60 分間隔）
  → [端末] GET /api/v1/sync/master?as_of={last_sync_ts}（API-sync-001）
  → [master-api] master_versions テーブルから差分抽出
  → [master-api] JSON パッケージ（圧縮）を返却
  → [端末 SQLite] マスタキャッシュ更新（sops/steps/processes 等）
  → [端末] last_sync_at 更新（TBL-034 相当）
```

### 2-5. 実績送信フロー（子機モード・IF-002）

担当バイナリ: **terminal-api（8080）**（Outbox 送信 tokio task）

```
Outbox 送信 tokio task（terminal-api 内）の定期実行
  → [PostgreSQL TBL-003] PENDING 行を SENDING に UPDATE
  → [terminal-api] POST {親機エンドポイント}/inbound（HMAC-SHA256 署名付き）
  → [親機] 受信確認（HTTP 200）
  → [terminal-api] TBL-003 status → SENT、sent_at 記録
  → 失敗時: retry_count++ / 3 回失敗で status → DLQ
  → DLQ: Webhook リトライ tokio task（terminal-api 内）が再投入試行
```

### 2-6. 監査照会フロー

担当バイナリ: **master-api（8081）**（監査ログの READ は master-api の app_read pool を使用）

```
quality_admin / system_admin が監査照会を要求
  → [master-api] GET /api/v1/audit/export/xes（API-audit-export-001）
  → [master-api] app_read pool 経由で TBL-032 audit_logs を参照（READ のみ）
  → [master-api] XES 形式で出力（.xes.gz）
  ※ terminal-api は audit_logs への WRITE（WorkEvent INSERT）を行うが、閲覧 API は持たない
```

---

## 3. データ滞留点と滞留時間上限

| 滞留点 | 最大滞留時間 | 根拠 NFR |
|---|---|---|
| 端末 SQLite Outbox | 無制限（ネットワーク回復まで）| Offline-First 原則（P1）|
| TBL-003 PENDING | CFG-003（バックオフ最大: 60 分）× CFG-002（最大 5 回）| IF-002 再試行ポリシー |
| TBL-003 DLQ | 1 年（NFR-OPS-035 の DLQ 保管）| 手動介入まで |
| NAS 未同期（バックアップ）| 5 分（PITR: WAL 5 分間隔）| NFR-AVL-015（RPO 15 分以内の NAS 部分）|

---

**本節で確定した方針**
- **データフロー 6 経路（作業記録・証拠ファイル・マスタ同期・実績送信・バックアップ・監査照会）を DFD Level 0 として確定した。フロー 1〜2・5 は terminal-api（8080）、フロー 3・6（管理・照会系）は master-api（8081）が担当することを明示した。**
- **バックエンドの scheduler サービスは廃止し、Outbox Consumer・Webhook リトライ等のスケジュール実行タスクは各バイナリ内の tokio task に内包する設計を確定した。**
- **証拠ファイル（バイナリ）は PostgreSQL に格納せず NAS 専用とし、SHA-256 とファイルパスのみを TBL-009 に保存する設計を確定した。**
- **オフライン記録時の二重タイムスタンプ（端末時刻 = timestamp_client・サーバー受信 = timestamp_server）を確定し、ALCOA+ Contemporaneous を技術的に担保する。**
- **監査ログ（TBL-032）の READ は master-api の app_read pool のみが担当し、terminal-api は監査ログへの WRITE（WorkEvent INSERT）のみを行う責務分離を確定した。**

---

## 参照業界分析

### 必須
- [`90_業界分析/27_オフライン同期とデータ整合性.md`](../../90_業界分析/27_オフライン同期とデータ整合性.md)

### 関連
- [`90_業界分析/21_作業ログ分析とプロセスマイニング.md`](../../90_業界分析/21_作業ログ分析とプロセスマイニング.md)
- [`90_業界分析/06_品質管理とトレーサビリティ.md`](../../90_業界分析/06_品質管理とトレーサビリティ.md)
