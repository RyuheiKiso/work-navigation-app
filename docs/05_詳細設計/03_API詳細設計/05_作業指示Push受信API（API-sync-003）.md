# 05 作業指示 Push 受信 API（API-sync-003）

本章は外部システムからの作業指示 Push 受信エンドポイント API-sync-003 を確定する。HMAC-SHA256 署名検証・Idempotency-Key による重複防止・external_key 解決・work_assignments テーブルへの Append-only 挿入・SSE キュー登録の全挙動をコーディング直前精度で記述する。

> **担当バイナリ**: 本章のエンドポイントは **`wnav_master_api`（ポート 8081）** が担当する。外部システムからの作業指示受信は管理系の責務であり、`wnav_terminal_api` には存在しない。

---

## 1. API-sync-003: POST /api/v1/work-assignments

### 1-1. 概要

| 項目 | 値 |
|---|---|
| API-ID | API-sync-003 |
| HTTP メソッド | POST |
| URL | `/api/v1/work-assignments` |
| 担当バイナリ | master-api（ポート 8081）|
| 認証方式 | HMAC-SHA256（KEY-010）+ Idempotency-Key（UUID v7）|
| Idempotency-Key | 必須（`Idempotency-Key` ヘッダ、UUID v7 形式）|
| レート制限カテゴリ | 同期（60 req / 60s）|
| 関連 FR | FR-SY-012 |

外部システム（生産管理システム・ERP 等）が作業指示を Push する際に呼び出すエンドポイント。受信後は TBL-052 work_assignments に Append-only で挿入し、該当端末の SSE キュー（MSG-006）に登録することで、端末へのリアルタイム配信を起動する。

### 1-2. 認証詳細

#### HMAC-SHA256 署名検証（KEY-010）

外部システムは共有シークレット KEY-010（CFG-028 で管理）を用いて、以下の手順で署名を生成し `X-Signature-256` ヘッダに付与する。

```
timestamp = Unix 秒（文字列）
nonce     = UUID v4（文字列）
body_hash = SHA-256( リクエストボディ の UTF-8 バイト列 )
signed_str = timestamp + "\n" + nonce + "\n" + HEX(body_hash)
signature = "sha256=" + HEX( HMAC-SHA256( KEY-010, signed_str ) )
```

| ヘッダ名 | 必須 | 説明 |
|---|---|---|
| `X-Signature-256` | 必須 | `sha256=<hex>` 形式の HMAC 署名 |
| `X-Timestamp` | 必須 | Unix 秒（文字列）。現在時刻との差が CFG-028 の `hmac_timeout_ms`（5000 ms）を超える場合、リプレイ攻撃として拒否（ERR-VAL-032）|
| `X-Nonce` | 必須 | UUID v4 文字列。TTL 内に同一 Nonce を再利用した場合も ERR-VAL-032 で拒否 |
| `Idempotency-Key` | 必須 | UUID v7 形式。TBL-035 で検証（§ 1-5 参照）|

### 1-3. リクエストスキーマ

```json
{
  "external_order_id": "ERP-2026-00042",
  "external_system": "sap_pp",
  "work_pattern_key": "welding_pattern_A",
  "target_terminal_key": "TERMINAL-LINE1-01",
  "lot_id_ext": "LOT-SAP-9900042",
  "suggested_worker_key": "WORKER-EMP0042",
  "suggested_equipment_key": "EQUIP-WELDER-03",
  "due_at": "2026-05-18T15:00:00.000Z",
  "priority": 2
}
```

| フィールド | 型 | 必須 | 制約 | 説明 |
|---|---|---|---|---|
| `external_order_id` | string | 必須 | 1〜128 文字 | 外部システムの注文 / 指示 ID |
| `external_system` | string | 必須 | 1〜64 文字、英数字・アンダースコア | 送信元システム識別子（例: `sap_pp`, `custom_mes`）|
| `work_pattern_key` | string | 必須 | 1〜128 文字 | 作業パターンの外部キー（TBL-007 に解決）|
| `target_terminal_key` | string | 必須 | 1〜128 文字 | 対象端末の外部キー（TBL-033 に解決）|
| `lot_id_ext` | string | 任意 | 1〜128 文字 | ロットの外部キー（TBL-024 に解決）|
| `suggested_worker_key` | string | 任意 | 1〜128 文字 | 推奨作業者の外部キー（TBL-016 に解決）|
| `suggested_equipment_key` | string | 任意 | 1〜128 文字 | 推奨設備の外部キー（TBL-018 に解決）|
| `due_at` | string (ISO 8601 UTC) | 任意 | — | 期限日時 |
| `priority` | integer | 任意 | 1〜5（デフォルト 3）| 優先度（1 が最高）|

### 1-4. レスポンススキーマ

#### HTTP 202 Accepted（正常受付）

```json
{
  "data": {
    "assignment_id": "019682ab-7c1f-7000-a1b2-3c4d5e6f7890",
    "status": "pending",
    "target_terminal_id": "019682ab-7c1f-7000-0000-000000000010",
    "received_at": "2026-05-18T10:30:00.000Z"
  },
  "meta": {
    "request_id": "019682ab-7c1f-7001-a1b2-3c4d5e6f7890",
    "server_time": "2026-05-18T10:30:00.000Z",
    "api_version": "v1"
  }
}
```

| フィールド | 型 | 説明 |
|---|---|---|
| `data.assignment_id` | string (UUID v7) | 採番された作業割当 ID（TBL-052）|
| `data.status` | string | 常に `"pending"`（端末へ SSE 配信待ち）|
| `data.target_terminal_id` | string (UUID v7) | 解決された端末内部 ID（TBL-033）|
| `data.received_at` | string (ISO 8601 UTC) | サーバー受信時刻（サーバー側で付与）|

#### エラーレスポンス一覧

| HTTP | ERR-CODE | 発生条件 |
|---|---|---|
| 401 | ERR-VAL-032 | HMAC 署名不正 / タイムスタンプ超過 / Nonce 再利用 |
| 409 | ERR-DB-001 | Idempotency-Key 重複（同 Key・異 Body）|
| 422 | ERR-BIZ-027 | external_key の解決不能（work_pattern_key / target_terminal_key 等が存在しない）|
| 503 | ERR-SYS-003 | DB 挿入失敗（一時的障害）|

### 1-5. Idempotency 処理フロー

TBL-035 `idempotency_keys` テーブルを使用して以下の手順で重複リクエストを処理する。

1. `Idempotency-Key` ヘッダを取得する。ヘッダが存在しない場合は ERR-VAL-001（422）を返す。
2. TBL-035 で `key = Idempotency-Key AND endpoint = 'POST /api/v1/work-assignments'` のレコードを検索する。
3. レコードが存在し `body_hash` が一致する場合: 保存済みレスポンス（HTTP 202 + JSON ボディ）をそのまま返す（DB 再書き込みなし）。
4. レコードが存在し `body_hash` が不一致の場合: ERR-DB-001（409）を返す。
5. レコードが存在しない場合: 通常処理を続行し、処理完了後に TBL-035 にレコードを挿入する（TTL 24h）。

### 1-6. external_key 解決ステップ

UC-021 の external_key 解決フローを流用する。リクエストボディの外部キー文字列を以下の順序で WNAV 内部 UUID に変換する。変換失敗は全件収集後に ERR-BIZ-027 でまとめて返す。

| キー種別 | フィールド | 解決テーブル | 解決条件 |
|---|---|---|---|
| 作業パターン | `work_pattern_key` | TBL-007 sops | `external_key = work_pattern_key AND deleted_at IS NULL` |
| 対象端末 | `target_terminal_key` | TBL-033 handy_terminals | `external_key = target_terminal_key AND deleted_at IS NULL` |
| ロット | `lot_id_ext` | TBL-024 lots | `external_key = lot_id_ext AND deleted_at IS NULL`（任意フィールド）|
| 推奨作業者 | `suggested_worker_key` | TBL-016 users | `external_key = suggested_worker_key AND deleted_at IS NULL`（任意フィールド）|
| 推奨設備 | `suggested_equipment_key` | TBL-018 equipments | `external_key = suggested_equipment_key AND deleted_at IS NULL`（任意フィールド）|

ERR-BIZ-027 レスポンスには解決できなかったキー種別と値を `violations` 配列で列挙する。

```json
{
  "type": "https://errors.wnav.example.com/ERR-BIZ-027",
  "title": "external_key_resolution_failed",
  "status": 422,
  "detail": "external_key の解決に失敗しました。",
  "instance": "/api/v1/work-assignments",
  "error_id": "ERR-BIZ-027",
  "violations": [
    {
      "field": "work_pattern_key",
      "value": "welding_pattern_UNKNOWN",
      "message": "作業パターン外部キーが見つかりません。"
    }
  ]
}
```

### 1-7. work_assignments INSERT ロジック（Append-only ポリシー）

external_key 解決後、TBL-052 `work_assignments` に以下の内容で INSERT する。Append-only ポリシーに基づき UPDATE / DELETE は DB ロール権限で物理禁止する。

```sql
INSERT INTO work_assignments (
  id,
  external_order_id,
  external_system,
  sop_id,
  target_terminal_id,
  lot_id,
  suggested_worker_id,
  suggested_equipment_id,
  due_at,
  priority,
  status,
  received_at,
  created_at
) VALUES (
  gen_random_uuid(),      -- UUID v7 相当（アプリ層で生成）
  $external_order_id,
  $external_system,
  $sop_id,                -- work_pattern_key から解決
  $target_terminal_id,    -- target_terminal_key から解決
  $lot_id,                -- lot_id_ext から解決（NULL 可）
  $suggested_worker_id,   -- suggested_worker_key から解決（NULL 可）
  $suggested_equipment_id,-- suggested_equipment_key から解決（NULL 可）
  $due_at,                -- NULL 可
  COALESCE($priority, 3),
  'pending',
  NOW(),
  NOW()
)
```

状態遷移は status カラムへの追記イベントで管理する。`pending` → `dispatched` → `acknowledged` / `cancelled` の順序で進む。過去状態への強制変更は禁止する。

### 1-8. SSE キュー登録（MSG-006）

INSERT 成功後、以下の手順で SSE キューに登録し、端末への配信を開始する。

1. TBL-053 `sse_dispatch_log` に `assignment_id` / `target_terminal_id` / `event_type = 'assignment.created'` / `status = 'queued'` のレコードを挿入する。
2. メモリ内 SSE ブロードキャストチャネル（MSG-006）に `WorkAssignmentCreated` メッセージを送信する。
3. 当該端末が SSE 接続中の場合、API-sync-004 のハンドラが即時配信する。接続中でない場合は TBL-053 が未配信レコードとして残り、次回接続時の初期配信ロジック（§ 1 in API-sync-004）で送信される。

### 1-9. RBAC

外部システムは JWT ではなく HMAC-SHA256 で認証するため、RBAC ロールは適用しない。HMAC 署名の検証に成功したリクエストのみが受け付けられる。

---

## 2. エラーコード一覧

| ERR-CODE | HTTP | title | 発生条件 |
|---|---|---|---|
| ERR-VAL-032 | 401 | hmac_authentication_failed | HMAC 署名不正 / X-Timestamp 超過（CFG-028 = 5000 ms）/ Nonce 再利用 |
| ERR-BIZ-027 | 422 | external_key_resolution_failed | work_pattern_key / target_terminal_key 等の解決失敗 |
| ERR-DB-001 | 409 | idempotency_replay_conflict | 同 Idempotency-Key・異 Body の再送 |
| ERR-VAL-001 | 422 | required_field_missing | 必須フィールド（external_order_id 等）が不足 |
| ERR-SYS-003 | 503 | database_write_failed | DB 挿入失敗（一時的障害）|

---

**本節で確定した方針**
- **API-sync-003 は master-api（ポート 8081）が担当し、外部システムからの作業指示 Push を HMAC-SHA256（KEY-010）+ Idempotency-Key で認証・重複排除した上で受け付けることを確定した。**
- **external_key 解決は UC-021 フローを流用し、work_pattern / terminal / lot / worker / equipment の 4〜5 キー種別を一括解決することを確定した。解決失敗は ERR-BIZ-027 で全件報告する。**
- **TBL-052 work_assignments への挿入は Append-only ポリシーに従い、UPDATE / DELETE を DB ロール権限で物理禁止することを確定した。**
- **INSERT 成功後は TBL-053 sse_dispatch_log にレコードを挿入し、MSG-006 チャネルに WorkAssignmentCreated を送信することで端末への SSE 配信を起動することを確定した。**

---

## 参照業界分析

### 必須
- [`90_業界分析/09_セキュリティとアクセス制御.md`](../../../90_業界分析/09_セキュリティとアクセス制御.md)

### 関連
- [`90_業界分析/06_品質管理とトレーサビリティ.md`](../../../90_業界分析/06_品質管理とトレーサビリティ.md)
