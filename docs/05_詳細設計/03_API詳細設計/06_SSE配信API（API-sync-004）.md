# 06 SSE 配信 API（API-sync-004）

本章は端末への Server-Sent Events（SSE）による作業割当リアルタイム配信エンドポイント API-sync-004 と、端末からの確認応答エンドポイント（ACK）を確定する。接続時初期配信・Last-Event-ID 再接続・keep-alive・IIS リバースプロキシ通過・sse_dispatch_log 更新の全挙動をコーディング直前精度で記述する。

> **担当バイナリ**: 本章のエンドポイントは **`wnav_terminal_api`（ポート 8080）** が担当する。現場端末との SSE 永続接続は端末系バイナリの責務であり、`wnav_master_api` には存在しない。

---

## 1. API-sync-004: GET /api/v1/sse/assignments

### 1-1. 概要

| 項目 | 値 |
|---|---|
| API-ID | API-sync-004 |
| HTTP メソッド | GET |
| URL | `/api/v1/sse/assignments` |
| 担当バイナリ | terminal-api（ポート 8080）|
| 認証方式 | JWT Bearer（`aud: "terminal-api"`、`terminal_id` クレーム必須）|
| Idempotency-Key | 不要（GET）|
| レート制限カテゴリ | 同期（60 req / 60s）|
| 関連 FR | FR-SY-013 |

端末が長時間接続を確立し、作業割当イベントをリアルタイムに受信するエンドポイント。接続確立後、まず未配信の割当を初期配信し、以降は SSE ストリームでイベントを Push する。接続が切れた場合は `Last-Event-ID` ヘッダを用いて再接続・再開できる。

### 1-2. リクエストヘッダ

| ヘッダ名 | 必須 | 説明 |
|---|---|---|
| `Authorization` | 必須 | `Bearer <JWT>`。JWT の `aud` が `"terminal-api"`、`terminal_id` クレームが存在することを検証する |
| `Last-Event-ID` | 任意 | 再接続時に最後に受信したイベント ID（UUID v7）を付与する。サーバーはこれ以降のイベントを送信する |
| `Accept` | 任意 | `text/event-stream` を指定することが推奨（省略時も SSE として扱う）|

### 1-3. レスポンスヘッダ

| ヘッダ名 | 値 | 説明 |
|---|---|---|
| `Content-Type` | `text/event-stream; charset=utf-8` | SSE ストリームであることを明示する |
| `Cache-Control` | `no-cache` | SSE の標準要件 |
| `Connection` | `keep-alive` | 永続接続を維持する |
| `X-Accel-Buffering` | `no` | nginx / IIS リバースプロキシのバッファリングを無効化する（§ 1-8 参照）|
| `Transfer-Encoding` | `chunked` | チャンク転送で逐次フラッシュを可能にする |

### 1-4. SSE イベントフォーマット詳細

SSE ストリームは RFC 8895 の text/event-stream フォーマットに従う。各イベントは `id` / `event` / `data` の 3 フィールドで構成し、末尾を空行 2 つで区切る。

```
id: 019682ab-7c1f-7000-a1b2-3c4d5e6f7890
event: assignment.created
data: {"assignment_id":"019682ab-7c1f-7000-a1b2-3c4d5e6f7890","sop_id":"019682ab-7c1f-7000-0000-000000000301","sop_name":"溶接工程A手順","lot_id":"019682ab-7c1f-7000-0000-000000000401","lot_number":"LOT-2026-0042","due_at":"2026-05-18T15:00:00.000Z","priority":2,"suggested_worker_id":null,"suggested_equipment_id":null,"received_at":"2026-05-18T10:30:00.000Z"}

```

| フィールド | 型 | 説明 |
|---|---|---|
| `id` | UUID v7 文字列 | イベント ID。UUID v7 はタイムスタンプを内包するため、Last-Event-ID 再接続時の時刻比較に直接使用できる |
| `event` | string | イベントタイプ（下記 4 種）|
| `data` | JSON 文字列（1 行）| イベント固有のペイロード（整形なし・改行なし）|

#### 4 イベントタイプ

| event 値 | 発生タイミング | data の主要フィールド |
|---|---|---|
| `assignment.created` | 新規作業割当が到着したとき（API-sync-003 経由）| assignment_id / sop_id / sop_name / lot_id / due_at / priority |
| `assignment.cancelled` | 外部システムまたは管理者が割当をキャンセルしたとき | assignment_id / cancelled_at / reason |
| `assignment.updated` | 割当の due_at / priority 等が変更されたとき | assignment_id / changed_fields (object) / updated_at |
| `keepalive` | CFG-029 = 25 秒ごとの keep-alive ping | timestamp（ISO 8601 UTC）|

`keepalive` イベントの `data` フィールドは以下の形式とする。

```
id: 019682ab-7c1f-7001-a1b2-3c4d5e6f7890
event: keepalive
data: {"timestamp":"2026-05-18T10:30:25.000Z"}

```

### 1-5. Last-Event-ID 再接続フロー

端末が SSE 接続を失った場合、`Last-Event-ID` ヘッダに最後に受信した UUID v7 を付与して再接続する。サーバーは以下の手順で再開位置を特定する。

1. `Last-Event-ID` ヘッダの値を UUID v7 として検証する。形式不正の場合は無視して初期配信フローを実行する。
2. UUID v7 からタイムスタンプ部（上位 48 bit）を抽出し、`last_event_time` として取得する。
3. TBL-053 `sse_dispatch_log` から `target_terminal_id = JWT.terminal_id AND dispatched_at > last_event_time AND status = 'dispatched'` のレコードを `dispatched_at ASC` で取得する。
4. 取得したレコードを順に SSE イベントとして送信する。その後、通常のリアルタイム配信モードに移行する。
5. `Last-Event-ID` に対応するレコードが TBL-053 に存在しない（TTL 超過等）場合は、初期配信フロー（§ 1-6）を実行する。

### 1-6. 接続時初期配信ロジック

`Last-Event-ID` が存在しない、または再接続で対応レコードが見つからない場合、以下のクエリで未受信の割当を初期配信する。

```sql
SELECT *
FROM work_assignments
WHERE target_terminal_id = $terminal_id
  AND status IN ('pending', 'dispatched')
ORDER BY received_at ASC
```

- テーブル: TBL-052 `work_assignments`
- フィルタ: `target_terminal_id = JWT.terminal_id`、`status IN ('pending', 'dispatched')`
- ソート: `received_at ASC`（古いものから順に送信）

取得した各レコードを `assignment.created` イベントとして送信する。初期配信完了後、通常のリアルタイム配信モードに移行する。

### 1-7. keep-alive ping 仕様

CFG-029 `sse.keep_alive_sec`（デフォルト 25 秒）間隔で `keepalive` イベントを送信する。

- 送信間隔: CFG-029 で変更可能（デフォルト 25 秒）
- 目的: IIS・nginx のアイドル接続タイムアウト（通常 60〜120 秒）によって接続が切断されることを防ぐ
- `keepalive` イベントは TBL-053 には記録しない（配信ログの肥大化防止）

### 1-8. sse_dispatch_log（TBL-053）の更新タイミング

| 操作 | タイミング |
|---|---|
| `status = 'queued'` のレコード挿入 | API-sync-003 の INSERT 成功後（§ 1-8 in API-sync-003）|
| `status = 'dispatched'`、`dispatched_at = NOW()` に更新 | terminal-api が端末の SSE ストリームへのフラッシュが成功したとき |
| `status = 'acknowledged'`、`acknowledged_at = NOW()` に更新 | ACK エンドポイント（§ 2）が呼び出されたとき |
| `status = 'failed'`、`failed_at = NOW()` に更新 | dispatch_retry_max（CFG-030）回試行後も配信できなかったとき |

配信リトライは CFG-030 `sse.dispatch_retry_max`（デフォルト 5 回）まで行う。超過した場合は `status = 'failed'` として記録し、アラートを発する。

### 1-9. IIS リバースプロキシ通過に関する注意事項

IIS（Windows Server 2022）が terminal-api のリバースプロキシとなる構成では、以下の追加設定が必要である。

- **ARR（Application Request Routing）のレスポンスバッファリング無効化**: ARR はデフォルトでレスポンスをバッファリングするため、SSE イベントが端末に到達しない。`disableCache="true"` および `responseBufferingEnabled="false"` を ARR のルール設定で明示する。
- **`X-Accel-Buffering: no` ヘッダ**: nginx 互換の設定値であり、IIS ARR でも有効に機能する場合があるため付与する。
- **アイドルタイムアウト**: IIS のアイドルタイムアウトは CFG-029（25 秒）の keep-alive ping によって維持する。IIS 側のタイムアウト値は 60 秒以上に設定し、keep-alive ping の送信間隔を下回ることを確認する。
- **HTTP/1.1 必須**: SSE は HTTP/1.1 の chunked transfer encoding に依存する。IIS の HTTP/2 設定が SSE 接続と競合する場合は、`/api/v1/sse/*` のパスに対して HTTP/1.1 を強制する。

### 1-10. RBAC

JWT の `terminal_id` クレームで自端末の割当のみが返される。他端末の割当は返さない。JWT の `aud` が `"terminal-api"` であることを検証し、不一致の場合は HTTP 401 を返す。

| ロール | アクセス |
|---|---|
| operator | 自端末（JWT.terminal_id）の割当のみ受信可 |
| supervisor | 参照可（監視用途）|

### 1-11. エラーコード

| ERR-CODE | HTTP | 発生条件 |
|---|---|---|
| ERR-AUTH-001 | 401 | Authorization ヘッダ不足 / JWT 不正 / `aud` 不一致 |
| ERR-VAL-003 | 422 | JWT に `terminal_id` クレームが存在しない |

---

## 2. ACK エンドポイント: POST /api/v1/work-assignments/{id}/ack

端末が割当を受信・確認したことをサーバーに通知するエンドポイント。

### 2-1. 概要

| 項目 | 値 |
|---|---|
| HTTP メソッド | POST |
| URL | `/api/v1/work-assignments/{id}/ack` |
| 担当バイナリ | terminal-api（ポート 8080）|
| 認証方式 | JWT Bearer（`aud: "terminal-api"`）|
| Idempotency-Key | 必須 |
| 関連 FR | FR-SY-013 |

`{id}` は TBL-052 `work_assignments` の `id`（UUID v7）。

### 2-2. リクエスト

```http
POST /api/v1/work-assignments/019682ab-7c1f-7000-a1b2-3c4d5e6f7890/ack
Authorization: Bearer eyJhbGci...
Idempotency-Key: 019682ab-7c1f-7002-a1b2-3c4d5e6f7890
Content-Type: application/json

{}
```

ボディは空オブジェクト `{}` を送付する。

### 2-3. レスポンス（HTTP 200）

```json
{
  "data": {
    "assignment_id": "019682ab-7c1f-7000-a1b2-3c4d5e6f7890",
    "status": "acknowledged",
    "acknowledged_at": "2026-05-18T10:30:05.000Z"
  },
  "meta": {
    "request_id": "019682ab-7c1f-7003-a1b2-3c4d5e6f7890",
    "server_time": "2026-05-18T10:30:05.000Z",
    "api_version": "v1"
  }
}
```

ACK 受信時、TBL-053 の対応レコードを `status = 'acknowledged'`、`acknowledged_at = NOW()` に更新する。また TBL-052 の `work_assignments.status` を `'dispatched'` から `'acknowledged'` に遷移させる（Append-only の状態遷移であるため、実装上は新しいステータスレコードを挿入する方式を採用する）。

### 2-4. エラーコード

| ERR-CODE | HTTP | 発生条件 |
|---|---|---|
| ERR-AUTH-001 | 401 | Authorization ヘッダ不足 / JWT 不正 |
| `404 Not Found` | 404 | `{id}` が存在しない、または JWT.terminal_id と target_terminal_id が不一致 |
| ERR-DB-001 | 409 | Idempotency-Key 重複（同 Key・異 Body）|

---

**本節で確定した方針**
- **API-sync-004 は terminal-api（ポート 8080）が担当し、JWT Bearer（`aud: "terminal-api"`、`terminal_id` クレーム必須）で認証することを確定した。**
- **レスポンスは `Content-Type: text/event-stream` とし、4 イベントタイプ（assignment.created / cancelled / updated / keepalive）を RFC 8895 フォーマットで配信することを確定した。**
- **Last-Event-ID は UUID v7 タイムスタンプ比較で再開位置を特定し、対応レコードが存在しない場合は TBL-052 の初期配信クエリ（target_terminal_id + status IN pending,dispatched + received_at ASC）を実行することを確定した。**
- **keep-alive ping は CFG-029（25 秒）間隔で送信し、IIS ARR のバッファリング無効化と合わせて IIS リバースプロキシ通過を保証することを確定した。**
- **TBL-053 sse_dispatch_log は queued → dispatched → acknowledged / failed の順で更新し、ACK エンドポイントが acknowledged 状態への遷移トリガーとなることを確定した。**

---

## 参照業界分析

### 必須
- [`90_業界分析/09_セキュリティとアクセス制御.md`](../../../90_業界分析/09_セキュリティとアクセス制御.md)

### 関連
- [`90_業界分析/06_品質管理とトレーサビリティ.md`](../../../90_業界分析/06_品質管理とトレーサビリティ.md)
