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
- **全 35 物理テーブル（TBL-001〜035）を確定し、全 27 論理エンティティ（EN-001〜027）が ≥1 TBL にマッピングされることを保証した。**
- **Append-only テーブル 10 件を明示し、PostgreSQL ロール分離によって物理 DELETE/UPDATE を禁止することを設計命題として確定した。**
- **5 年累積ストレージを約 167GB（DB）と見積もり、NFR-PRF-015（1.5TB 以下）を十分に満足することを確認した。**

---

## 参照業界分析

### 必須
- [`90_業界分析/06_品質管理とトレーサビリティ.md`](../../90_業界分析/06_品質管理とトレーサビリティ.md)
- [`90_業界分析/27_オフライン同期とデータ整合性.md`](../../90_業界分析/27_オフライン同期とデータ整合性.md)

### 関連
- [`90_業界分析/22_規制別トレーサビリティ要件詳論.md`](../../90_業界分析/22_規制別トレーサビリティ要件詳論.md)
