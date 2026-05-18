# 05 Outbox 実績送信 API（IF-002）

本章の責務は、子機から親機へ作業実績・証拠記録を送信する Outbox Push API（IF-002）の設計を確定することである。対応する API 識別子: `API-sync-002`。

**担当バイナリ: `wnav_terminal_api`（8080）**

IF-002 の送信元は `wnav_terminal_api` 内の OutboxWorker（BAT-002）である。`wnav_outbox` crate は `wnav_terminal_api` バイナリのみに依存し、`wnav_master_api` は Outbox 送信機能を持たない。

---

## 1. 設計方針

### 1-1. At-Least-Once + Idempotency = Exactly-Once 意味論

- `wnav_terminal_api` 内の子機バックエンド（BAT-002）が TBL-003（outbox_events）の PENDING 行を親機に POST する
- 親機は `Idempotency-Key` で重複検出し、既受信の場合は 200 OK を返す（再処理しない）
- これにより At-Least-Once 配信 + 親機側の Idempotency = Exactly-Once 意味論を実現する

### 1-2. Webhook 署名（HMAC-SHA256）

```http
POST {親機}/api/v1/sync/outbox/inbound
Authorization: Bearer {access_token}
Idempotency-Key: 018d0e81-7e4e-7f00-b8b5-1234567890ab
X-WNAV-Signature: sha256={HMAC-SHA256(KEY-002, request_body)}
Content-Type: application/json

{
  "event_type": "work_event",
  "event_id": "018d0e81-7e4e-7f00-b8b5-1234567890ab",
  "factory_id": "...",
  "payload": { ... },
  "occurred_at": "2026-05-17T10:30:00Z"
}
```

### 1-3. 再試行ポリシー

- 最大 CFG-002（デフォルト 5 回）
- 指数バックオフ: CFG-003（初期 60s）× 2^n（最大 CFG-004 = 3600s）
- 3 回目の失敗後（または `status` が DLQ）に MET-005（outbox.dlq_count）を increment
- DLQ 移行後は BAT-008（webhook_retry_scheduler）が 1 分ごとに再投入を試みる

---

## 2. 親機側の受信契約

子機モード設計では、本アプリは「子機」として動作する。親機 ERP/MES の受信 API 仕様は本アプリが確定できない（親機ごとに異なる）。そのため、送信側（本アプリ）が提供するのは以下のみ：

| 提供するもの | 内容 |
|---|---|
| ペイロードスキーマ（送信側）| `{event_type, event_id, factory_id, payload, occurred_at}` の JSON |
| HMAC-SHA256 署名ヘッダ | `X-WNAV-Signature: sha256={value}`（KEY-002 で計算）|
| Idempotency-Key | UUID v7 形式のヘッダ |
| エラー応答の期待値 | 200-299 = 成功、429 = レート制限（retry_after）、400-499 = DLQ 移行候補 |

親機側の具体的な受信処理は親機ベンダーが実装する（本アプリのスコープ外）。

---

**本節で確定した方針**
- **IF-002 の送信元は `wnav_terminal_api` 内の OutboxWorker（BAT-002）であることを確定した。`wnav_outbox` crate は `wnav_terminal_api` バイナリのみに依存する。**
- **IF-002 は Outbox Pattern（BAT-002）+ HMAC-SHA256 署名 + Idempotency-Key で Exactly-Once 意味論を実現する設計を確定した。**
- **再試行は指数バックオフで最大 5 回とし、失敗後は DLQ に移行して MET-005 でアラートを発生させる。**
- **親機側の受信契約（ペイロードスキーマ・署名ヘッダ）を定義し、親機ベンダーとのインテグレーション基準を確定した。**

---

## 参照業界分析

### 必須
- [`90_業界分析/27_オフライン同期とデータ整合性.md`](../../90_業界分析/27_オフライン同期とデータ整合性.md)

### 関連
- [`90_業界分析/17_サプライチェーンと作業依存性.md`](../../90_業界分析/17_サプライチェーンと作業依存性.md)
