# 13 IQC 領域論理設計

本章は IQC（入荷検査）機能領域の論理設計を確定する。EN-028〜031 の 4 エンティティを中心に、AQL 判定ロジック・後工程ハードゲート・特採/選別フローの実装方針を設計命題として記述する。

---

## 1. エンティティ関係概要

EN-028 Material と EN-029 Supplier は多対多の取引関係を持つ（同一材料を複数仕入先から調達可能）。入荷時に EN-030 IncomingInspection が生成され、EN-031 SamplingPlan を時点参照して n/Ac/Re を決定する。検査完了後は TBL-042 lot_qc_states に qc_status を反映し、後工程の TBL-024 lots QR スキャン時のゲート判定に使用する。

---

## 2. 設計確定事項

### 2-1. AQL マスタの時点固定

EN-031 SamplingPlan の `aql_table_snapshot JSONB` に JIS Z 9015-1 のサンプル文字表と AQL マスタ表を検査計画作成時点で焼き付ける（既存の `sop_version_id` 時点参照原則を踏襲）。インポート後の AQL 規格改訂は既存計画に影響しない。

### 2-2. 後工程ハードゲート（FR-IQ-009）

TBL-024 lots の `qc_status` が `PASSED` または `CONDITIONAL_PASS`（特採有効期限内）の場合のみ、SOP 実行時の材料 QR スキャン API（API-step-events-001 拡張）が step_completed を受け付ける。それ以外は `ERR-BIZ-015`（lot_not_qc_passed）を返す。

特採時は黄色警告バナーを強制表示（dismissable 不可）し、適用範囲・期限を表示する（FR-IQ-010）。

### 2-3. 検査厳しさ状態機械（JIS Z 9015-1 §10）

| 遷移 | 条件 | CFG 参照 |
|---|---|---|
| なみ → きつい | 連続 CFG-020（デフォルト 2）回不合格 | CFG-020 |
| きつい → なみ | 連続 CFG-021（デフォルト 5）回合格 | CFG-021 |
| なみ → ゆるい | 連続 CFG-022（デフォルト 10）回合格 | CFG-022 |
| ゆるい → なみ | 1 回でも不合格 | — |

状態は `TBL-038.severity_state ENUM('NORMAL','TIGHTENED','REDUCED')` で保持する。

### 2-4. 4 区分判定フロー

AQL 判定で REJECTED の場合、品質担当（quality_admin ロール）が 4 区分から判定を選択する:

1. **CONCESSION（特採）**: 有効期限・有効範囲・理由テキストを必須入力。`TBL-041 concession_approvals` に Append-only 記録。
2. **SCREENING_REQUIRED（選別）**: 全数検査後に合格個体を `parent_lot_id` 付き子ロットとして発行（TBL-024 lots に `parent_lot_id` FK）。
3. **REJECTED（返品）**: `TBL-050 return_to_vendor_records` に追跡番号・運送業者を Append-only 記録。
4. **SCRAPPED（廃却）**: `TBL-049 scrap_records` に廃棄物処理票・立会者サインを Append-only 記録。

---

## 3. 個人別集計禁止（BR-BUS-038）

仕入先品質実績は仕入先 × 材料 × 月次の集計のみを API リソースとして提供する（BAT-012）。`incoming_inspections.inspector_id` 単位の集計クエリは RBAC で quality_admin ロールに対しても提供しない（NFR-ETH-002 遵守）。

---

## 4. wnav_iqc BC 境界

IQC BC は NonConformity BC と明確に分離する:
- **IN**: `lots` の受入ロット（supplier_id / material_id 拡張）が IQC の起点
- **OUT**: `lot_qc_states.qc_status` の更新が後工程ゲートを制御
- **イベント**: `IqcJudged`、`LotQcStateChanged`、`ConcessionApproved` の 3 ドメインイベントを Outbox Pattern で発行

---

## 参照業界分析

### 必須

[`90_業界分析/06_品質管理とトレーサビリティ.md`](../../../90_業界分析/06_品質管理とトレーサビリティ.md)

[`90_業界分析/11_計測・工程能力と統計的品質工学.md`](../../../90_業界分析/11_計測・工程能力と統計的品質工学.md)

### 関連

[`90_業界分析/17_サプライチェーンと作業依存性.md`](../../../90_業界分析/17_サプライチェーンと作業依存性.md)

[`90_業界分析/33_計量法・JCSS校正トレーサビリティとSI単位.md`](../../../90_業界分析/33_計量法・JCSS校正トレーサビリティとSI単位.md)
