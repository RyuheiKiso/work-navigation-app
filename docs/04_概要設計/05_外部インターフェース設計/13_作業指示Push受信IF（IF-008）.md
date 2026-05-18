# 13 作業指示 Push 受信 IF（IF-008）

本章は IF-008（親機 → master-api:8081 への Webhook 受信）の概要設計を記述する。MES/ERP が本システムへ作業指示を能動的に送信する方式・認証・データ契約・external_key 解決・エラー応答を確定する。

---

## 1. IF-008 概要

| 項目 | 内容 |
|---|---|
| IF-ID | IF-008 |
| 名称 | 親機 Webhook 受信（作業指示 Push）|
| 方向 | 親機（ERP/MES）→ master-api:8081（受信）|
| プロトコル | HTTPS/REST/JSON + TLS 1.3 |
| 認証 | HMAC-SHA256（KEY-010）|
| エンドポイント | POST /api/v1/work-assignments（API-sync-003）|
| 担当バイナリ | master-api（8081）|
| 縮退方針 | 一時障害時は 503 を返却し親機側がリトライ。解決不能の external_key は `pending_resolution` ステータスで保留 |

---

## 2. 認証方式

IF-002（Outbox 実績送信）の HMAC-SHA256 規約（KEY-002）と同一方式を採用する。ただし鍵は独立した KEY-010 を使用し、送信側（本システム → 親機）と受信側（親機 → 本システム）で鍵を分離する。

| 項目 | 内容 |
|---|---|
| 署名アルゴリズム | HMAC-SHA256 |
| 署名ヘッダ | `X-WNAV-Signature: sha256={hex}` |
| 署名対象 | リクエストボディ全体（UTF-8 バイト列）|
| 鍵管理 | KEY-010（180 日ローテーション・env 保管・`webhook.receiver.hmac_key` 設定参照）|
| Idempotency | `Idempotency-Key: {uuid-v7}` ヘッダ必須。24h TTL（CFG-002 と共通の冪等キー管理テーブル TBL-035 を使用）|

---

## 3. データ契約

### 3-1. リクエストボディ（最小必須フォーマット）

```json
{
  "external_order_id":       "<MES 側製造オーダー ID>",
  "external_system":         "<MES 識別子（例: SAP・OMRON-MES）>",
  "work_pattern_key":        "<work_patterns の外部キー値>",
  "target_terminal_key":     "<端末の外部キー値>",
  "lot_id_ext":              "<ロット外部キー値（任意）>",
  "suggested_worker_key":    "<推奨作業員外部キー値（任意）>",
  "suggested_equipment_key": "<推奨設備外部キー値（任意）>",
  "due_at":                  "<ISO 8601 UTC・任意>",
  "priority":                2
}
```

`priority` の値: 1 = 緊急 / 2 = 通常（デフォルト）/ 3 = 低

### 3-2. レスポンスコード

| HTTP ステータス | 意味 | 親機側の対処 |
|---|---|---|
| 202 Accepted | 受理・DB 保存・SSE キュー登録完了 | 正常終了 |
| 401 Unauthorized | HMAC 署名不正（ERR-VAL-032）| 鍵設定を確認・訂正してリトライ |
| 409 Conflict | 重複受信（Idempotency-Key 一致）| 正常（200 相当として扱う）|
| 422 Unprocessable Entity | external_key 解決不能（ERR-BIZ-027）| IT 担当が external_key_binding を設定後にリトライ |
| 503 Service Unavailable | 一時障害 | 指数バックオフ（1→4→16 秒）でリトライ |

---

## 4. external_key 解決方針

UC-021（外部一意キー解決）の既存ロジックを流用する。

```
受信キー             external_key_binding          内部 ID
work_pattern_key  →  (external_system, key_type='work_pattern') → work_pattern_id (UUID)
target_terminal_key → (external_system, key_type='terminal')    → target_terminal_id (UUID)
lot_id_ext        →  (external_system, key_type='lot')          → lot_id (UUID)
```

解決優先順位: 有効期間（valid_from ≦ now ≦ valid_to）内の 1 件を採用。複数件は valid_from 降順で最新を優先。

**解決不能時の動作**: 422 を返却し、同時に `status='pending_resolution'` の work_assignments 行を INSERT する。IT 担当が external_key_binding を設定してから管理コンソールで「再解決」操作を行うことで通常フローへ復帰する。

---

## 5. エラー処理と縮退

| エラー種別 | 処置 |
|---|---|
| HMAC 署名不正 | 401 即時返却。DB に痕跡を残さない。LOG-XXX（SECURITY レベル）にログ記録 |
| Idempotency-Key 重複 | 409 返却。work_assignments は変更しない |
| external_key 全件解決不能 | 422 返却 + `pending_resolution` 行 INSERT + IT 担当通知（ERR-BIZ-027）|
| DB 障害（INSERT 失敗）| 503 返却。親機側リトライを期待 |
| リクエストボディ不正（JSON パースエラー等）| 400 返却。親機側でリクエストを修正 |

---

**本節で確定した方針**
- IF-008 を HMAC-SHA256（KEY-010）・Idempotency-Key UUID v7・202 Accepted（非同期受理）方式として確定する。
- 認証方式は IF-002 と同一規約を採用し、鍵（KEY-010）を送信用 KEY-002 から分離することで障害切り分けを容易にする。
- external_key 解決不能時も `pending_resolution` 行を作成して IT 担当に通知する設計を確定し、割当が無言でロストすることを防ぐ。

---

## 参照

### 必須
- [`03_要件定義/機能要件/13_外部インタフェース要件.md`](../../03_要件定義/機能要件/13_外部インタフェース要件.md) §9（IF-008 要件定義）
- [`04_概要設計/05_外部インターフェース設計/02_Outbox実績送信IF（IF-002）.md`](02_Outbox実績送信IF（IF-002）.md)（HMAC-SHA256・Idempotency-Key 流用元）

### 関連
- [`03_要件定義/機能要件/07_UC記述_子機モード同期.md`](../../03_要件定義/機能要件/07_UC記述_子機モード同期.md) UC-021（external_key 解決ロジック流用元）
- [`14_端末SSE配信IF（IF-009）.md`](14_端末SSE配信IF（IF-009）.md)（本 IF の受信後に続く配信工程）
