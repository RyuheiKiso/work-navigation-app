# 13a ハッシュチェーン破断時のクライアント resume 判定

本章は端末（ハンディ APP）がハッシュチェーン破断を検知した場合、および破断修復後に作業を再開する場合の
クライアント側状態管理と resume 判定を規定する。

関連: ALG-025、SCR-HA-021、ERR-DB-004

---

## 1. サーバーから端末への通知経路

| イベント | 通知方式 |
|---|---|
| 破断検知（初回）| API レスポンス 500 ERR-DB-003（hash_chain_broken）|
| 補正承認待ち中 | 次回 Outbox sync または定期 poll で `hash_chain_status: "SUSPENDED"` を伝搬 |
| 補正完了・回復 | `hash_chain_status: "NORMAL"` を伝搬 |

---

## 2. 端末側の SUSPENDED 挙動

1. `app_settings.hash_chain_status` を SQLite に `'SUSPENDED'` として保存する
2. 影響を受ける `case_id` 群（`broken_at_block_id` の case_id）の全 Step 操作をブロックする
3. 既存 `notifySupervisor('HASH_CHAIN_INCONSISTENCY')` + `suspendCurrentExecution()` を実行する
4. SCR-HA-021（フルスクリーンバナー「監査保留中：管理者承認待ち」）を表示する

**Emergency Mode との区別**: Emergency Mode は通信断による縮退動作（FR-ST-012）であり、
ハッシュチェーン破断の SUSPENDED は監査的な保留状態である。表示 UI を明確に区別する。

---

## 3. SUSPENDED からの resume 判定

### 3-1. 前提条件

- `quality_admin` が SCR-MC-008（2 名承認）で補正レコードを承認・INSERT 済みであること
- サーバーが `hash_chain_status: "NORMAL"` を broadcast していること

### 3-2. 端末側の resume フロー

```
1. 次回 sync で hash_chain_status: "NORMAL" を受信する
2. SQLite の app_settings.hash_chain_status を 'NORMAL' に更新する
3. 影響 case_id の最後の補正ブロック（is_correction=TRUE）の chain_hash を取得する
4. 以降の work_events 記録では prev_hash = 補正ブロックの chain_hash として継続する（D2 / ALG-025）
5. SCR-HA-021 バナーを解除する
```

---

## 4. SUSPENDED からの resume 不可ケース

端末が SUSPENDED 中に再起動した場合:

- SQLite から `app_settings.hash_chain_status = 'SUSPENDED'` を復元する
- ブロック状態を継続する（自動 resume は禁止）
- ユーザーが明示的に同期操作を行い、サーバーから `NORMAL` を受け取ることで resume する

---

## 5. オフライン中の SUSPENDED 検知

端末がオフライン（DISCONNECTED または EMERGENCY_MODE）の場合:

- ERR-DB-003 を受け取れない可能性がある
- 次回 Outbox sync 成功時に `hash_chain_status` を確認し、`SUSPENDED` であればブロック状態に移行する

---

## 参照

- [`09_運用・保守/障害対応/08_ハッシュチェーン破断対応.md`](../../09_運用・保守/障害対応/08_ハッシュチェーン破断対応.md) — P1 プレイブック
- [`05_詳細設計/07_アルゴリズム詳細設計/03a_ハッシュチェーン破断時の補正レコード継続規則.md`](../../05_詳細設計/07_アルゴリズム詳細設計/03a_ハッシュチェーン破断時の補正レコード継続規則.md) — ALG-025
- SCR-HA-021: 補正承認待ちフルスクリーンバナー（画面詳細は別タスク）
