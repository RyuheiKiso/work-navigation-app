# 14 端末 SSE 配信 IF（IF-009）

本章は IF-009（terminal-api:8080 → ハンディ端末への Server-Sent Events 配信）の概要設計を記述する。IF-008 で受信した作業指示を端末にリアルタイム配信する方式・認証・イベント仕様・再接続規約・オフライン縮退を確定する。

---

## 1. IF-009 概要

| 項目 | 内容 |
|---|---|
| IF-ID | IF-009 |
| 名称 | 端末 SSE 配信（割当通知）|
| 方向 | terminal-api:8080 → ハンディ端末（Push）|
| プロトコル | text/event-stream（Server-Sent Events / HTTP/1.1 または HTTP/2）|
| 認証 | JWT Bearer（aud=terminal-api、`terminal_id` クレーム必須）|
| エンドポイント | GET /api/v1/sse/assignments（API-sync-004）|
| 担当バイナリ | terminal-api（8080）|
| 縮退方針 | SSE 不可時は端末が API-sync-005（GET /api/v1/work-assignments）を 30 秒間隔でポーリング |

---

## 2. 認証方式

| 項目 | 内容 |
|---|---|
| 認証方式 | JWT Bearer（RS256）|
| 要求クレーム | `aud: terminal-api`・`terminal_id: {UUID}` の 2 クレーム必須 |
| 発行元 | 端末ログイン時の POST /auth/login レスポンス（API-auth-001）|
| 有効期限 | 8 時間（CFG-005）。期限切れ時は端末がリフレッシュ（API-auth-002）してから再接続 |

`terminal_id` クレームが存在しない JWT は 401 で拒否する。これは端末ごとの割当フィルタに不可欠である。

---

## 3. イベント仕様

### 3-1. event types

| event type | 発火タイミング | data フィールド（JSON）|
|---|---|---|
| `assignment.created` | work_assignments が INSERT されたとき（status=pending）| assignment_id, work_pattern_id, lot_id, due_at, priority, suggested_worker_id, suggested_equipment_id |
| `assignment.cancelled` | status が cancelled または expired に遷移したとき | assignment_id |
| `assignment.updated` | due_at または priority が変更されたとき | assignment_id, due_at, priority |
| `keepalive` | 25 秒ごと（CFG-029）| （データなし・コメント行として送信）|

### 3-2. イベントフォーマット

```
id: <assignment_id（UUID v7）>
event: assignment.created
data: {"assignment_id":"<UUID>","work_pattern_id":"<UUID>","lot_id":"<UUID|null>","due_at":"<ISO8601|null>","priority":2,"suggested_worker_id":"<UUID|null>","suggested_equipment_id":"<UUID|null>"}

```

`id:` フィールドに assignment_id を使用することで、再接続時の `Last-Event-ID` ヘッダが自動的に最後に受信した割当 ID を示す。

---

## 4. 接続時の初期配信

| タイミング | 動作 |
|---|---|
| 初回接続（Last-Event-ID なし）| terminal_id 宛の `status IN ('pending','dispatched')` の全割当を `received_at` 昇順で配信 |
| 再接続（Last-Event-ID = assignment_id X）| UUID v7 のタイムスタンプ比較で X より新しい（received_at が X より後の）pending/dispatched 行を一括配信 |

これにより端末がオフライン中に到着した割当を再接続時に漏れなく受信できる。

---

## 5. keep-alive

25 秒間隔（CFG-029）で以下のコメント行を送信する:

```
: ping

```

これはプロキシや IIS のアイドル接続タイムアウト（デフォルト 30〜60 秒）による強制切断を防ぐ。IIS リバースプロキシ経由の場合は HTTP/1.1 の Transfer-Encoding: chunked を利用する。

---

## 6. 縮退方針

| 障害 | 縮退動作 |
|---|---|
| SSE 接続失敗（5xx 応答・接続タイムアウト）| 端末は EventSource の自動再接続機能で 3 秒後に再試行 |
| 30 秒以上接続不可 | 端末は SSE を諦め、API-sync-005（GET /api/v1/work-assignments?status=pending&terminal_id=XXX）を 30 秒間隔でポーリング |
| IIS リバースプロキシの長期接続制限 | keep-alive（25 秒）でタイムアウト前に送信し、切断を防止。IIS `connectionTimeout` を 60 秒以上に設定することを運用手順に記載 |
| BAT-014 再送スケジューラー | sse_dispatch_log の delivery_status が sent でない行を 1 分周期で再送。ack が確認できない場合は retry_count を加算し、上限（CFG-030）超過で failed に遷移 |

---

## 7. sse_dispatch_log との連携

端末は割当イベントを受信した際に `POST /api/v1/work-assignments/{id}/ack` で配信確認を送信する。sse_dispatch_log（TBL-053）の対応行が `delivery_status=ack` に更新される。

これは Last-Event-ID による HTTP/EventSource 標準の再送制御と二重化しており、配信保証の堅牢性を高める。

---

**本節で確定した方針**
- IF-009 を JWT Bearer（terminal_id クレーム必須）・Last-Event-ID 再接続・25 秒 keep-alive 方式として確定する。
- 接続時の初期配信により、オフライン中に到着した割当を再接続時に漏れなく受信できる設計を確定する。
- SSE 不可時の Pull フォールバック（API-sync-005、30 秒間隔）を設計必達として確定し、リアルタイム性と確実な受信を両立する。

---

## 参照

### 必須
- [`03_要件定義/機能要件/13_外部インタフェース要件.md`](../../03_要件定義/機能要件/13_外部インタフェース要件.md) §10（IF-009 要件定義）
- [`13_作業指示Push受信IF（IF-008）.md`](13_作業指示Push受信IF（IF-008）.md)（本 IF の上流処理）

### 関連
- [`03_要件定義/機能要件/07_UC記述_子機モード同期.md`](../../03_要件定義/機能要件/07_UC記述_子機モード同期.md) UC-022（通信断縮退方針との整合）
- [`90_業界分析/27_オフライン同期とデータ整合性.md`](../../90_業界分析/27_オフライン同期とデータ整合性.md)
