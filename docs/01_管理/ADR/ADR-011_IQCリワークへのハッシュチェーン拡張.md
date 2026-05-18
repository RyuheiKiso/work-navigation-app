# ADR-011: IQC・リワーク Append-only テーブルへのハッシュチェーン拡張

**状態**: 採択
**日付**: 2026-05-18
**担当サブ**: 05_詳細設計/01_データベース詳細設計

## 背景

TBL-001 work_events は SHA-256 ハッシュチェーン（prev_hash / content_hash 列 + 週次 BAT-001 検証）により ALCOA+ Original 要件（改ざん検知・真正性証明）を担保している。

一方、IQC / リワーク系の Append-only テーブル群（TBL-038/040/041/044/045/047/049/050 の 8 テーブル）には同等の機構が存在しなかった。これらのテーブルは入荷受入検査記録・特採承認・ディスポジション・リワーク検証・廃棄記録など、製造 GMP・ISO 9001 の観点から改ざん耐性が求められる品質記録を保持する。

DB ロールレベルの `REVOKE UPDATE, DELETE`（第一層）と `electronic_signs` 紐付け（承認証跡・第二層）のみでは、DB レコード自体の改ざんを事後的に検知する手段が欠如していた。本 ADR でハッシュチェーン（第三層）を追加し、三層防御を完成させる。

## 決定事項

**IQC・リワーク Append-only テーブル 8 件に `qc_case_id` / `prev_hash` / `content_hash` 列を追加し、per qc_case_id genesis 方式の独立ハッシュチェーンを適用する。**

### 対象テーブルと qc_case_id 割当規約

| TBL-ID | テーブル名 | qc_case_id 値 | ハッシュ対象主列 |
|---|---|---|---|
| TBL-038 | incoming_inspections | self.inspection_id（genesis） | lot_id, supplier_id, material_id, inspector_id, received_at, qc_status |
| TBL-040 | incoming_inspection_measurements | inspection_id | inspection_id, sample_no, measured_value, defect_flag, measured_at |
| TBL-041 | concession_approvals | inspection_id | inspection_id, decision, reason, approver_id, approved_at |
| TBL-044 | dispositions | nonconformity_id | nonconformity_id, decision, quality_admin_sign_id, supervisor_sign_id, decided_at |
| TBL-045 | rework_verifications | rework_id | rework_id, verifier_id, verdict, verified_at |
| TBL-047 | reworked_lot_labels | rework_id | rework_id, giai, parent_lot_id, issued_by, issued_at |
| TBL-049 | scrap_records | rework_id | rework_id, witness_id, recorded_at |
| TBL-050 | return_to_vendor_records | rework_id | rework_id, vendor_id, tracking_no, returned_at |

### 共通追加列定義

```sql
qc_case_id    UUID     NOT NULL,    -- チェーン単位
prev_hash     CHAR(64) NOT NULL,    -- 前ブロックの content_hash（genesis は "0"×64）
content_hash  CHAR(64) NOT NULL,    -- 本レコードの SHA-256
CONSTRAINT ck_{tbl}_hash_length CHECK (length(prev_hash) = 64 AND length(content_hash) = 64)
```

### ADR-010（後埋め列禁止）との整合

`prev_hash` / `content_hash` は INSERT 時にアプリ層（`wnav_master_api`）で計算し確定する値であり、INSERT 後に UPDATE で設定する「後埋め列」ではない。ADR-010 の禁止規約に違反しない。

### Genesis 設計（ADR-007 拡張）

ADR-007（per case_id genesis）の思想を継承する。各 qc_case_id の最初のレコードが genesis となり、`prev_hash = "0"×64`、`content_hash = SHA-256(qc_case_id || primary_fields)` を持つ。同一 qc_case_id 内で 2 件目以降のレコードは前レコードの `content_hash` を `prev_hash` として受け取る。

### BAT-001 拡張

`wnav_hash_chain::HashChainService` に `verify_inspection_chain_all()` と `verify_rework_chain_all()` を追加し、週次スケジューラ（BAT-001）で work_events チェーン検証の後に順次実行する。

### 除外対象と理由

| TBL-ID | テーブル名 | 除外理由 |
|---|---|---|
| TBL-043 | reworks | status 列が UPDATE 可（限定可変）のためハッシュチェーン不適合 |
| TBL-042 | lot_qc_states | 更新可テーブル |
| TBL-048 | rework_cost_records | BAT-011 が日次で上書きするためハッシュチェーン不適合 |

## 根拠（代替案との比較表）

| 代替案 | メリット | デメリット | 採否 |
|---|---|---|---|
| **A案（採択）: IQC テーブルに独立チェーン追加** | work_events と同等の第三層改ざん検知を IQC にも適用。BAT-001 を拡張するだけで既存チェーン計算ロジック（FNC-BE-009/010）を再利用できる | 8 テーブルへの DDL 変更とマイグレーション 1 本が必要。Rust 側に FNC-BE-018〜020 を追加実装する必要がある | **採択** |
| B案: work_events に IQC activity を追加して既存チェーンに統合 | DDL 変更最小。チェーンが 1 本に統一される | work_events の意味論（作業実行イベント）が入荷検査・品質判定まで膨らむ。case_id 設計（ADR-007）との整合が崩れる | 却下 |
| C案: ver1.0.0 では見送り、ADR 記録のみ | 変更コストゼロ | ALCOA+ Original 要件の三層目が未実装のまま残り、品質記録の完全性が担保できない。本プロジェクトの「妥協禁止・プロダクション品質」ポリシーに反する | 却下 |

## 結果

- TBL-038/040/041/044/045/047/049/050 の 8 テーブルに prev_hash / content_hash / qc_case_id 列を追加
- `wnav_hash_chain` クレートに FNC-BE-018（compute_content_hash_for_inspection）・FNC-BE-019（compute_content_hash_for_rework）・FNC-BE-020（compute_content_hash_for_disposition）を追加
- BAT-001 スケジューラに IQC チェーン週次検証を追加
- マイグレーション `V20260518XXXXXX__add_hash_chain_to_iqc_tables.sql` を追加
- IQC 品質記録に対して三層防御（DBロール制御 + 電子サイン承認 + ハッシュチェーン）が完成

## 参照

- [`ADR-007_per_case_id_genesis採用.md`](ADR-007_per_case_id_genesis採用.md) — per genesis 方式の原則
- [`ADR-008_補正レコード継続規則.md`](ADR-008_補正レコード継続規則.md) — チェーン破断時の補正方式
- [`ADR-010_Append-only後埋め列禁止規約.md`](ADR-010_Append-only後埋め列禁止規約.md) — 後埋め列禁止との整合確認
- `05_詳細設計/01_データベース詳細設計/02_トランザクション系テーブルDDL.md` — DDL 全文（指摘5対応）
- `05_詳細設計/02_バックエンド詳細設計/06_wnav_hash_chain詳細設計.md` — FNC-BE-018〜020 設計

## 更新履歴

| 日付 | 変更 | 変更者 |
|---|---|---|
| 2026-05-18 | 初版（DB 詳細設計レビューで IQC の第三層改ざん防止欠落を発見し対応）| RyuheiKiso |
