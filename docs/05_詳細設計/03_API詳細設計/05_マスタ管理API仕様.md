# 05 マスタ管理 API 仕様

本章は API-master-001〜007（マスタバージョン管理：一覧・下書き作成・更新・提出・承認・ロールバック・ドライラン）を確定する。工程・SOP・ユーザーの補足仕様を含む。

> **担当バイナリ**: 本章の全エンドポイントは **`wnav_master_api`（ポート 8081）** が担当する。SOP CRUD・マスタ版管理・ユーザー管理はすべて管理系操作であり、`wnav_terminal_api` には存在しない。

---

## 1. マスタバージョン管理の状態モデル

SOP マスタは版管理（TBL-007 / TBL-004）によって以下の状態を遷移する。

```
Draft --> InReview --> Published
   |           |
   |           +-- Rejected --> Draft
   +-- (rollback) <-- Published
```

| 状態 | 説明 |
|---|---|
| `draft` | 編集中。master_admin のみ変更可 |
| `in_review` | レビュー待ち。quality_admin が承認 / 棄却 |
| `published` | 有効。作業実行が参照する |
| `deprecated` | 廃止。参照のみ。ロールバック元にはなれない |

---

## 2. API-master-001: GET /api/v1/master-versions

### 2-1. 概要

| 項目 | 値 |
|---|---|
| API-ID | API-master-001 |
| HTTP メソッド | GET |
| URL | `/api/v1/master-versions` |
| 担当バイナリ | master-api |
| 認証要否 | 必須 |
| Idempotency-Key | 不要（GET）|
| レート制限カテゴリ | 読み取り（1000 req / 60s）|
| 関連 FR | FR-MA-001 |

### 2-2. クエリパラメータ

| パラメータ | 型 | 必須 | 説明 |
|---|---|---|---|
| `sop_id` | string (UUID v7) | 任意 | SOP ID でフィルタ |
| `status` | string | 任意 | `draft` / `in_review` / `published` / `deprecated` |
| `process_id` | string (UUID v7) | 任意 | 工程 ID でフィルタ |
| `page` | integer | 任意 | デフォルト 1 |
| `per_page` | integer | 任意 | デフォルト 50 / 最大 200 |

### 2-3. レスポンススキーマ（HTTP 200）

```json
{
  "data": [
    {
      "id": "019682ab-7c1f-7000-0000-000000001001",
      "sop_id": "019682ab-7c1f-7000-0000-000000000301",
      "sop_name": "溶接工程A 標準手順書",
      "version": "3.2.0",
      "status": "published",
      "process_id": "019682ab-7c1f-7000-0000-000000000201",
      "process_name": "溶接工程A",
      "step_count": 12,
      "published_at": "2026-04-01T09:00:00.000Z",
      "published_by": "019682ab-7c1f-7000-0000-000000000050",
      "created_at": "2026-03-20T10:00:00.000Z",
      "updated_at": "2026-04-01T09:00:00.000Z"
    }
  ],
  "meta": {
    "request_id": "019682ab-7c1f-7030-a1b2-3c4d5e6f7890",
    "server_time": "2026-05-17T10:30:00.000Z",
    "api_version": "v1",
    "total": 8,
    "page": 1,
    "per_page": 50,
    "total_pages": 1
  }
}
```

### 2-4. 補足: GET /api/v1/master/processes（工程一覧）

採番台帳に個別エントリがないが、フロントエンドが必要とするため補足仕様として定義する。

```
GET /api/v1/master/processes?is_active=true&page=1&per_page=200
```

レスポンス `data` は TBL-021（processes）の配列。全ロールで参照可。

### 2-5. 補足: GET /api/v1/master/sops（SOP 一覧）

```
GET /api/v1/master/sops?process_id={uuid}&has_published_version=true
```

クエリパラメータ `has_published_version=true` で Published バージョンを持つ SOP のみに絞り込む。

---

## 3. API-master-002: POST /api/v1/master-versions/draft

### 3-1. 概要

| 項目 | 値 |
|---|---|
| API-ID | API-master-002 |
| HTTP メソッド | POST |
| URL | `/api/v1/master-versions/draft` |
| 担当バイナリ | master-api |
| 認証要否 | 必須 |
| Idempotency-Key | 必須 |
| レート制限カテゴリ | 書き込み（500 req / 60s）|
| 関連 FR | FR-MA-001 |

### 3-2. リクエストスキーマ

```json
{
  "sop_id": "019682ab-7c1f-7000-0000-000000000301",
  "base_version_id": "019682ab-7c1f-7000-0000-000000001001",
  "version_label": "3.3.0",
  "change_summary": "手順 5 の締め付けトルク値を更新",
  "steps": [
    {
      "step_number": 1,
      "title": "安全確認",
      "instruction": "作業開始前に安全装備を確認する。",
      "step_type": "check",
      "is_mandatory": true,
      "requires_evidence": false,
      "requires_sign": false,
      "skill_level_required": 1,
      "estimated_seconds": 60,
      "flow_rules": {
        "on_complete": "next",
        "on_skip": "supervisor_approval_required"
      }
    }
  ]
}
```

| フィールド | 型 | 必須 | 制約 | 説明 |
|---|---|---|---|---|
| `sop_id` | string (UUID v7) | 必須 | TBL-007 に存在 | 対象 SOP ID |
| `base_version_id` | string (UUID v7) | 任意 | TBL-004 に存在。省略時は最新 Published バージョンをベースにする | ベースとなるバージョン ID |
| `version_label` | string | 必須 | semver 形式（例: `3.3.0`）。既存バージョンより大きいこと | バージョン番号 |
| `change_summary` | string | 必須 | 1〜500 文字 | 変更概要 |
| `steps` | array | 必須 | 1〜500 件 | ステップ定義配列 |

#### steps 配列 各要素スキーマ

| フィールド | 型 | 必須 | 制約 | 説明 |
|---|---|---|---|---|
| `step_number` | integer | 必須 | 1 以上、配列内一意 | ステップ番号 |
| `title` | string | 必須 | 1〜100 文字 | ステップタイトル |
| `instruction` | string | 必須 | 1〜2000 文字 | 作業指示本文 |
| `step_type` | string | 必須 | `check` / `operation` / `measurement` / `sign` / `photo` | ステップ種別 |
| `is_mandatory` | boolean | 必須 | — | 必須ステップか否か |
| `requires_evidence` | boolean | 必須 | — | エビデンス添付必須か否か |
| `requires_sign` | boolean | 必須 | — | 電子サイン必須か否か |
| `skill_level_required` | integer | 必須 | 1〜5 | 必要スキルレベル（TBL-018 参照）|
| `estimated_seconds` | integer | 任意 | 0 以上 | 標準所要時間（秒）|
| `flow_rules` | object | 必須 | — | フロー制御ルール（TBL-030）|

### 3-3. レスポンススキーマ（HTTP 201）

```json
{
  "data": {
    "id": "019682ab-7c1f-7000-0000-000000001002",
    "sop_id": "019682ab-7c1f-7000-0000-000000000301",
    "version": "3.3.0",
    "status": "draft",
    "step_count": 12,
    "created_by": "019682ab-7c1f-7000-0000-000000000050",
    "created_at": "2026-05-17T10:00:00.000Z"
  },
  "meta": {
    "request_id": "019682ab-7c1f-7031-a1b2-3c4d5e6f7890",
    "server_time": "2026-05-17T10:00:00.000Z",
    "api_version": "v1"
  }
}
```

### 3-4. RBAC

`master_admin` のみ作成可。

### 3-5. エラーコード

| ERR-CODE | HTTP | 発生条件 |
|---|---|---|
| ERR-AUTH-004 | 403 | master_admin 以外 |
| ERR-BIZ-007 | 409 | version_label が既存バージョンと同じ |
| ERR-DB-002 | 409 | sop_id が TBL-007 に存在しない |
| ERR-VAL-001 | 422 | 必須フィールド不足 |
| ERR-VAL-002 | 422 | skill_level_required が 1〜5 範囲外 |
| ERR-VAL-004 | 422 | instruction が 2000 文字超 |

---

## 4. API-master-003: PATCH /api/v1/master-versions/{id}

### 4-1. 概要

| 項目 | 値 |
|---|---|
| API-ID | API-master-003 |
| HTTP メソッド | PATCH |
| URL | `/api/v1/master-versions/{id}` |
| 担当バイナリ | master-api |
| 認証要否 | 必須 |
| Idempotency-Key | 必須 |
| 関連 FR | FR-MA-002 |

### 4-2. リクエストスキーマ

```json
{
  "change_summary": "締め付けトルクを 25Nm から 28Nm に変更",
  "steps": [
    {
      "step_number": 5,
      "title": "締め付けトルク確認",
      "instruction": "トルクレンチで 28Nm になるまで締め付ける。",
      "step_type": "measurement",
      "is_mandatory": true,
      "requires_evidence": true,
      "requires_sign": true,
      "skill_level_required": 2,
      "estimated_seconds": 120,
      "flow_rules": {
        "on_complete": "next",
        "on_skip": "forbidden"
      }
    }
  ]
}
```

部分更新（PATCH）のため、変更するフィールドのみを送付する。`steps` を含む場合は全ステップを置換する。

`status` が `draft` 以外のバージョンへの PATCH は ERR-BIZ-005 で拒否する。

---

## 5. API-master-004: POST /api/v1/master-versions/{id}/submit

### 5-1. 概要

`draft` バージョンをレビューに提出する（`draft` → `in_review`）。

| 項目 | 値 |
|---|---|
| API-ID | API-master-004 |
| HTTP メソッド | POST |
| URL | `/api/v1/master-versions/{id}/submit` |
| 担当バイナリ | master-api |
| 認証要否 | 必須 |
| Idempotency-Key | 必須 |
| 関連 FR | FR-MA-005 |

### 5-2. リクエストスキーマ

```json
{
  "submitted_by": "019682ab-7c1f-7000-0000-000000000050",
  "review_comment": "品質部門へ承認依頼。手順 5 のトルク値変更のみ。"
}
```

| フィールド | 型 | 必須 | 説明 |
|---|---|---|---|
| `submitted_by` | string (UUID v7) | 必須 | 提出者 ID（TBL-016）|
| `review_comment` | string | 任意 | レビュー依頼コメント（最大 1000 文字）|

### 5-3. レスポンススキーマ（HTTP 200）

```json
{
  "data": {
    "id": "019682ab-7c1f-7000-0000-000000001002",
    "status": "in_review",
    "submitted_at": "2026-05-17T11:00:00.000Z",
    "submitted_by": "019682ab-7c1f-7000-0000-000000000050"
  },
  "meta": {
    "request_id": "019682ab-7c1f-7032-a1b2-3c4d5e6f7890",
    "server_time": "2026-05-17T11:00:00.000Z",
    "api_version": "v1"
  }
}
```

---

## 6. API-master-005: POST /api/v1/master-versions/{id}/approve

### 6-1. 概要

`in_review` バージョンを承認して `published` に移行する。

| 項目 | 値 |
|---|---|
| API-ID | API-master-005 |
| HTTP メソッド | POST |
| URL | `/api/v1/master-versions/{id}/approve` |
| 担当バイナリ | master-api |
| 認証要否 | 必須 |
| Idempotency-Key | 必須 |
| 関連 FR | FR-MA-005 |

### 6-2. リクエストスキーマ

```json
{
  "approved_by": "019682ab-7c1f-7000-0000-000000000060",
  "approval_comment": "内容確認済み。承認します。",
  "electronic_sign_id": "019682ab-7c1f-7000-a1b2-3c4d5e6f7890"
}
```

| フィールド | 型 | 必須 | 制約 | 説明 |
|---|---|---|---|---|
| `approved_by` | string (UUID v7) | 必須 | quality_admin ロール保有者 | 承認者 ID |
| `approval_comment` | string | 任意 | 最大 1000 文字 | 承認コメント |
| `electronic_sign_id` | string (UUID v7) | 必須 | TBL-002 に存在し context_type が `approval_sign` | 承認電子サイン ID |

### 6-3. レスポンススキーマ（HTTP 200）

```json
{
  "data": {
    "id": "019682ab-7c1f-7000-0000-000000001002",
    "status": "published",
    "published_at": "2026-05-17T14:00:00.000Z",
    "published_by": "019682ab-7c1f-7000-0000-000000000060"
  },
  "meta": {
    "request_id": "019682ab-7c1f-7033-a1b2-3c4d5e6f7890",
    "server_time": "2026-05-17T14:00:00.000Z",
    "api_version": "v1"
  }
}
```

承認時、前の `published` バージョンを `deprecated` に変更し、MSG-004（`internal.master_published`）を発行する。

### 6-4. RBAC

`quality_admin` のみ実行可。

### 6-5. エラーコード

| ERR-CODE | HTTP | 発生条件 |
|---|---|---|
| ERR-AUTH-004 | 403 | quality_admin 以外 |
| ERR-BIZ-003 | 409 | status が `in_review` でない |
| ERR-VAL-001 | 422 | electronic_sign_id 不足 |

---

## 7. API-master-006: POST /api/v1/master-versions/{id}/rollback

### 7-1. 概要

現在の `published` バージョンを `deprecated` にし、指定バージョンを `published` に戻す。

| 項目 | 値 |
|---|---|
| API-ID | API-master-006 |
| HTTP メソッド | POST |
| URL | `/api/v1/master-versions/{id}/rollback` |
| 担当バイナリ | master-api |
| 認証要否 | 必須 |
| Idempotency-Key | 必須 |
| 関連 FR | FR-MA-006 |

### 7-2. リクエストスキーマ

```json
{
  "rollback_by": "019682ab-7c1f-7000-0000-000000000060",
  "reason": "手順 5 のトルク値に誤りが発見されたため前バージョンに戻す",
  "electronic_sign_id": "019682ab-7c1f-7000-a1b2-3c4d5e6f7891"
}
```

`{id}` は戻す先（ロールバック先）の旧バージョン ID（`deprecated` または `published` 状態のもの）。

### 7-3. エラーコード

| ERR-CODE | HTTP | 発生条件 |
|---|---|---|
| ERR-AUTH-004 | 403 | quality_admin 以外 |
| ERR-BIZ-005 | 409 | ロールバック先バージョンが `in_review` / `draft` 状態 |
| ERR-VAL-001 | 422 | 必須フィールド不足 |

---

## 8. API-master-007: POST /api/v1/master-versions/{id}/dry-run

### 8-1. 概要

バージョンを廃止または置換する前に、影響を受ける作業実行（TBL-005）を事前確認する。実際の状態変更は行わない。

| 項目 | 値 |
|---|---|
| API-ID | API-master-007 |
| HTTP メソッド | POST |
| URL | `/api/v1/master-versions/{id}/dry-run` |
| 担当バイナリ | master-api |
| 認証要否 | 必須 |
| Idempotency-Key | 必須 |
| 関連 FR | FR-MA-008 |

### 8-2. リクエストスキーマ

```json
{
  "action": "deprecate"
}
```

| フィールド | 型 | 必須 | 制約 | 説明 |
|---|---|---|---|---|
| `action` | string | 必須 | `deprecate` / `publish` | シミュレートするアクション |

### 8-3. レスポンススキーマ（HTTP 200）

```json
{
  "data": {
    "action": "deprecate",
    "target_version_id": "019682ab-7c1f-7000-0000-000000001001",
    "affected_work_executions": [
      {
        "work_execution_id": "019682ab-7c1f-7000-b1c2-3d4e5f6a7b8c",
        "work_order_id": "019682ab-7c1f-7000-0000-000000000101",
        "operator_id": "019682ab-7c1f-7000-0000-000000000002",
        "status": "in_progress"
      }
    ],
    "affected_count": 1,
    "is_safe_to_proceed": false,
    "warnings": [
      "1 件の in_progress 作業実行がこのバージョンを参照しています。完了または完了後に廃止してください。"
    ]
  },
  "meta": {
    "request_id": "019682ab-7c1f-7034-a1b2-3c4d5e6f7890",
    "server_time": "2026-05-17T10:30:00.000Z",
    "api_version": "v1"
  }
}
```

| フィールド | 型 | 説明 |
|---|---|---|
| `affected_count` | integer | 影響を受ける作業実行の件数 |
| `is_safe_to_proceed` | boolean | `true` = 影響なし・即時実行可、`false` = 事前対応が必要 |
| `warnings` | array of string | 警告メッセージ一覧 |

---

## 9. マスタユーザー管理 補足仕様

採番台帳に個別エントリがない以下のエンドポイントを補足仕様として定義する。

### 9-1. GET /api/v1/master/users（ユーザー一覧）

```
GET /api/v1/master/users?is_active=true&role=operator&page=1&per_page=50
```

レスポンス `data` は TBL-016 ユーザーオブジェクトの配列（パスワードハッシュ・PIN ハッシュを除く）。`system_admin` のみアクセス可。

### 9-2. POST /api/v1/master/users（ユーザー作成）

```json
{
  "login_id": "operator05",
  "display_name": "鈴木 花子",
  "email": "hanako.suzuki@factory.local",
  "password_initial": "TempP@ss0001",
  "factory_id": "019682ab-7c1f-7000-0000-000000000001",
  "roles": ["operator"],
  "skills": [
    { "skill_id": "019682ab-7c1f-7000-0000-000000000901", "level": 2 }
  ]
}
```

`system_admin` のみ作成可。レスポンス HTTP 201。

### 9-3. PUT /api/v1/master/users/{id}/roles（ロール割当）

```json
{
  "roles": ["operator", "supervisor"]
}
```

`system_admin` のみ実行可。レスポンス HTTP 200。フィールド `roles` の内容で完全置換する。

---

**本節で確定した方針**
- **本章の全エンドポイント（API-master-001〜007、補足仕様のユーザー管理含む）は `wnav_master_api`（ポート 8081）が担当し、`wnav_terminal_api` には存在しないことを確定した。SOP CRUD・マスタ版管理・ユーザー管理はすべて管理系操作であるため master-api に集約する。**
- **マスタバージョンは Draft → InReview → Published の状態遷移を持ち、Published 移行時に前バージョンを Deprecated にして MSG-004 を発行することを確定した。**
- **API-master-007 ドライランは廃止前の影響確認専用エンドポイントであり、実際の状態変更を行わず `is_safe_to_proceed` フラグで結果を返すことを確定した。**
- **ロールバック（API-master-006）と承認（API-master-005）は quality_admin 専用とし、両操作に電子サイン ID（TBL-002）を必須とすることを確定した。**

---

## 参照業界分析

### 必須
- [`90_業界分析/09_セキュリティとアクセス制御.md`](../../../90_業界分析/09_セキュリティとアクセス制御.md)

### 関連
- [`90_業界分析/06_品質管理とトレーサビリティ.md`](../../../90_業界分析/06_品質管理とトレーサビリティ.md)
