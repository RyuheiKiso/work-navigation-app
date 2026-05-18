# 02 物理テーブル一覧（TBL カタログ）

本章の責務は、全物理テーブル（TBL-001〜TBL-035）の名称・対応論理エンティティ・種別・推定行数・保存期間・担当章を一覧化することである。このカタログは DTM M2（FR × TBL）の参照源であり、設計完了時には全 EN-NNN が少なくとも 1 つの TBL にマッピングされていることを保証する。

---

## 1. TBL カタログ全量（TBL-001〜035）

| TBL-ID | 物理テーブル名 | 対応 EN | 対応関係 | 種別 | 推定行数/年 | 保存期間 |
|---|---|---|---|---|---|---|
| TBL-001 | work_events | EN-012 | 1:1 | **Append-only** | 1,500 万行 | 7 年以上 |
| TBL-002 | electronic_signs | EN-015 | 1:1 | **Append-only** | 50 万行 | 7 年以上 |
| TBL-003 | outbox_events | EN-021 | 1:1 | **Append-only** | 1,500 万行（SENT: 90 日後アーカイブ）| 90 日〜1 年 |
| TBL-004 | master_versions | EN-010 | 1:1 | 更新可（状態のみ）| 1 万行 | 永続 |
| TBL-005 | work_executions | EN-011 | 1:1 | 更新可 | 100 万行 | 7 年以上 |
| TBL-006 | work_orders | EN-011（ワークオーダー部分）| 1:N EN-011 | マスタ | 10 万行 | 永続 |
| TBL-007 | sops | EN-008 | 1:1 | マスタ（版管理）| 1 万行 | 永続 |
| TBL-008 | steps | EN-009 | 1:1 | マスタ（版管理）| 10 万行 | 永続 |
| TBL-009 | evidence_files | EN-013 | 1:1 | **Append-only** | 500 万行 | 7 年以上 |
| TBL-010 | measurements | EN-014 | 1:1 | **Append-only** | 200 万行 | 7 年以上 |
| TBL-011 | suspensions | EN-016 | 1:1 | **Append-only** | 10 万行 | 7 年以上 |
| TBL-012 | andon_alerts | EN-017 | 1:1 | 更新可 | 5 万行 | 5 年以上 |
| TBL-013 | nonconformities | EN-018 | 1:1 | 更新可 | 1 万行 | 7 年以上 |
| TBL-014 | capas | EN-019 | 1:1 | 更新可 | 5,000 行 | 7 年以上 |
| TBL-015 | kaizen_proposals | EN-020 | 1:1 | 更新可 | 1 万行 | 5 年以上 |
| TBL-016 | users | EN-001 | 1:1 | マスタ | 1,000 行 | 退職 60 日後に匿名化 |
| TBL-017 | roles | EN-002 | 1:1 | マスタ（固定 6 種）| 6 行 | 永続 |
| TBL-018 | skills | EN-003 | 1:1 | マスタ | 1,000 行 | 永続 |
| TBL-019 | user_roles | EN-001 × EN-002 | N:M 中間 | マスタ | 3,000 行 | 永続 |
| TBL-020 | user_skills | EN-001 × EN-003 | N:M 中間 | マスタ | 5,000 行 | 永続 |
| TBL-021 | processes | EN-005 | 1:1 | マスタ | 500 行 | 永続 |
| TBL-022 | operations | EN-006 | 1:1 | マスタ | 2,000 行 | 永続 |
| TBL-023 | products | EN-007 | 1:1 | マスタ | 1,000 行 | 永続 |
| TBL-024 | lots | EN-021（Lot）| 1:1 | マスタ | 50 万行 | 7 年以上 |
| TBL-025 | equipments | EN-019（Equipment）| 1:1 | マスタ | 500 行 | 永続 |
| TBL-026 | instruments | EN-024（Instrument・校正器）| 1:1 | マスタ | 200 行 | 永続 |
| TBL-027 | external_key_bindings | EN-022（ExternalKeyBinding）| 1:1 | **Append-only**（有効期間管理）| 1 万行 | 永続 |
| TBL-028 | work_patterns | EN-025（WorkPattern）| 1:1 | マスタ | 1,000 行 | 永続 |
| TBL-029 | step_type_definitions | EN-026 | 1:1 | マスタ（版管理）| 100 行 | 永続 |
| TBL-030 | step_flow_rules | EN-027 | 1:1 | マスタ（版管理）| 1,000 行 | 永続 |
| TBL-031 | hash_chain_blocks | EN-025（HashChainBlock）| 1:1 | **Append-only** | 1,000 行（週次 1 ブロック）| 7 年以上 |
| TBL-032 | auth_logs | EN-022（AuthLog）| 1:1 | **Append-only** | 50 万行 | 90 日 |
| TBL-033 | devices | EN-023（Device）| 1:1 | マスタ | 100 行 | 廃棄時削除 |
| TBL-034 | device_sync_states | EN-024（DeviceSyncState）| 1:1 | 更新可 | 100 行 | Device と同期 |
| TBL-035 | idempotency_keys | （制御テーブル・EN なし）| — | 制御 | 100 万行（TTL 24h）| 24 時間（自動削除）|

**TBL-025 equipments 拡張注記**: scan_code（スキャン照合 ID）/ tool_subtype（工具/治具サブ種別）/ calibration_due_date（治具点検期限 NULL 許容）を追加保持する。ポカヨケ照合のマスタとして機能する。

---

## 1b. IQC / リワーク 追加テーブル（TBL-036〜050）

IQC 機能（機能ランキング 10 位）とリワーク機能（同 9 位）の導入に伴い、TBL-036〜050 を追加する。  
既存テーブル拡張（TBL-005/007/013/014/024）は §1c に記載する。

| TBL-ID | 物理テーブル名 | 対応 EN | 対応関係 | 種別 | 推定行数/年 | 保存期間 |
|---|---|---|---|---|---|---|
| TBL-036 | materials | EN-028 | 1:1 | マスタ（版管理）| 500 行 | 永続 |
| TBL-037 | suppliers | EN-029 | 1:1 | マスタ（版管理）| 100 行 | 永続 |
| TBL-038 | incoming_inspections | EN-030 | 1:1 | 限定可変（qc_status のみ更新可）| 5 万行 | 7 年以上 |
| TBL-039 | sampling_plans | EN-031 | 1:1 | マスタ（版管理・JSONB スナップショット）| 5,000 行 | 永続 |
| TBL-040 | incoming_inspection_measurements | EN-030（詳細）| 1:N EN-030 | **Append-only** | 100 万行 | 7 年以上 |
| TBL-041 | concession_approvals | EN-030（承認詳細）| 1:N EN-030 | **Append-only** | 1 万行 | 7 年以上 |
| TBL-042 | lot_qc_states | EN-030 × EN-021 | 1:1 lots FK | 状態管理（qc_status 更新可）| 50 万行 | lots と同期 |
| TBL-043 | reworks | EN-032 | 1:1 | 限定可変（status のみ更新可）| 5 万行 | 7 年以上 |
| TBL-044 | dispositions | EN-033 | 1:1 | **Append-only** | 5 万行 | 7 年以上 |
| TBL-045 | rework_verifications | EN-034 | 1:1 | **Append-only** | 5 万行 | 7 年以上 |
| TBL-046 | rework_sop_mapping | EN-035 | 1:1 | マスタ（版管理）| 1,000 行 | 永続 |
| TBL-047 | reworked_lot_labels | EN-036 | 1:1 | **Append-only** | 5 万行 | 7 年以上 |
| TBL-048 | rework_cost_records | EN-037 | 1:1 | 集計値（BAT-011 上書き）| 5 万行 | 7 年以上 |
| TBL-049 | scrap_records | EN-038 | 1:1 | **Append-only** | 5,000 行 | 7 年以上 |
| TBL-050 | return_to_vendor_records | EN-039 | 1:1 | **Append-only** | 5,000 行 | 7 年以上 |

**TBL-038 incoming_inspections 設計注記**: `qc_status ENUM('PENDING','INSPECTING','PASSED','CONDITIONAL_PASS','SCREENING_REQUIRED','REJECTED','SCRAPPED','RETURNED')` + `severity_state ENUM('NORMAL','TIGHTENED','REDUCED')` を保有。AQL 合否判定は §1-4 の NFR-DQ-005 準拠。

**TBL-039 sampling_plans 設計注記**: `aql_table_snapshot JSONB` に JIS Z 9015-1 サンプル文字表・AQL マスタ表を時点固定で焼き付ける（既存の時点参照原則を踏襲）。

**TBL-044 dispositions 設計注記**: `quality_admin_sign_id FK→electronic_signs` と `supervisor_sign_id FK→electronic_signs` の 2 FK を持つ。DB トリガ `check_disposition_distinct_signers` で 2 つの signer_id が必ず異なることを検証する（NFR-SEC-048）。

**TBL-043 reworks 設計注記**: `rework_type ENUM('TOUCH_UP','REWORK_FULL','SORTING','SCRAP','RETURN')`。`parent_case_id FK→work_executions` と `rework_case_id NULL FK→work_executions` の双方向 FK で ALCOA+ Original 不変性（NFR-DQ-010）を実装する。

---

## 1d. 排他制御追加テーブル（TBL-051）

Case 端末排他占有機能（FR-SY-011 / ADR-009）の導入に伴い、TBL-051 を追加する。

| TBL-ID | 物理テーブル名 | 対応 EN | 対応関係 | 種別 | 推定行数/年 | 保存期間 |
|---|---|---|---|---|---|---|
| TBL-051 | case_locks | （制御テーブル・EN なし）| — | 制御（例外: INSERT/UPDATE/DELETE 許可）| 実行中 case 数分（常時数十行未満）| ロック期間中のみ（解放後 DELETE）|

**TBL-051 case_locks 設計注記**: 端末側 LocalCaseLock エンティティと対応あり。heartbeat_at が 5 分超過した ACTIVE レコードは BAT-013 が自動 EXPIRED 化する。`ON CONFLICT (case_id) DO NOTHING` による atomic な占有試行で排他性を保証する。

---

## 1c. 既存テーブル拡張（IQC/リワーク対応）

| TBL-ID | テーブル名 | 追加列 | 目的 |
|---|---|---|---|
| TBL-005 | work_executions | `execution_type ENUM('NORMAL','REWORK','VERIFICATION','IQC') DEFAULT 'NORMAL'`、`source_rework_id NULL FK→reworks`、`source_inspection_id NULL FK→incoming_inspections` | リワーク/IQC 実行との関連付け |
| TBL-007 | sops | `sop_type ENUM('NORMAL','REWORK','INSPECTION','SCRAP_RECORD','RETURN_RECORD','IQC') DEFAULT 'NORMAL'` | SOP の用途分類 |
| TBL-013 | nonconformities | `disposition_id NULL FK→dispositions` | ディスポジション判定との連携 |
| TBL-014 | capas | `rework_summary JSONB` | リワーク結果の CAPA への集約 |
| TBL-024 | lots | `supplier_id NULL FK→suppliers`、`material_id NULL FK→materials`、`qc_status ENUM`、`rework_history_count INT DEFAULT 0`、`parent_lot_id NULL FK→lots` | 入荷検査結果・リワーク履歴の管理 |

---

## 2. Append-only テーブルの物理保証

以下のテーブルは `INSERT` のみを許可する。`UPDATE` / `DELETE` は PostgreSQL ロールレベルで禁止する。

| TBL-ID | テーブル名 | ロール制御 |
|---|---|---|
| TBL-001 | work_events | `GRANT INSERT, SELECT ON work_events TO app_event_writer;` |
| TBL-002 | electronic_signs | 同上 |
| TBL-003 | outbox_events | `INSERT` + `UPDATE（status 列のみ）` を許可（送信状態更新が必要なため）|
| TBL-009 | evidence_files | `INSERT, SELECT` のみ |
| TBL-010 | measurements | `INSERT, SELECT` のみ |
| TBL-011 | suspensions | `INSERT, SELECT` のみ |
| TBL-021 | outbox_events | （TBL-003 と同一）|
| TBL-027 | external_key_bindings | `INSERT, SELECT` のみ（廃棄は valid_to 更新のみ）|
| TBL-031 | hash_chain_blocks | `INSERT, SELECT` のみ |
| TBL-032 | auth_logs | `INSERT, SELECT` のみ |
| TBL-040 | incoming_inspection_measurements | `INSERT, SELECT` のみ |
| TBL-041 | concession_approvals | `INSERT, SELECT` のみ |
| TBL-044 | dispositions | `INSERT, SELECT` のみ（DB トリガで 2 署名者異一性を検証）|
| TBL-045 | rework_verifications | `INSERT, SELECT` のみ |
| TBL-047 | reworked_lot_labels | `INSERT, SELECT` のみ |
| TBL-049 | scrap_records | `INSERT, SELECT` のみ |
| TBL-050 | return_to_vendor_records | `INSERT, SELECT` のみ |

詳細は §05（イベントストア設計）と `07_セキュリティ方式設計/07_脆弱性管理.md` で確定する。

---

## 3. ストレージ容量見積もり

| カテゴリ | 主要テーブル | 5 年累積推定 |
|---|---|---|
| 作業イベント（TBL-001） | 1,500 万行/年 × 5 = 7,500 万行 | 約 120GB（1,600 bytes/行想定）|
| 証拠ファイル参照（TBL-009） | 500 万行/年 × 5 | 約 15GB（メタデータのみ・バイナリは NAS）|
| その他トランザクション | — | 約 30GB |
| マスタ系 | — | 約 2GB |
| **合計（DB）** | — | **約 167GB**（NFR-PRF-015: 1.5TB 以下に大きく余裕あり）|

証拠ファイル（写真等のバイナリ）は NAS に格納し、TBL-009 はファイルパスと SHA-256 ハッシュのみ保持する。バイナリ含む推定総ストレージは約 500GB（5 年）。

---

**本節で確定した方針**
- **全 51 物理テーブル（TBL-001〜051）を確定し、全 39 論理エンティティ（EN-001〜039）が ≥1 TBL にマッピングされることを保証した。**
- **Append-only テーブル 17 件を明示し、PostgreSQL ロール分離によって物理 DELETE/UPDATE を禁止することを設計命題として確定した。**
- **5 年累積ストレージを約 167GB（DB）と見積もり、NFR-PRF-015（1.5TB 以下）を十分に満足することを確認した。**（IQC/リワーク追加分は年 10GB 未満と試算、見積もりは余裕あり）
- **TBL-025 equipments に scan_code / tool_subtype / calibration_due_date を追加し、新規 TBL を発行せずに EN-019 Equipment 拡張を確定する。**
- **TBL-036〜050（IQC/リワーク 15 テーブル）を追加。TBL-044 dispositions に DB トリガ `check_disposition_distinct_signers` を設け、NFR-SEC-048（二者電子サイン分離）を物理レベルで保証することを確定する。**
- **TBL-043 reworks は `parent_case_id` と `rework_case_id` の双方向 FK で ALCOA+ Original 原則（NFR-DQ-010）を実装することを確定する。**
- **TBL-051 case_locks を追加し、1 case_id = 1 端末の排他占有制御テーブルとして確定した。heartbeat UPDATE・解放 DELETE が必要なため app_event_insert ロールへの INSERT/UPDATE/DELETE 許可を例外として認める（FR-SY-011 / ADR-009）。**

---

## 参照業界分析

### 必須
- [`90_業界分析/06_品質管理とトレーサビリティ.md`](../../90_業界分析/06_品質管理とトレーサビリティ.md)
- [`90_業界分析/27_オフライン同期とデータ整合性.md`](../../90_業界分析/27_オフライン同期とデータ整合性.md)

### 関連
- [`90_業界分析/22_規制別トレーサビリティ要件詳論.md`](../../90_業界分析/22_規制別トレーサビリティ要件詳論.md)
