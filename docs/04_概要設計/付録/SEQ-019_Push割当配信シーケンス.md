# SEQ-019 Push 割当配信シーケンス（Push 割当受信〜端末配信〜開始）

本文書は MES/ERP から Webhook で作業指示が送られ、端末の作業員が割当から作業を開始するまでの全体シーケンス（UC-041 + UC-042）を記述する。

---

## 1. シーケンス概要

関係するコンポーネント:
- **MES/ERP**: 親機（外部システム）
- **master-api**: Webhook 受信・external_key 解決・DB 保存
- **terminal-api**: SSE 配信・sse_dispatch_log 管理
- **DB（PostgreSQL）**: work_assignments・sse_dispatch_log・external_key_bindings
- **端末（React Native）**: SSE クライアント・AssignmentInbox・UI

図（将来追加予定）: drawio-authoring スキルで SEQ-019 シーケンス図を作成し、`img/fig_des_seq_push_assignment.drawio` + `.svg` として保存する。

---

## 2. 正常系シーケンス（テキスト記述）

### Phase 1: Webhook 受信（UC-041 主シナリオ 1〜7）

```
参加者: MES/ERP（外部）、master-api（wnav_master_api:8081）、terminal-api（wnav_terminal_api:8080）、DB（PostgreSQL）

1. MES/ERP が POST /api/v1/work-assignments に Idempotency-Key・X-WNAV-Signature ヘッダを付与してリクエスト送信
2. master-api が HMAC-SHA256 署名を KEY-010 で検証（失敗 → 401）
3. master-api が Idempotency-Key を TBL-035 で確認（重複 → 409）
4. master-api が external_key_bindings（TBL-027）で各外部キーを UUID に解決（失敗 → pending_resolution INSERT + 422）
5. master-api が work_assignments（TBL-052）に status='pending' で INSERT
6. master-api が sse_dispatch_log（TBL-053）に delivery_status='queued' で INSERT
7. master-api が内部チャネル（MSG-006 internal.assignment_created）を通じて terminal-api の SSE dispatcher に通知
8. master-api が 202 Accepted を MES/ERP に返却
```

### Phase 2: SSE 配信（UC-041 代替シナリオ A1）

```
参加者: terminal-api（wnav_terminal_api:8080）、DB（PostgreSQL）、端末（React Native）

9.  terminal-api の SSE dispatcher が MSG-006 シグナルを受信
10. SSE dispatcher が接続中の対象端末（terminal_id 一致）の SSE セッションに assignment.created イベントを送信
11. sse_dispatch_log が delivery_status='sent' に更新される
```

### Phase 3: 端末受信・表示（UC-042 事前条件）

```
参加者: 端末（React Native）、LocalDB（SQLite）、terminal-api（wnav_terminal_api:8080）、DB（PostgreSQL）

12. 端末の EventSource クライアントが assignment.created イベントを受信
13. AssignmentInbox が local_assignments（SQLite）に INSERT
14. last_event_id を AsyncStorage に永続化
15. UI が SCR-HA-002 に AssignmentBanner（CMP-HA-021）または AssignmentListPanel（CMP-HA-022）を表示
16. 端末が POST /api/v1/work-assignments/{id}/ack を送信 → sse_dispatch_log が delivery_status='ack' に更新
```

### Phase 4: 作業開始（UC-042 主シナリオ 1〜6）

```
参加者: Operator（端末）、FE-HA（React Native）、terminal-api（wnav_terminal_api:8080）、DB（PostgreSQL）

17. 作業員がバナーまたはリスト行をタップ
18. AssignmentDetailDialog（CMP-HA-023）が表示される
19. 作業員が「開始」ボタンをタップ（明示操作必須）
20. 端末が PUT /api/v1/work-assignments/{id} で status=accepted に更新
21. 端末が POST /api/v1/work-executions で case_id を生成
22. work_assignments.case_id に case_id がバインドされ、status=in_progress に遷移
23. 端末がナビゲーション画面（SCR-HA-004）へ遷移し UC-001 Step 5 に合流
```

---

## 3. 異常系・縮退シーケンス

### 3-1. 端末オフライン中に割当が到着した場合

```
状態: Phase 1〜2 は完了（work_assignments 行が存在）

- 端末が再接続した際に Last-Event-ID 付き SSE で差分配信
- 再接続できない場合は API-sync-005（GET /work-assignments）で Pull 取得
```

### 3-2. external_key 解決不能

```
状態: Phase 1 手順 4 で 422 を返却

- work_assignments に status='pending_resolution' で INSERT
- IT 担当が external_key_binding を設定後、管理コンソールで「再解決」操作 → status='pending' に遷移し Phase 2 へ
```

### 3-3. BAT-014 再送スケジューラー

```
- sse_dispatch_log の delivery_status='sent'・ack_at=NULL の行を 1 分周期で検出
- retry_count が CFG-030（デフォルト 5）を超過した場合は failed に遷移
```

---

## 4. 関連ドキュメント

- UC-041・UC-042 記述: `docs/03_要件定義/機能要件/17_UC記述_Push型作業指示.md`
- IF-008 設計: `docs/04_概要設計/05_外部インターフェース設計/13_作業指示Push受信IF（IF-008）.md`
- IF-009 設計: `docs/04_概要設計/05_外部インターフェース設計/14_端末SSE配信IF（IF-009）.md`
- TBL-052/053 DDL: `docs/05_詳細設計/01_データベース詳細設計/05_作業指示テーブルDDL（TBL-052〜053）.md`
