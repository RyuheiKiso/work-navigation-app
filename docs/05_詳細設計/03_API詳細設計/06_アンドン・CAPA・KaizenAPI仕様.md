# 06 アンドン・CAPA・Kaizen API 仕様

本章は API-andon-001〜002（アンドンアラート）・API-capa-001〜002（CAPA）・API-kaizen-001（改善提案）に加え、採番台帳未収録の非適合品登録（POST /api/v1/nonconformities）を補足仕様として確定する。

---

## 1. API-andon-001: POST /api/v1/alerts

### 1-1. 概要

| 項目 | 値 |
|---|---|
| API-ID | API-andon-001 |
| HTTP メソッド | POST |
| URL | `/api/v1/alerts` |
| 認証要否 | 必須 |
| Idempotency-Key | 必須 |
| レート制限カテゴリ | 書き込み（500 req / 60s）|
| 関連 FR | FR-KZ-001 |

### 1-2. リクエストスキーマ

```json
{
  "alert_type": "quality",
  "severity": "high",
  "work_execution_id": "019682ab-7c1f-7000-b1c2-3d4e5f6a7b8c",
  "step_id": "019682ab-7c1f-7000-0000-000000000602",
  "raised_by": "019682ab-7c1f-7000-0000-000000000002",
  "title": "溶接部にクラック発見",
  "description": "手順 6 実施中、溶接部に 2mm 程度のクラックを発見。",
  "timestamp_client": "2026-05-17T09:45:00.123Z"
}
```

| フィールド | 型 | 必須 | 制約 | 説明 |
|---|---|---|---|---|
| `alert_type` | string | 必須 | `quality` / `safety` / `equipment` / `process` | アラート種別 |
| `severity` | string | 必須 | `low` / `medium` / `high` / `critical` | 重大度 |
| `work_execution_id` | string (UUID v7) | 任意 | TBL-005 に存在 | 関連する作業実行 ID |
| `step_id` | string (UUID v7) | 任意 | TBL-008 に存在 | 関連するステップ ID |
| `raised_by` | string (UUID v7) | 必須 | TBL-016 に存在 | 起票者 ID |
| `title` | string | 必須 | 1〜200 文字 | アラートタイトル |
| `description` | string | 必須 | 1〜2000 文字 | 詳細説明 |
| `timestamp_client` | string (ISO 8601) | 必須 | — | クライアント側の発生時刻 |

### 1-3. レスポンススキーマ（HTTP 201）

```json
{
  "data": {
    "alert_id": "019682ab-7c1f-7000-h1i2-3j4k5l6m7n8o",
    "alert_type": "quality",
    "severity": "high",
    "status": "open",
    "work_execution_id": "019682ab-7c1f-7000-b1c2-3d4e5f6a7b8c",
    "raised_by": "019682ab-7c1f-7000-0000-000000000002",
    "title": "溶接部にクラック発見",
    "raised_at": "2026-05-17T09:45:00.000Z",
    "notification_sent": true
  },
  "meta": {
    "request_id": "019682ab-7c1f-7040-a1b2-3c4d5e6f7890",
    "server_time": "2026-05-17T09:45:00.000Z",
    "api_version": "v1"
  }
}
```

| フィールド | 型 | 説明 |
|---|---|---|
| `alert_id` | string (UUID v7) | アラート ID（TBL-012）|
| `status` | string | 常に `open`（作成直後）|
| `notification_sent` | boolean | supervisor への通知送信結果 |

`severity: critical` の場合は TBL-003（Outbox）に MSG-005（`internal.alert_triggered`）を挿入し、factory の supervisor / quality_admin 全員に通知する。

### 1-4. RBAC

全ロール（operator を含む）がアラートを起票可能。

### 1-5. エラーコード

| ERR-CODE | HTTP | 発生条件 |
|---|---|---|
| ERR-AUTH-001 | 401 | JWT 無効 |
| ERR-VAL-001 | 422 | 必須フィールド不足 |
| ERR-VAL-004 | 422 | description が 2000 文字超 |

---

## 2. API-andon-002: PATCH /api/v1/alerts/{id}/acknowledge

### 2-1. 概要

| 項目 | 値 |
|---|---|
| API-ID | API-andon-002 |
| HTTP メソッド | PATCH |
| URL | `/api/v1/alerts/{id}/acknowledge` |
| 認証要否 | 必須 |
| Idempotency-Key | 必須 |
| 関連 FR | FR-KZ-002 |

### 2-2. リクエストスキーマ

```json
{
  "acknowledged_by": "019682ab-7c1f-7000-0000-000000000051",
  "acknowledgement_note": "現場に向かいます。",
  "timestamp_client": "2026-05-17T09:47:00.123Z"
}
```

| フィールド | 型 | 必須 | 説明 |
|---|---|---|---|
| `acknowledged_by` | string (UUID v7) | 必須 | 確認者 ID（supervisor 以上）|
| `acknowledgement_note` | string | 任意 | 確認コメント（最大 500 文字）|
| `timestamp_client` | string (ISO 8601) | 必須 | クライアント側の確認時刻 |

### 2-3. レスポンススキーマ（HTTP 200）

```json
{
  "data": {
    "alert_id": "019682ab-7c1f-7000-h1i2-3j4k5l6m7n8o",
    "status": "acknowledged",
    "acknowledged_by": "019682ab-7c1f-7000-0000-000000000051",
    "acknowledged_at": "2026-05-17T09:47:00.000Z"
  },
  "meta": {
    "request_id": "019682ab-7c1f-7041-a1b2-3c4d5e6f7890",
    "server_time": "2026-05-17T09:47:00.000Z",
    "api_version": "v1"
  }
}
```

### 2-4. アラート解決（補足仕様）

採番台帳未収録だが、アラートクローズのエンドポイントを補足として定義する。

```
POST /api/v1/alerts/{id}/resolve
```

```json
{
  "resolved_by": "019682ab-7c1f-7000-0000-000000000060",
  "resolution_note": "クラック箇所を補修し品質確認完了。",
  "nonconformity_id": "019682ab-7c1f-7000-p1q2-3r4s5t6u7v8w",
  "timestamp_client": "2026-05-17T11:00:00.123Z"
}
```

`supervisor` / `quality_admin` のみ実行可。`status` を `resolved` に変更する。

---

## 3. 非適合品登録 補足仕様: POST /api/v1/nonconformities

採番台帳に個別エントリはないが、CAPA との連携に必要なため補足仕様として定義する。

### 3-1. リクエストスキーマ

```json
{
  "alert_id": "019682ab-7c1f-7000-h1i2-3j4k5l6m7n8o",
  "work_execution_id": "019682ab-7c1f-7000-b1c2-3d4e5f6a7b8c",
  "lot_id": "019682ab-7c1f-7000-0000-000000000401",
  "nc_type": "process_deviation",
  "description": "溶接トルク値が規定値の 15% 超過",
  "discovered_by": "019682ab-7c1f-7000-0000-000000000060",
  "discovery_step_id": "019682ab-7c1f-7000-0000-000000000605",
  "evidence_ids": ["019682ab-7c1f-7000-f1a2-3b4c5d6e7f8a"],
  "timestamp_client": "2026-05-17T10:00:00.123Z"
}
```

| フィールド | 型 | 必須 | 制約 | 説明 |
|---|---|---|---|---|
| `alert_id` | string (UUID v7) | 任意 | TBL-012 に存在 | 関連アラート ID |
| `work_execution_id` | string (UUID v7) | 任意 | TBL-005 に存在 | 関連作業実行 ID |
| `lot_id` | string (UUID v7) | 任意 | TBL-024 に存在 | 関連ロット ID |
| `nc_type` | string | 必須 | `process_deviation` / `material_defect` / `measurement_out_of_spec` / `document_error` | 非適合種別 |
| `description` | string | 必須 | 1〜2000 文字 | 非適合内容説明 |
| `discovered_by` | string (UUID v7) | 必須 | quality_admin / supervisor | 発見者 ID |
| `evidence_ids` | array of string | 任意 | 各要素が UUID v7、TBL-009 に存在 | 関連エビデンス ID 一覧 |

レスポンス HTTP 201。TBL-013（nonconformities）にレコードを挿入する。

---

## 4. API-capa-001: POST /api/v1/capas

### 4-1. 概要

| 項目 | 値 |
|---|---|
| API-ID | API-capa-001 |
| HTTP メソッド | POST |
| URL | `/api/v1/capas` |
| 認証要否 | 必須 |
| Idempotency-Key | 必須 |
| 関連 FR | FR-KZ-003 |

### 4-2. リクエストスキーマ

```json
{
  "nonconformity_id": "019682ab-7c1f-7000-p1q2-3r4s5t6u7v8w",
  "title": "溶接工程トルク管理手順の改定",
  "root_cause_analysis": "手順書のトルク値記載が曖昧であったため、作業者が誤った値を適用した。",
  "corrective_action": "手順書 5.3 項のトルク値を数値明示（28Nm ± 2Nm）に改定する。",
  "preventive_action": "マスタ更新時の数値フィールド必須チェックを追加する。",
  "assigned_to": "019682ab-7c1f-7000-0000-000000000050",
  "due_date": "2026-06-17",
  "created_by": "019682ab-7c1f-7000-0000-000000000060",
  "timestamp_client": "2026-05-17T11:30:00.123Z"
}
```

| フィールド | 型 | 必須 | 制約 | 説明 |
|---|---|---|---|---|
| `nonconformity_id` | string (UUID v7) | 任意 | TBL-013 に存在 | 起因となった非適合品 ID |
| `title` | string | 必須 | 1〜200 文字 | CAPA タイトル |
| `root_cause_analysis` | string | 必須 | 1〜5000 文字 | 根本原因分析 |
| `corrective_action` | string | 必須 | 1〜5000 文字 | 是正処置内容 |
| `preventive_action` | string | 任意 | 最大 5000 文字 | 再発防止処置内容 |
| `assigned_to` | string (UUID v7) | 必須 | TBL-016 に存在 | 担当者 ID |
| `due_date` | string (ISO 8601 date) | 必須 | 本日以降の日付 | 完了予定日 |
| `created_by` | string (UUID v7) | 必須 | quality_admin ロール保有者 | 作成者 ID |

### 4-3. レスポンススキーマ（HTTP 201）

```json
{
  "data": {
    "capa_id": "019682ab-7c1f-7000-x1y2-3z4a5b6c7d8e",
    "status": "open",
    "title": "溶接工程トルク管理手順の改定",
    "assigned_to": "019682ab-7c1f-7000-0000-000000000050",
    "due_date": "2026-06-17",
    "created_at": "2026-05-17T11:30:00.000Z"
  },
  "meta": {
    "request_id": "019682ab-7c1f-7042-a1b2-3c4d5e6f7890",
    "server_time": "2026-05-17T11:30:00.000Z",
    "api_version": "v1"
  }
}
```

TBL-014（capas）にレコードを挿入する。

### 4-4. RBAC

`quality_admin` / `system_admin` のみ作成可。

---

## 5. API-capa-002: PATCH /api/v1/capas/{id}

### 5-1. 概要

| 項目 | 値 |
|---|---|
| API-ID | API-capa-002 |
| HTTP メソッド | PATCH |
| URL | `/api/v1/capas/{id}` |
| 認証要否 | 必須 |
| Idempotency-Key | 必須 |
| 関連 FR | FR-KZ-003 |

### 5-2. リクエストスキーマ（部分更新）

```json
{
  "status": "in_progress",
  "progress_note": "手順書改定作業を開始。5/25 完成予定。",
  "updated_by": "019682ab-7c1f-7000-0000-000000000050",
  "timestamp_client": "2026-05-20T10:00:00.123Z"
}
```

| フィールド | 型 | 必須 | 制約 | 説明 |
|---|---|---|---|---|
| `status` | string | 任意 | `open` → `in_progress` → `pending_verification` → `closed` | 新しいステータス |
| `progress_note` | string | 任意 | 最大 2000 文字 | 進捗メモ |
| `corrective_action` | string | 任意 | 最大 5000 文字 | 是正処置内容の更新 |
| `preventive_action` | string | 任意 | 最大 5000 文字 | 防止処置内容の更新 |
| `due_date` | string (ISO 8601 date) | 任意 | 本日以降 | 完了予定日の変更 |
| `updated_by` | string (UUID v7) | 必須 | TBL-016 に存在 | 更新者 ID |
| `timestamp_client` | string (ISO 8601) | 必須 | — | クライアント側の更新時刻 |

`status: "closed"` への変更は `quality_admin` のみ可。既に `closed` の CAPA への PATCH は ERR-BIZ-008。

### 5-3. エラーコード

| ERR-CODE | HTTP | 発生条件 |
|---|---|---|
| ERR-AUTH-004 | 403 | 権限不足（closed は quality_admin 以外）|
| ERR-BIZ-008 | 409 | CAPA が既に closed |
| ERR-VAL-001 | 422 | updated_by 不足 |

---

## 6. API-kaizen-001: POST /api/v1/kaizen-proposals

### 6-1. 概要

| 項目 | 値 |
|---|---|
| API-ID | API-kaizen-001 |
| HTTP メソッド | POST |
| URL | `/api/v1/kaizen-proposals` |
| 認証要否 | 必須 |
| Idempotency-Key | 必須 |
| 関連 FR | FR-KZ-007 |

### 6-2. リクエストスキーマ

```json
{
  "proposer_id": "019682ab-7c1f-7000-0000-000000000002",
  "process_id": "019682ab-7c1f-7000-0000-000000000201",
  "category": "efficiency",
  "title": "ステップ 8 の確認待ち時間を短縮する",
  "current_situation": "ステップ 8 完了後、次ステップの材料準備まで平均 3 分待機が発生している。",
  "proposal_detail": "材料準備をステップ 7 実施中に並行して行うよう手順を変更する。",
  "expected_benefit": "工程時間を 3 分短縮。月間換算で 120 分の削減効果。",
  "related_sop_id": "019682ab-7c1f-7000-0000-000000000301",
  "evidence_ids": [],
  "timestamp_client": "2026-05-17T12:30:00.123Z"
}
```

| フィールド | 型 | 必須 | 制約 | 説明 |
|---|---|---|---|---|
| `proposer_id` | string (UUID v7) | 必須 | TBL-016 に存在 | 提案者 ID |
| `process_id` | string (UUID v7) | 任意 | TBL-021 に存在 | 対象工程 ID |
| `category` | string | 必須 | `efficiency` / `safety` / `quality` / `cost` / `environment` | 改善カテゴリ |
| `title` | string | 必須 | 1〜200 文字 | 提案タイトル |
| `current_situation` | string | 必須 | 1〜2000 文字 | 現状説明 |
| `proposal_detail` | string | 必須 | 1〜5000 文字 | 改善提案の詳細 |
| `expected_benefit` | string | 任意 | 最大 2000 文字 | 期待効果 |
| `related_sop_id` | string (UUID v7) | 任意 | TBL-007 に存在 | 関連 SOP ID |
| `evidence_ids` | array of string (UUID v7) | 任意 | 各要素が TBL-009 に存在 | 添付エビデンス ID 一覧（最大 10 件）|

### 6-3. レスポンススキーマ（HTTP 201）

```json
{
  "data": {
    "proposal_id": "019682ab-7c1f-7000-a1b2-3c4d5e6f9999",
    "status": "submitted",
    "title": "ステップ 8 の確認待ち時間を短縮する",
    "proposer_id": "019682ab-7c1f-7000-0000-000000000002",
    "created_at": "2026-05-17T12:30:00.000Z"
  },
  "meta": {
    "request_id": "019682ab-7c1f-7043-a1b2-3c4d5e6f7890",
    "server_time": "2026-05-17T12:30:00.000Z",
    "api_version": "v1"
  }
}
```

TBL-015（kaizen_proposals）にレコードを挿入する。

### 6-4. RBAC

全ロール（operator を含む）が提案可能。

### 6-5. エラーコード

| ERR-CODE | HTTP | 発生条件 |
|---|---|---|
| ERR-AUTH-001 | 401 | JWT 無効 |
| ERR-VAL-001 | 422 | 必須フィールド不足 |
| ERR-VAL-002 | 422 | evidence_ids が 10 件超 |
| ERR-VAL-004 | 422 | proposal_detail が 5000 文字超 |

---

**本節で確定した方針**
- **API-andon-001 のアラート起票は全ロールに開放し、severity: critical の場合のみ MSG-005 を Outbox に挿入して supervisor / quality_admin 全員に通知することを確定した。**
- **CAPA（API-capa-001〜002）は quality_admin のみ作成・クローズ可とし、クローズ済み CAPA への更新は ERR-BIZ-008 で拒否することを確定した。**
- **改善提案（API-kaizen-001）は operator を含む全ロールが投稿可能とし、最大 10 件のエビデンス添付を許容することを確定した。**

---

## 参照業界分析

### 必須
- [`90_業界分析/09_セキュリティとアクセス制御.md`](../../../90_業界分析/09_セキュリティとアクセス制御.md)

### 関連
- [`90_業界分析/06_品質管理とトレーサビリティ.md`](../../../90_業界分析/06_品質管理とトレーサビリティ.md)
