# 03 統合テストケース（API）

本章は REST API エンドポイントの統合テストケース TST-intg-001〜020 を確定する。テスト対象は認証（ログイン/失敗）・ステップイベント記録＋ハッシュチェーン・証拠ファイルアップロード・SOP 承認ワークフローの 4 領域である。各テストケースは HTTP メソッド・URL・リクエストボディ・期待レスポンス・テスト後 DB 状態を記述し、`axum_test` + `sqlx::test` で実行可能な精度とする。

---

## 1. 認証（TST-intg-001〜005）

### TST-intg-001: ログイン成功（ゴールデンパス）

| 項目 | 内容 |
|---|---|
| TST-ID | TST-intg-001 |
| HTTP | `POST /api/v1/auth/login` |
| リクエストボディ | `{"login_id": "OP001", "password": "correct_password_hash"}` |
| 前提条件 | users テーブルに OP001 が存在・is_active=true・failed_login_count=0 |
| 期待レスポンス | HTTP 200, `{"access_token": "<JWT>", "token_type": "Bearer", "expires_in": 3600}` |
| DB 状態（後）| auth_logs に SUCCEEDED レコードが 1 件挿入される |
| 対応 FR | FR-AU-001/002 |

### TST-intg-002: ログイン失敗（パスワード不一致）

| 項目 | 内容 |
|---|---|
| TST-ID | TST-intg-002 |
| HTTP | `POST /api/v1/auth/login` |
| リクエストボディ | `{"login_id": "OP001", "password": "wrong_password"}` |
| 期待レスポンス | HTTP 401, `{"error": "ERR-AUTH-001", "message": "Invalid credentials"}` |
| DB 状態（後）| users.failed_login_count が +1 される |
| 対応 FR | FR-AU-001 |

### TST-intg-003: ログイン失敗（存在しないユーザー）

| 項目 | 内容 |
|---|---|
| TST-ID | TST-intg-003 |
| HTTP | `POST /api/v1/auth/login` |
| リクエストボディ | `{"login_id": "NONEXISTENT", "password": "any"}` |
| 期待レスポンス | HTTP 401, `{"error": "ERR-AUTH-001"}` |
| 対応 FR | FR-AU-001 |

### TST-intg-004: 認証ログ記録

| 項目 | 内容 |
|---|---|
| TST-ID | TST-intg-004 |
| HTTP | 成功ログインの後、JWT を使用してエンドポイントにアクセス |
| 期待レスポンス | auth_logs テーブルに occurred_at・user_id・ip_address が記録される |
| 対応 FR | FR-AU-006 |

### TST-intg-005: 期限切れ JWT での API アクセス

| 項目 | 内容 |
|---|---|
| TST-ID | TST-intg-005 |
| HTTP | `GET /api/v1/work-executions`（期限切れ JWT） |
| 期待レスポンス | HTTP 401, `{"error": "ERR-AUTH-002", "message": "Token expired"}` |
| 対応 FR | FR-AU-002 |

---

## 2. ステップイベント記録＋ハッシュチェーン（TST-intg-006〜010）

### TST-intg-006: ステップ完了の正常記録

| 項目 | 内容 |
|---|---|
| TST-ID | TST-intg-006 |
| HTTP | `POST /api/v1/step-events` |
| リクエストボディ | `{"exec_id": "<uuid>", "step_id": "<uuid>", "event_id": "<uuidv7>", "input": {"type": "Numeric", "value": 50.0, "unit": "mm"}}` |
| 前提条件 | work_executions に IN_PROGRESS の作業が存在 |
| 期待レスポンス | HTTP 201, `{"event_id": "<uuidv7>", "content_hash": "<sha256_hex>"}` |
| DB 状態（後）| work_events +1, hash_chain_blocks +1, outbox_events +1（全てアトミック）|
| 対応 FR | FR-NV-002, FR-EV-001, FR-SY-002 |

### TST-intg-007: ロックステップ違反（スキップ試行）

| 項目 | 内容 |
|---|---|
| TST-ID | TST-intg-007 |
| HTTP | `POST /api/v1/step-events` |
| リクエストボディ | current_step_index=1 の作業に targetStepIndex=3 でリクエスト |
| 期待レスポンス | HTTP 422, `{"error": "ERR-BIZ-001", "reason": "SKIPPED_STEP"}` |
| DB 状態（後）| work_events・hash_chain_blocks・outbox_events 全て変化なし |
| 対応 FR | FR-NV-001, BR-BUS-001 |

### TST-intg-008: 冪等性（同一 event_id の二重送信）

| 項目 | 内容 |
|---|---|
| TST-ID | TST-intg-008 |
| HTTP | 同一 event_id で `POST /api/v1/step-events` を 2 回実行 |
| 期待レスポンス | 1 回目: HTTP 201, 2 回目: HTTP 200（キャッシュ応答・INSERT なし）|
| DB 状態（後）| work_events の行数は 1 件のみ（二重 INSERT なし）|
| 対応 FR | FR-SY-004 |

### TST-intg-009: ハッシュチェーンの prev_hash 連鎖確認

| 項目 | 内容 |
|---|---|
| TST-ID | TST-intg-009 |
| HTTP | 同一 execution に対して `POST /api/v1/step-events` を 3 回実行 |
| 期待確認 | 3 件の hash_chain_blocks において: block[1].prev_hash = genesis_zeros, block[2].prev_hash = block[1].chain_hash, block[3].prev_hash = block[2].chain_hash |
| 対応 FR | FR-EV-001/002 |

### TST-intg-010: 入力値範囲外（ERR-VAL-002）

| 項目 | 内容 |
|---|---|
| TST-ID | TST-intg-010 |
| HTTP | `POST /api/v1/step-events` |
| リクエストボディ | step.range={0, 100} に対して value=150 |
| 期待レスポンス | HTTP 400, `{"error": "ERR-VAL-002", "field": "input.value", "constraint": "max:100"}` |
| 対応 FR | FR-NV-002 |

---

## 3. 証拠ファイルアップロード（TST-intg-011〜015）

### TST-intg-011: 写真アップロード成功

| 項目 | 内容 |
|---|---|
| TST-ID | TST-intg-011 |
| HTTP | `POST /api/v1/evidences`（multipart/form-data） |
| リクエスト | file=test.jpg（100KB）, exec_id=<uuid>, step_id=<uuid>, event_id=<uuidv7> |
| 期待レスポンス | HTTP 201, `{"file_id": "<uuid>", "file_hash": "<sha256_hex>"}` |
| DB 状態（後）| evidence_files +1, work_events +1（activity='evidence_attached'）|
| 対応 FR | FR-NV-007 |

### TST-intg-012: ファイルサイズ超過

| 項目 | 内容 |
|---|---|
| TST-ID | TST-intg-012 |
| HTTP | `POST /api/v1/evidences` |
| リクエスト | file=large.jpg（11MB, CFG 上限 10MB 超過）|
| 期待レスポンス | HTTP 413, `{"error": "ERR-VAL-001", "message": "File size exceeds 10MB limit"}` |
| 対応 FR | FR-NV-007 |

### TST-intg-013: サポート外ファイル形式

| 項目 | 内容 |
|---|---|
| TST-ID | TST-intg-013 |
| HTTP | `POST /api/v1/evidences` |
| リクエスト | file=script.js（application/javascript）|
| 期待レスポンス | HTTP 400, `{"error": "ERR-VAL-001", "message": "Unsupported file type"}` |
| 対応 FR | FR-NV-007 |

### TST-intg-014: 電子署名記録

| 項目 | 内容 |
|---|---|
| TST-ID | TST-intg-014 |
| HTTP | `POST /api/v1/sign-records` |
| リクエストボディ | `{"exec_id": "<uuid>", "step_id": "<uuid>", "sign_id": "<uuid>", "signer_id": "<uuid>"}` |
| 期待レスポンス | HTTP 201 |
| DB 状態（後）| work_events に activity='sign_applied' が記録される |
| 対応 FR | FR-NV-008 |

### TST-intg-015: アンドン発報

| 項目 | 内容 |
|---|---|
| TST-ID | TST-intg-015 |
| HTTP | `POST /api/v1/andon-alerts` |
| リクエストボディ | `{"exec_id": "<uuid>", "step_id": "<uuid>", "alert_type": "QUALITY_ISSUE"}` |
| 期待レスポンス | HTTP 201 |
| DB 状態（後）| andon_alerts に status='ALERTING' が記録・v_andon_active ビューに反映 |
| 対応 FR | FR-NV-009 |

---

## 4. SOP 承認ワークフロー（TST-intg-016〜020）

### TST-intg-016: SOP 下書き作成

| 項目 | 内容 |
|---|---|
| TST-ID | TST-intg-016 |
| HTTP | `POST /api/v1/sops` |
| リクエストボディ | `{"title": "組立工程 A", "operation_id": "<uuid>", "steps": [...]}`（SUPERVISOR role JWT）|
| 期待レスポンス | HTTP 201, `{"sop_id": "<uuid>", "status": "DRAFT", "version": "1.0.0"}` |
| 対応 FR | FR-MA-001, FR-MA-003 |

### TST-intg-017: SOP バージョンアップ

| 項目 | 内容 |
|---|---|
| TST-ID | TST-intg-017 |
| HTTP | `POST /api/v1/sops/{sop_id}/versions` |
| 前提条件 | sop_id が存在・status=PUBLISHED |
| 期待レスポンス | HTTP 201, `{"sop_id": "<uuid>", "version": "2.0.0", "status": "DRAFT"}` |
| DB 状態（後）| sop_version_history +1 |
| 対応 FR | FR-MA-002 |

### TST-intg-018: マスタ同期プル

| 項目 | 内容 |
|---|---|
| TST-ID | TST-intg-018 |
| HTTP | `GET /api/v1/sync/master?since=0` |
| 期待レスポンス | HTTP 200, 全マスタデータの差分 JSON |
| 対応 FR | FR-SY-003 |

### TST-intg-019: HMAC 署名付き Webhook 配信

| 項目 | 内容 |
|---|---|
| TST-ID | TST-intg-019 |
| 検証方法 | outbox_events から PENDING を選択し、モック親システムへ POST |
| 期待確認 | X-WNAV-Signature ヘッダが HMAC-SHA256(CFG.webhook_secret, payload) と一致 |
| 対応 FR | FR-SY-005 |

### TST-intg-020: 帳票生成トリガ（RP-001）

| 項目 | 内容 |
|---|---|
| TST-ID | TST-intg-020 |
| HTTP | `POST /api/v1/work-executions/{exec_id}/complete` |
| 前提条件 | 全ステップ完了状態の作業 |
| 期待レスポンス | HTTP 200, work_execution.status = 'COMPLETED' |
| DB 状態（後）| reports テーブルに RP-001 レコードが非同期で生成される（Eventual Consistency）|
| 対応 FR | FR-RP-001, FR-NV-002 |

---

**本節で確定した方針**
- **TST-intg-001〜020 の 20 統合テストケースを 4 領域（認証・ステップ記録・証拠・SOP ワークフロー）に分類し、HTTP Method/URL・リクエストボディ・期待レスポンス・テスト後 DB 状態を全件確定した。**
- **TST-intg-006/007/008 はステップ完了・スキップ違反・冪等性の 3 観点を独立したテストケースとして分離し、各ケースがアトミックなトランザクション動作を検証することを確定した。**
- **TST-intg-009 はハッシュチェーンの prev_hash 連鎖が 3 ブロック以上の連続記録で正しく形成されることを統合テストレベルで確認することを確定した。単体テストでは検証できないデータベース永続化後の連鎖整合性を確認する。**

---

## 参照業界分析

### 必須
- [`90_業界分析/21_電子記録の法規制とALCOA+.md`](../../90_業界分析/21_電子記録の法規制とALCOA+.md)

### 関連
- [`90_業界分析/06_品質管理とトレーサビリティ.md`](../../90_業界分析/06_品質管理とトレーサビリティ.md)
