# 02 REST API 設計方針（OpenAPI 3.1）

本章の責務は、全内部 REST API（API-NNN）に適用される設計方針を確定することである。URL 命名・バージョニング・エラーモデル・冪等性・レート制御・OpenAPI 仕様管理を定める。

---

## 1. URL 設計規約

### 1-1. URL 構造

```
/api/v{major}/{resource}[/{id}][/{sub-resource}]
```

| 要素 | 規約 | 例 |
|---|---|---|
| プレフィックス | `/api/v1/` 固定 | — |
| resource | 英小文字・ハイフン区切り・複数形 | `work-executions`, `step-events` |
| id | UUID v7（URL パスに含める）| `/work-executions/{id}` |
| アクション | POST + 動詞形サブパス（DELETE を使わない場合）| `/work-executions/{id}/suspend` |

### 1-2. HTTP メソッドと意味

| メソッド | 用途 | 冪等性 | 例 |
|---|---|---|---|
| GET | 参照（読み取り）| べき等 | `GET /work-orders` |
| POST | 作成 / 操作 | 非べき等（Idempotency Key で制御）| `POST /work-executions` |
| PATCH | 部分更新 | べき等（適用後の状態が同じ）| `PATCH /master-versions/{id}` |
| PUT | 完全置換 | べき等 | `PUT /device-config` |
| DELETE | **全エンドポイントで禁止** | — | — |

DELETE の代替: `POST /{id}/deactivate`（論理削除）または PATCH で `is_active: false`。

---

## 2. バージョニング規約

### 2-1. URL バージョニング

`/api/v{major}/` の major バージョンのみ URL に含める。

- **v1**: ver1.0.0 リリース時の最初のバージョン
- Breaking Change（後方非互換変更）が発生した場合のみ `v2` に移行
- 旧バージョンは最低 6 ヶ月間の deprecated 期間を設け、その後削除

### 2-2. バージョン判定基準

| Breaking Change（v2 移行が必要）| 非 Breaking（v1 のまま）|
|---|---|
| リクエスト・レスポンスのフィールド削除 | フィールド追加 |
| HTTP メソッドの変更 | エラーメッセージの変更 |
| URL 構造の変更 | パフォーマンス改善 |
| 認証方式の変更 | バグ修正 |

---

## 3. エラーレスポンス（RFC 9457 Problem Details）

### 3-1. レスポンス形式

```json
{
  "type": "https://errors.wnav.example.com/ERR-BIZ-001",
  "title": "lock_step_violation",
  "status": 409,
  "detail": "直前のステップが完了していません。ステップ番号 3 を完了してください。",
  "instance": "/requests/018d0e81-7e4e-7f00-b8b5-1234567890ab",
  "error_id": "ERR-BIZ-001"
}
```

Content-Type: `application/problem+json`（RFC 9457）

### 3-2. エラーコードと HTTP ステータスの対応

（ERR 全量は `02_ソフトウェア方式設計/07_例外・エラーハンドリング統一方式.md` 参照）

---

## 4. 冪等性設計（Idempotency-Key ヘッダ）

### 4-1. 必須エンドポイント

全 POST・PATCH エンドポイントに `Idempotency-Key: {UUID v7}` ヘッダを必須とする。

```http
POST /api/v1/work-executions/{id}/events
Content-Type: application/json
Idempotency-Key: 018d0e81-7e4e-7f00-b8b5-1234567890ab
Authorization: Bearer {JWT}

{
  "activity": "step_completed",
  "step_id": "...",
  "timestamp_client": "2026-05-17T10:30:00Z",
  ...
}
```

### 4-2. 重複リクエストの処理

1. TBL-035（idempotency_keys）で Idempotency-Key を検索
2. 存在する → 前回と同じレスポンスを返す（DB 再書き込みなし）
3. 存在しない → 通常処理を実行し、結果を TBL-035 に保存
4. 同じ Key で異なる Body → ERR-DB-001（idempotency_replay_conflict）

---

## 5. レート制御

### 5-1. トークンバケット方式

rate_limit_key = `{factory_id}:{endpoint_category}` の組み合わせ。

| エンドポイントカテゴリ | 上限 | 時間窓 |
|---|---|---|
| 読み取り（GET）| 1000 req | 60s |
| 書き込み（POST/PATCH）| 500 req | 60s |
| 認証（POST /auth/*）| 10 req | 60s（ブルートフォース防止）|
| Sync（GET/POST /sync/*）| 60 req | 60s |

上限超過時: ERR-SYS-002（429）+ `Retry-After: {seconds}` ヘッダ

---

## 6. OpenAPI 3.1 仕様管理

### 6-1. バイナリ分割と OpenAPI ファイル分割方針

バックエンドは以下の 2 バイナリに分割されている。

| バイナリ | ポート | 用途 | OpenAPI ファイル |
|---|---|---|---|
| `wnav_terminal_api` | 8080 | ハンディ端末向け API | `openapi-terminal.yaml` |
| `wnav_master_api` | 8081 | マスタメンテ・管理コンソール向け API | `openapi-master.yaml` |

各バイナリは `utoipa` crate（Apache 2.0）を使用して、自バイナリが担当するエンドポイントのみを含む OpenAPI 3.1 仕様を独立して自動生成する。単一の統合 OpenAPI ファイルは存在しない。

### 6-2. utoipa による自動生成

各バイナリで `utoipa` のパスマクロを定義し、バイナリのエントリポイントで統合する。

```rust
// wnav_terminal_api 側の例
#[utoipa::path(
    post,
    path = "/api/v1/work-executions/{id}/events",
    operation_id = "postWorkEvent",
    request_body = PostWorkEventRequest,
    responses(
        (status = 200, description = "イベント記録成功", body = WorkEventResponse),
        (status = 409, description = "冪等性競合", body = ProblemDetails),
        (status = 422, description = "バリデーションエラー", body = ProblemDetails),
    ),
    security(("bearer_auth" = [])),
    tag = "step-events",
)]
pub async fn post_work_event(...) { ... }
```

### 6-3. OpenAPI 仕様の配信

各バイナリは自バイナリの OpenAPI 3.1 仕様を `/openapi.json`（または `/api/v1/openapi.json`）で配信する。

| バイナリ | エンドポイント | 説明 |
|---|---|---|
| `wnav_terminal_api`（8080）| `GET /api/v1/openapi.json` | `openapi-terminal.yaml` 相当の JSON を返す |
| `wnav_master_api`（8081）| `GET /api/v1/openapi.json` | `openapi-master.yaml` 相当の JSON を返す |

### 6-4. クライアント側の型生成方針

フロントエンドは `openapi-typescript-codegen` を使用して TypeScript クライアントを自動生成する。生成元は担当バイナリの仕様に対応する。

| フロントエンド | 参照する OpenAPI | 説明 |
|---|---|---|
| FE-HA（ハンディ APP・React Native）| `openapi-terminal.yaml` | ハンディ端末向け API の型を生成 |
| FE-MA（マスタメンテ APP・React）| `openapi-master.yaml` | マスタ管理系 API の型を生成 |
| FE-MC（管理コンソール・React）| `openapi-master.yaml` | 管理コンソール系 API の型を生成 |

### 6-5. URL プレフィックスの共通維持

`/api/v1/` プレフィックスは `wnav_terminal_api`・`wnav_master_api` の両バイナリで共通して維持する。バイナリ分割はポート番号で区別し、URL 構造は変更しない。

---

**本節で確定した方針**
- **URL 命名（`/api/v1/{resource}/{id}/{sub-resource}` 形式・DELETE 禁止・POST + アクションサブパス代替）を全 API に適用する。**
- **Idempotency-Key ヘッダを全書き込み API に必須とし、TBL-035 で重複を検出して前回レスポンスを再返却する冪等性を確定した。**
- **OpenAPI 仕様を `openapi-terminal.yaml`（wnav_terminal_api）と `openapi-master.yaml`（wnav_master_api）の 2 ファイルに分割し、各バイナリが自バイナリの仕様を独立して生成・配信する構成を確定した。**
- **FE-HA は `openapi-terminal.yaml`、FE-MA / FE-MC は `openapi-master.yaml` を参照して TypeScript クライアントを自動生成する方針を確定した。**
- **`/api/v1/` プレフィックスは両バイナリ共通のまま維持することを確定した。**

---

## 参照業界分析

### 必須
- [`90_業界分析/27_オフライン同期とデータ整合性.md`](../../90_業界分析/27_オフライン同期とデータ整合性.md)

### 関連
- [`90_業界分析/22_規制別トレーサビリティ要件詳論.md`](../../90_業界分析/22_規制別トレーサビリティ要件詳論.md)
