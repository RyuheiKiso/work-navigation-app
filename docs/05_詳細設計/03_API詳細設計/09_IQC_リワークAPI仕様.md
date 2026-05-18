# 09 IQC・リワーク API 仕様

本章は IQC（入荷検査）とリワーク（修正作業）機能の REST API を確定する。API 共通仕様（RFC 9457 エラー・Idempotency-Key・JWT 認証）は `01_OpenAPI共通仕様と設計原則.md` に準拠する。

> **担当バイナリ方針**:
> - IQC 検査実施系（`POST /api/v1/iqc/incoming-inspections`、測定値入力）→ **terminal-api**（現場端末からの入荷検査実施）
> - IQC 合否判定・特採承認（`POST /api/v1/iqc/incoming-inspections/{id}/judge`、`/api/v1/iqc/incoming-inspections/{id}/concession`）→ **master-api**（品質管理者による判定・承認）
> - リワーク作業実施（`POST /api/v1/reworks`）→ **terminal-api**（現場端末からのリワーク作業実施）
> - ディスポジション承認（`POST /api/v1/dispositions`）→ **master-api**（品質管理者・監督者による承認）
> - 廃却記録（`POST /api/v1/scrap-records`）→ **master-api**
> - 返却記録（`POST /api/v1/return-records`）→ **master-api**
> - 再検査記録（`POST /api/v1/rework-verifications`）→ **terminal-api**（operator / quality_admin が現場で実施）

---

## 1. IQC API

### 1-1. POST /api/v1/iqc/incoming-inspections（API-iqc-001）

入荷ロット受入登録を行い、対象 lot の sampling_plan を解決して incoming_inspections レコードを生成する。

| 項目 | 内容 |
|---|---|
| API-ID | API-iqc-001 |
| HTTP Method | POST |
| URL | `/api/v1/iqc/incoming-inspections` |
| 担当バイナリ | terminal-api（現場端末からの入荷ロット受入登録）|
| 必要ロール | operator（IQC）/ quality_admin |
| Idempotency | Idempotency-Key ヘッダ必須 |
| 関連 FR | FR-IQ-001, FR-IQ-002, BR-BUS-032 |

**リクエスト本文**:

```json
{
  "lot_id": "019682ab-7c1f-7000-b1c2-3d4e5f6a7b8c",
  "supplier_id": "019682ab-7c1f-7000-b1c2-aabbcc000001",
  "material_id": "019682ab-7c1f-7000-b1c2-aabbcc000002",
  "lot_quantity": 1000
}
```

**レスポンス（201 Created）**:

```json
{
  "inspection_id": "019682ab-7c1f-7000-b1c2-ddccbb000001",
  "sampling_plan_id": "019682ab-7c1f-7000-b1c2-plan00000001",
  "sample_size_n": 80,
  "accept_number_ac": 3,
  "reject_number_re": 4,
  "severity_state": "NORMAL",
  "qc_status": "PENDING"
}
```

### 1-2. POST /api/v1/iqc/incoming-inspections/{id}/measurements（API-iqc-003）

サンプル測定値を 1 個ずつ登録する（Append-only）。

| 項目 | 内容 |
|---|---|
| API-ID | API-iqc-003 |
| HTTP Method | POST |
| URL | `/api/v1/iqc/incoming-inspections/{id}/measurements` |
| 担当バイナリ | terminal-api（現場端末からの測定値入力）|
| 必要ロール | operator（IQC）/ quality_admin |
| 関連 FR | FR-IQ-005, FR-IQ-006 |

**リクエスト本文**:

```json
{
  "sample_no": 1,
  "measured_value": 12.345,
  "defect_flag": false,
  "evidence_file_id": null
}
```

### 1-3. POST /api/v1/iqc/incoming-inspections/{id}/judge（API-iqc-004）

AQL 自動合否判定を実行し qc_status を確定する。不良数 ≤ Ac → PASSED。不良数 ≥ Re → REJECTED。

| 項目 | 内容 |
|---|---|
| API-ID | API-iqc-004 |
| HTTP Method | POST |
| URL | `/api/v1/iqc/incoming-inspections/{id}/judge` |
| 担当バイナリ | master-api（品質管理者による AQL 合否判定）|
| 必要ロール | quality_admin |
| 関連 FR | FR-IQ-007, FR-IQ-008 |

エラー: `ERR-BIZ-016`（検査未実施）、`ERR-BIZ-017`（判定済）、`ERR-VAL-030`（測定数 < n）

### 1-4. POST /api/v1/iqc/incoming-inspections/{id}/concession（API-iqc-005）

特採（CONCESSION）承認を行い concession_approvals を Append-only で記録する。

| 項目 | 内容 |
|---|---|
| API-ID | API-iqc-005 |
| HTTP Method | POST |
| URL | `/api/v1/iqc/incoming-inspections/{id}/concession` |
| 担当バイナリ | master-api（品質管理者による特採承認）|
| 必要ロール | quality_admin |
| 関連 FR | FR-IQ-010 |

**リクエスト本文**:

```json
{
  "reason": "寸法公差外だが強度試験は合格。品質担当が有効範囲を限定して承認。",
  "validity_scope": {"processes": ["PROC-001"], "max_quantity": 200},
  "valid_until": "2026-08-31",
  "electronic_sign_id": "019682ab-7c1f-7000-b1c2-sign00000001"
}
```

---

## 2. リワーク API

### 2-1. POST /api/v1/dispositions（API-dispositions-001）

ディスポジション判定を Append-only で記録する。Two-Person Integrity を DB トリガで保証。

| 項目 | 内容 |
|---|---|
| API-ID | API-dispositions-001 |
| HTTP Method | POST |
| URL | `/api/v1/dispositions` |
| 担当バイナリ | master-api（品質管理者・監督者による Two-Person Integrity 承認）|
| 必要ロール | quality_admin（quality_admin_sign）+ supervisor（supervisor_sign）|
| Idempotency | Idempotency-Key ヘッダ必須 |
| 関連 FR | FR-ST-013, FR-EV-014, BR-BUS-040 |

**リクエスト本文**:

```json
{
  "nonconformity_id": "019682ab-7c1f-7000-b1c2-nc000000001",
  "decision": "REWORK",
  "decision_reason": "当該不適合は修正可能と判断。リワーク実施を承認する。",
  "quality_admin_sign_id": "019682ab-7c1f-7000-b1c2-sign000qa01",
  "supervisor_sign_id": "019682ab-7c1f-7000-b1c2-sign000sup1"
}
```

エラー: `ERR-BIZ-021`（同一署名者）、`ERR-BIZ-019`（判定済）

### 2-2. POST /api/v1/reworks（API-reworks-001）

リワーク作業を開始し reworks レコードを作成する。

| 項目 | 内容 |
|---|---|
| API-ID | API-reworks-001 |
| HTTP Method | POST |
| URL | `/api/v1/reworks` |
| 担当バイナリ | terminal-api（現場端末からのリワーク作業開始）|
| 必要ロール | operator / supervisor |
| 関連 FR | FR-ST-014, BR-BUS-041 |

エラー: `ERR-BIZ-022`（リワーク上限超過）

### 2-3. POST /api/v1/rework-verifications（API-rework-verifications-001）

再検査を実施し rework_verifications を Append-only で記録する。

| 項目 | 内容 |
|---|---|
| API-ID | API-rework-verifications-001 |
| HTTP Method | POST |
| URL | `/api/v1/rework-verifications` |
| 担当バイナリ | terminal-api（operator / quality_admin が現場で再検査実施）|
| 必要ロール | operator / quality_admin |
| 関連 FR | FR-EV-015, BR-BUS-042 |

エラー: `ERR-BIZ-023`（再検査者同一）

### 2-4. POST /api/v1/scrap-records（API-scrap-records-001）

廃却処理を Append-only で記録する。

| 項目 | 内容 |
|---|---|
| API-ID | API-scrap-records-001 |
| HTTP Method | POST |
| URL | `/api/v1/scrap-records` |
| 担当バイナリ | master-api（品質管理者による廃却処理承認）|
| 必要ロール | quality_admin |
| 関連 FR | FR-MA-017, BR-BUS-043 |

エラー: `ERR-BIZ-024`（立会者同一）

### 2-5. POST /api/v1/return-records（API-return-records-001）

仕入先返却を Append-only で記録する。

| 項目 | 内容 |
|---|---|
| API-ID | API-return-records-001 |
| HTTP Method | POST |
| URL | `/api/v1/return-records` |
| 担当バイナリ | master-api（品質管理者による仕入先返却処理）|
| 必要ロール | quality_admin |
| 関連 FR | FR-MA-018, BR-BUS-044 |

エラー: `ERR-BIZ-025`（追跡番号未入力）

---

---

**本節で確定した方針**
- **IQC 検査実施系（API-iqc-001: 入荷登録・API-iqc-003: 測定値入力）およびリワーク作業実施（API-reworks-001）・再検査記録（API-rework-verifications-001）は `wnav_terminal_api`（ポート 8080）が担当する。**
- **IQC 合否判定（API-iqc-004）・特採承認（API-iqc-005）・ディスポジション承認（API-dispositions-001）・廃却記録（API-scrap-records-001）・返却記録（API-return-records-001）は `wnav_master_api`（ポート 8081）が担当する。承認・判定系はすべて管理コンソールからの操作に限定する。**

---

## 参照業界分析

### 必須

[`90_業界分析/06_品質管理とトレーサビリティ.md`](../../../../90_業界分析/06_品質管理とトレーサビリティ.md)

[`90_業界分析/28_不適合と手順改訂のフィードバックループ.md`](../../../../90_業界分析/28_不適合と手順改訂のフィードバックループ.md)

### 関連

[`90_業界分析/13_安全文化と安全管理システム.md`](../../../../90_業界分析/13_安全文化と安全管理システム.md)
