# 07 作業指示 Pull 補完 API（API-sync-005）

本章は SSE 接続不可時の縮退動作および起動時初期取得に使用する作業割当一覧取得エンドポイント API-sync-005 を確定する。JWT による端末フィルタリング・UUID v7 ベースのカーソルページング・ポーリング動作の全仕様をコーディング直前精度で記述する。

> **担当バイナリ**: 本章のエンドポイントは **`wnav_terminal_api`（ポート 8080）** が担当する。端末からの作業割当取得は端末系バイナリの責務であり、`wnav_master_api` には存在しない。

---

## 1. API-sync-005: GET /api/v1/work-assignments

### 1-1. 概要

| 項目 | 値 |
|---|---|
| API-ID | API-sync-005 |
| HTTP メソッド | GET |
| URL | `/api/v1/work-assignments` |
| 担当バイナリ | terminal-api（ポート 8080）|
| 認証方式 | JWT Bearer（`aud: "terminal-api"`）|
| Idempotency-Key | 不要（GET）|
| レート制限カテゴリ | 同期（60 req / 60s）|
| 関連 FR | FR-SY-013（SSE フォールバック）|

SSE 接続が確立できない場合のフォールバックとして、端末が 30 秒間隔でポーリングして作業割当を取得するエンドポイント。アプリ起動時の初期取得にも使用する。JWT の `terminal_id` クレームに基づき自端末の割当のみを返す。

### 1-2. 端末がこのエンドポイントを使う場面

| 場面 | 詳細 |
|---|---|
| アプリ起動時の初期取得 | SSE 接続確立前にローカルキャッシュを更新するため、起動直後に一度呼び出す |
| SSE 接続不可時（縮退動作）| SSE 接続の確立・維持に失敗した場合、30 秒間隔でポーリングする。SSE 接続が回復した時点でポーリングを停止する |
| ネットワーク復帰時の差分同期 | オフライン復帰後に SSE 再接続と並行して呼び出し、接続不在期間の割当を補完する |

> SSE 接続が正常に維持されている場合は、本エンドポイントは呼び出さない。SSE がプライマリ配信経路であり、本エンドポイントはフォールバックである。

### 1-3. クエリパラメータ

| パラメータ | 型 | 必須 | デフォルト | 説明 |
|---|---|---|---|---|
| `status` | string | 任意 | `pending,dispatched` | カンマ区切りの状態フィルタ。指定可能値: `pending` / `dispatched` / `acknowledged` / `cancelled` |
| `limit` | integer | 任意 | 50 | 最大取得件数（1〜200）|
| `after` | string (UUID v7) | 任意 | — | カーソルページング用。指定した UUID v7 より新しい（`received_at` がより大きい）レコードを返す |

`status` のデフォルト値 `pending,dispatched` は、端末が処理すべき未完了の割当を取得する典型的なユースケースを想定している。

### 1-4. レスポンススキーマ（HTTP 200）

```json
{
  "data": [
    {
      "id": "019682ab-7c1f-7000-a1b2-3c4d5e6f7890",
      "sop_id": "019682ab-7c1f-7000-0000-000000000301",
      "sop_name": "溶接工程A手順",
      "lot_id": "019682ab-7c1f-7000-0000-000000000401",
      "lot_number": "LOT-2026-0042",
      "suggested_worker_id": null,
      "suggested_equipment_id": null,
      "due_at": "2026-05-18T15:00:00.000Z",
      "priority": 2,
      "status": "pending",
      "received_at": "2026-05-18T10:30:00.000Z"
    }
  ],
  "meta": {
    "request_id": "019682ab-7c1f-7001-a1b2-3c4d5e6f7890",
    "server_time": "2026-05-18T10:30:05.000Z",
    "api_version": "v1",
    "limit": 50,
    "has_more": false,
    "next_cursor": null
  }
}
```

#### data 配列の各フィールド

本エンドポイントは TBL-052 `work_assignments` の必要列のみを返す。機密性の高い内部管理フィールド（`external_order_id` / `external_system` 等）は端末に返さない。

| フィールド | 型 | 説明 |
|---|---|---|
| `id` | string (UUID v7) | 作業割当 ID（TBL-052）|
| `sop_id` | string (UUID v7) | SOP ID（TBL-007）|
| `sop_name` | string | SOP 名称（JOIN して取得）|
| `lot_id` | string (UUID v7) / null | ロット ID（TBL-024）|
| `lot_number` | string / null | ロット番号（人間可読）|
| `suggested_worker_id` | string (UUID v7) / null | 推奨作業者 ID（TBL-016）|
| `suggested_equipment_id` | string (UUID v7) / null | 推奨設備 ID（TBL-018）|
| `due_at` | string (ISO 8601 UTC) / null | 期限日時 |
| `priority` | integer | 優先度（1〜5）|
| `status` | string | `pending` / `dispatched` / `acknowledged` / `cancelled` |
| `received_at` | string (ISO 8601 UTC) | サーバー受信時刻（ソート・カーソル基準）|

#### meta フィールド

ページングは UUID v7 タイムスタンプに基づくカーソルページングを採用する（従来のページ番号方式は採用しない）。

| フィールド | 型 | 説明 |
|---|---|---|
| `meta.limit` | integer | リクエストで指定した limit 値 |
| `meta.has_more` | boolean | まだ取得できるレコードが存在する場合 `true` |
| `meta.next_cursor` | string (UUID v7) / null | 次ページ取得時に `after` パラメータに指定する値。`has_more = false` の場合 `null` |

次ページ取得例:

```http
GET /api/v1/work-assignments?status=pending,dispatched&limit=50&after=019682ab-7c1f-7000-a1b2-3c4d5e6f7890
Authorization: Bearer eyJhbGci...
```

### 1-5. DB クエリ

```sql
SELECT
  wa.id,
  wa.sop_id,
  s.name_json ->> 'ja' AS sop_name,
  wa.lot_id,
  l.lot_number,
  wa.suggested_worker_id,
  wa.suggested_equipment_id,
  wa.due_at,
  wa.priority,
  wa.status,
  wa.received_at
FROM work_assignments wa
LEFT JOIN sops s ON s.id = wa.sop_id
LEFT JOIN lots l ON l.id = wa.lot_id
WHERE wa.target_terminal_id = $terminal_id      -- JWT.terminal_id で強制フィルタリング
  AND wa.status = ANY($status_array)            -- status クエリパラメータ
  AND ($after IS NULL OR wa.received_at > (
        SELECT received_at FROM work_assignments WHERE id = $after
      ))
ORDER BY wa.received_at ASC
LIMIT $limit + 1                                -- has_more 判定のため limit + 1 件取得
```

`limit + 1` 件取得し、実際に返す件数が `limit + 1` の場合は `has_more = true` として最後の 1 件を除いて返す。

### 1-6. セキュリティ

JWT の `terminal_id` クレームで他端末の割当を絶対に返さない。

| セキュリティ要件 | 実装方法 |
|---|---|
| 自端末フィルタリング | `WHERE wa.target_terminal_id = JWT.terminal_id` を必須条件として常に付与する。クエリパラメータで `target_terminal_id` を上書きすることは禁止する |
| JWT の `aud` 検証 | `aud: "terminal-api"` を検証し、不一致の場合は HTTP 401 を返す |
| status による情報漏洩防止 | `cancelled` 状態の割当は端末の必要性が低いため、デフォルトの `status` フィルタに含めない |

### 1-7. RBAC

| ロール | アクセス |
|---|---|
| operator | 自端末（JWT.terminal_id）の割当のみ参照可 |
| supervisor | 自端末の割当を参照可（監視用途）|

### 1-8. エラーコード

| ERR-CODE | HTTP | 発生条件 |
|---|---|---|
| ERR-AUTH-001 | 401 | Authorization ヘッダ不足 / JWT 不正 / `aud` 不一致 |
| ERR-VAL-003 | 422 | JWT に `terminal_id` クレームが存在しない |
| ERR-VAL-001 | 422 | `limit` が 1〜200 の範囲外、または `after` が UUID v7 形式でない |

---

**本節で確定した方針**
- **API-sync-005 は terminal-api（ポート 8080）が担当し、JWT Bearer（`aud: "terminal-api"`）で認証することを確定した。**
- **端末は SSE 接続不可時に 30 秒間隔でポーリングする縮退動作に使用し、SSE 回復時はポーリングを停止することを確定した。アプリ起動時の初期取得にも使用する。**
- **クエリパラメータは status（デフォルト: pending,dispatched）/ limit（デフォルト: 50）/ after（UUID v7 カーソル）の 3 種とし、`next_cursor` を用いたカーソルページングを採用することを確定した。**
- **JWT の `terminal_id` クレームを WHERE 句に必須条件として埋め込み、他端末の割当を返さないことをクエリレベルで保証することを確定した。**
- **レスポンスは TBL-052 の必要列のみとし、外部システム管理フィールド（external_order_id 等）は端末に返さないことを確定した。**

---

## 参照業界分析

### 必須
- [`90_業界分析/09_セキュリティとアクセス制御.md`](../../../90_業界分析/09_セキュリティとアクセス制御.md)

### 関連
- [`90_業界分析/06_品質管理とトレーサビリティ.md`](../../../90_業界分析/06_品質管理とトレーサビリティ.md)
