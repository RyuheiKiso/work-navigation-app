# 07 運用・監視 API 仕様

本章は API-ops-001〜002（DLQ 管理）・API-reports-001〜002（帳票・監査ログ）・API-system-001〜003（ヘルスチェック・OpenAPI 配信）・API-sync-001〜002・API-trace-001〜002 を確定する。

---

## 1. API-ops-001: GET /api/v1/ops/outbox/dlq

### 1-1. 概要

Outbox の Dead Letter Queue（DLQ）に入ったイベントの一覧と件数を返す。

| 項目 | 値 |
|---|---|
| API-ID | API-ops-001 |
| HTTP メソッド | GET |
| URL | `/api/v1/ops/outbox/dlq` |
| 認証要否 | 必須 |
| Idempotency-Key | 不要（GET）|
| レート制限カテゴリ | 読み取り（1000 req / 60s）|
| 関連 FR | FR-SY-008 |

### 1-2. クエリパラメータ

| パラメータ | 型 | 必須 | 説明 |
|---|---|---|---|
| `event_type` | string | 任意 | `work_event` / `electronic_sign` / `webhook_audit_event` |
| `failed_after` | string (ISO 8601) | 任意 | 指定時刻以降に失敗したイベントのみ |
| `page` | integer | 任意 | デフォルト 1 |
| `per_page` | integer | 任意 | デフォルト 50 / 最大 200 |

### 1-3. レスポンススキーマ（HTTP 200）

```json
{
  "data": {
    "dlq_count": 3,
    "items": [
      {
        "dlq_item_id": "019682ab-7c1f-7000-0000-000000002001",
        "outbox_event_id": "019682ab-7c1f-7000-0000-000000003001",
        "event_type": "work_event",
        "payload_summary": {
          "work_execution_id": "019682ab-7c1f-7000-b1c2-3d4e5f6a7b8c",
          "activity": "step_completed"
        },
        "retry_count": 5,
        "last_error": "Connection refused: wnav-parent.factory.local:443",
        "first_failed_at": "2026-05-17T06:00:00.000Z",
        "last_failed_at": "2026-05-17T09:00:00.000Z"
      }
    ]
  },
  "meta": {
    "request_id": "019682ab-7c1f-7050-a1b2-3c4d5e6f7890",
    "server_time": "2026-05-17T10:30:00.000Z",
    "api_version": "v1",
    "total": 3,
    "page": 1,
    "per_page": 50,
    "total_pages": 1
  }
}
```

| フィールド | 型 | 説明 |
|---|---|---|
| `dlq_count` | integer | DLQ 全件数（ページングに依存しない総数）|
| `items[].dlq_item_id` | string (UUID v7) | DLQ アイテム ID |
| `items[].outbox_event_id` | string (UUID v7) | 元の Outbox イベント ID（TBL-003）|
| `items[].retry_count` | integer | 累計リトライ回数 |
| `items[].last_error` | string | 最後のエラーメッセージ |
| `items[].first_failed_at` | string (ISO 8601 UTC) | 最初の失敗時刻 |

`dlq_count > 0` の場合、MET-005 カウンタが非ゼロとなりアラートが発火する。

### 1-4. RBAC

`system_admin` のみアクセス可。

---

## 2. API-ops-002: POST /api/v1/ops/outbox/{id}/requeue

### 2-1. 概要

| 項目 | 値 |
|---|---|
| API-ID | API-ops-002 |
| HTTP メソッド | POST |
| URL | `/api/v1/ops/outbox/{id}/requeue` |
| 認証要否 | 必須 |
| Idempotency-Key | 必須 |
| 関連 FR | FR-SY-008 |

`{id}` は TBL-003 の Outbox イベント ID。

### 2-2. リクエストスキーマ

```json
{
  "requeued_by": "019682ab-7c1f-7000-0000-000000000099",
  "reason": "親機への接続が回復したため手動再送を実施する"
}
```

| フィールド | 型 | 必須 | 説明 |
|---|---|---|---|
| `requeued_by` | string (UUID v7) | 必須 | 操作者 ID（system_admin）|
| `reason` | string | 必須 | 再送理由（1〜500 文字）|

### 2-3. レスポンススキーマ（HTTP 200）

```json
{
  "data": {
    "outbox_event_id": "019682ab-7c1f-7000-0000-000000003001",
    "status": "pending",
    "retry_count_reset": true,
    "requeued_at": "2026-05-17T10:35:00.000Z"
  },
  "meta": {
    "request_id": "019682ab-7c1f-7051-a1b2-3c4d5e6f7890",
    "server_time": "2026-05-17T10:35:00.000Z",
    "api_version": "v1"
  }
}
```

再送時、`retry_count` をゼロにリセットし `status: pending` に戻す。LOG-008（outbox.dlq.moved）を記録する。

### 2-4. エラーコード

| ERR-CODE | HTTP | 発生条件 |
|---|---|---|
| ERR-AUTH-004 | 403 | system_admin 以外 |
| ERR-SYS-005 | 503 | DLQ が上限に達している（DLQ overflow）|
| `404 Not Found` | 404 | id が存在しない |

---

## 3. API-reports-001: POST /api/v1/reports/sop-execution

### 3-1. 概要

RP-001「SOP 実施率・工数実績レポート」のデータを生成して返す。

| 項目 | 値 |
|---|---|
| API-ID | API-reports-001 |
| HTTP メソッド | POST |
| URL | `/api/v1/reports/sop-execution` |
| 認証要否 | 必須 |
| Idempotency-Key | 必須 |
| 関連 FR（レポート）| RP-001 |

### 3-2. リクエストスキーマ

```json
{
  "process_ids": ["019682ab-7c1f-7000-0000-000000000201"],
  "date_from": "2026-05-01",
  "date_to": "2026-05-17",
  "sop_id": "019682ab-7c1f-7000-0000-000000000301",
  "requested_by": "019682ab-7c1f-7000-0000-000000000060",
  "format": "json"
}
```

| フィールド | 型 | 必須 | 制約 | 説明 |
|---|---|---|---|---|
| `process_ids` | array of string (UUID v7) | 任意 | 最大 50 件 | 対象工程 ID 一覧。省略時は全工程 |
| `date_from` | string (ISO 8601 date) | 必須 | — | 集計開始日 |
| `date_to` | string (ISO 8601 date) | 必須 | `date_from` 以降 | 集計終了日 |
| `sop_id` | string (UUID v7) | 任意 | TBL-007 に存在 | SOP でフィルタ |
| `requested_by` | string (UUID v7) | 必須 | TBL-016 に存在 | 依頼者 ID |
| `format` | string | 任意 | `json`（デフォルト）/ `csv` | 出力形式 |

### 3-3. レスポンススキーマ（HTTP 200）

```json
{
  "data": {
    "report_id": "019682ab-7c1f-7000-0000-000000004001",
    "generated_at": "2026-05-17T10:30:05.000Z",
    "period": { "from": "2026-05-01", "to": "2026-05-17" },
    "summary": {
      "total_executions": 142,
      "completed": 138,
      "suspended": 3,
      "in_progress": 1,
      "completion_rate": 97.2
    },
    "by_process": [
      {
        "process_id": "019682ab-7c1f-7000-0000-000000000201",
        "process_name": "溶接工程A",
        "executions": 45,
        "avg_duration_minutes": 185.3,
        "step_skip_count": 2
      }
    ],
    "document_hash": "sha256:report_hash_abc123..."
  },
  "meta": {
    "request_id": "019682ab-7c1f-7052-a1b2-3c4d5e6f7890",
    "server_time": "2026-05-17T10:30:05.000Z",
    "api_version": "v1"
  }
}
```

帳票生成後、BAT-007（document_hash_recorder）が `document_hash` を TBL-031 に記録する。LOG-010 を出力する。

---

## 4. API-reports-002: POST /api/v1/reports/audit-xes

### 4-1. 概要

RP-002「監査ログ XES エクスポート」。プロセスマイニング用 IEEE XES 形式でイベントログをエクスポートする。

| 項目 | 値 |
|---|---|
| API-ID | API-reports-002 |
| HTTP メソッド | POST |
| URL | `/api/v1/reports/audit-xes` |
| 認証要否 | 必須 |
| Idempotency-Key | 必須 |
| 関連 FR（レポート）| RP-002 |

### 4-2. リクエストスキーマ

```json
{
  "date_from": "2026-04-01",
  "date_to": "2026-05-17",
  "process_ids": ["019682ab-7c1f-7000-0000-000000000201"],
  "include_evidence_hashes": true,
  "requested_by": "019682ab-7c1f-7000-0000-000000000060"
}
```

### 4-3. レスポンス

- `format: "xes"` の場合: Content-Type `application/octet-stream` で XES ファイルをストリーム返却。
- `format: "json"` の場合: JSON エンベロープで XES 構造を返却。

`quality_admin` / `system_admin` のみアクセス可。MET-009 で配信成功率を監視する。

### 4-4. エラーコード

| ERR-CODE | HTTP | 発生条件 |
|---|---|---|
| ERR-AUTH-004 | 403 | quality_admin / system_admin 以外 |
| ERR-SYS-003 | 500 | 帳票生成失敗（BAT リトライ対象）|

---

## 5. API-sync-001: GET /api/v1/sync/master

### 5-1. 概要

子機が親機からマスタデータを同期取得する。

| 項目 | 値 |
|---|---|
| API-ID | API-sync-001 |
| HTTP メソッド | GET |
| URL | `/api/v1/sync/master` |
| 認証要否 | 必須（OAuth 2.1 Client Credentials または mTLS）|
| Idempotency-Key | 不要（GET）|
| レート制限カテゴリ | 同期（60 req / 60s）|
| 関連 FR | FR-SY-001 |

### 5-2. クエリパラメータ

| パラメータ | 型 | 必須 | 説明 |
|---|---|---|---|
| `since` | string (ISO 8601) | 任意 | 指定時刻以降に更新されたデータのみ返す（差分同期）|
| `resource_types` | string | 任意 | カンマ区切り: `sops,processes,users,equipments,instruments` |

### 5-3. レスポンス（HTTP 200）

```json
{
  "data": {
    "sync_timestamp": "2026-05-17T10:30:00.000Z",
    "sops": [...],
    "processes": [...],
    "users": [...],
    "has_more": false
  },
  "meta": {
    "request_id": "019682ab-7c1f-7053-a1b2-3c4d5e6f7890",
    "server_time": "2026-05-17T10:30:00.000Z",
    "api_version": "v1"
  }
}
```

---

## 6. API-sync-002: POST /api/v1/sync/outbox/inbound

### 6-1. 概要

子機バックエンドから親機へ Outbox イベントを送信する（IF-002）。

| 項目 | 値 |
|---|---|
| API-ID | API-sync-002 |
| HTTP メソッド | POST |
| URL | `/api/v1/sync/outbox/inbound` |
| 認証要否 | 必須（OAuth 2.1 Client Credentials / scope: wnav.outbox.write）|
| Idempotency-Key | 必須 |
| 関連 FR | FR-SY-002 |

### 6-2. リクエストスキーマ

```json
{
  "source_factory_id": "019682ab-7c1f-7000-0000-000000000001",
  "events": [
    {
      "outbox_event_id": "019682ab-7c1f-7000-0000-000000003001",
      "event_type": "work_event",
      "payload": { "...": "..." },
      "occurred_at": "2026-05-17T09:00:00.000Z"
    }
  ]
}
```

配列一括送信（最大 100 件 / リクエスト）。各 `outbox_event_id` で Idempotency チェックを行う。

---

## 7. API-trace-001: GET /api/v1/trace/forward

### 7-1. 概要

指定した入力（ロット / 部品 / 素材）から製品方向への前向きトレース。

| 項目 | 値 |
|---|---|
| API-ID | API-trace-001 |
| HTTP メソッド | GET |
| URL | `/api/v1/trace/forward` |
| 認証要否 | 必須 |
| 関連 FR | FR-KZ-007 |

クエリパラメータ: `lot_id` または `product_id` を必須。深さ `max_depth`（デフォルト 5 / 最大 20）。

---

## 8. API-trace-002: GET /api/v1/trace/backward

後向きトレース（製品 → 素材 / 部品 / ロット）。API-trace-001 と同構造で逆方向。

---

## 9. API-system-001: GET /api/v1/healthz

### 9-1. 概要

Kubernetes / IIS の liveness probe に使用する。認証不要。

| 項目 | 値 |
|---|---|
| API-ID | API-system-001 |
| HTTP メソッド | GET |
| URL | `/api/v1/healthz` |
| 認証要否 | 不要 |
| レート制限 | 適用外 |
| 関連 NFR | NFR-AVL-001 |

### 9-2. レスポンス（HTTP 200）

```json
{
  "status": "ok",
  "timestamp": "2026-05-17T10:30:00.000Z"
}
```

バックエンドプロセスが起動中であれば常に HTTP 200 / `status: "ok"` を返す。DB 接続状態には依存しない。

---

## 10. API-system-002: GET /api/v1/readyz

### 10-1. 概要

readiness probe。DB・外部依存が利用可能かを確認する。

| 項目 | 値 |
|---|---|
| API-ID | API-system-002 |
| HTTP メソッド | GET |
| URL | `/api/v1/readyz` |
| 認証要否 | 不要 |
| 関連 NFR | NFR-AVL-001 |

### 10-2. レスポンス（HTTP 200）

```json
{
  "status": "ready",
  "checks": {
    "database": "ok",
    "outbox_consumer": "ok",
    "ldap": "degraded"
  },
  "timestamp": "2026-05-17T10:30:00.000Z"
}
```

| フィールド | 値 | 説明 |
|---|---|---|
| `checks.database` | `ok` / `error` | PostgreSQL 接続確認 |
| `checks.outbox_consumer` | `ok` / `error` | Outbox Consumer 稼働確認 |
| `checks.ldap` | `ok` / `degraded` / `error` | LDAP 接続確認（degraded はローカル認証フォールバック中）|

DB が `error` の場合、HTTP 503 を返す（ロードバランサーがトラフィックを切り離す）。

---

## 11. API-system-003: GET /api/v1/openapi.json

### 11-1. 概要

utoipa が生成した OpenAPI 3.1 仕様 JSON を返す。

| 項目 | 値 |
|---|---|
| API-ID | API-system-003 |
| HTTP メソッド | GET |
| URL | `/api/v1/openapi.json` |
| 認証要否 | 不要（開発環境）/ 必須（本番環境は system_admin のみ）|

### 11-2. レスポンス（HTTP 200）

Content-Type: `application/json`。OpenAPI 3.1 準拠の JSON オブジェクトを返す。

```json
{
  "openapi": "3.1.0",
  "info": {
    "title": "WNAV API",
    "version": "1.0.0",
    "description": "作業ナビゲーションシステム REST API"
  },
  "servers": [
    { "url": "https://wnav-server.factory.local/api/v1" }
  ],
  "paths": { "...": "..." },
  "components": { "...": "..." }
}
```

---

> **運用手順参照先**: DLQ 復旧の運用手順は `docs/09_運用・保守/障害対応/09_Outbox_DLQ復旧手順.md` に確定した。

**本節で確定した方針**
- **DLQ 管理（API-ops-001〜002）は system_admin 専用とし、requeue 時に retry_count をリセットして LOG-008 を出力することを確定した。**
- **ヘルスチェック（API-system-001）はプロセス起動確認のみで DB 接続状態に依存しない liveness probe として確定し、readyz（API-system-002）は DB / Outbox / LDAP の状態を checks オブジェクトで返す readiness probe として確定した。**
- **API-system-003 は utoipa 自動生成 OpenAPI 3.1 仕様を返し、本番環境では system_admin のみアクセス可とすることを確定した。**

---

## 参照業界分析

### 必須
- [`90_業界分析/09_セキュリティとアクセス制御.md`](../../../90_業界分析/09_セキュリティとアクセス制御.md)

### 関連
- [`90_業界分析/06_品質管理とトレーサビリティ.md`](../../../90_業界分析/06_品質管理とトレーサビリティ.md)
