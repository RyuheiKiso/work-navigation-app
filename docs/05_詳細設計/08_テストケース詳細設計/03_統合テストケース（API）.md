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
- **TST-intg-021〜030（IQC/リワーク 10 件）を追加し、後工程ハードゲート・AQL 判定・Two-Person Integrity の技術的保証を統合テストレベルで確認することを確定した。**

---

## 5. IQC API（TST-intg-021〜025）

### TST-intg-021: 入荷ロット受入登録（ゴールデンパス）

| 項目 | 内容 |
|---|---|
| TST-ID | TST-intg-021 |
| HTTP | `POST /api/v1/iqc/incoming-inspections` |
| リクエストボディ | `{"lot_id":"<uuid>","supplier_id":"<uuid>","material_id":"<uuid>","lot_quantity":1000}` |
| 前提条件 | lot・supplier・material・sampling_plan が存在 |
| 期待レスポンス | HTTP 201, `{"inspection_id":"<uuid>","sample_size_n":80,"accept_number_ac":3,"reject_number_re":4}` |
| DB 状態（後）| incoming_inspections に 1 レコード（qc_status=PENDING）|
| 対応 FR | FR-IQ-001, BR-BUS-032 |

### TST-intg-022: 後工程ハードゲート（ERR-BIZ-015）

| 項目 | 内容 |
|---|---|
| TST-ID | TST-intg-022 |
| HTTP | `POST /api/v1/work-executions/{id}/events`（材料 QR スキャンイベント）|
| 前提条件 | lot_qc_states.qc_status = 'REJECTED' |
| 期待レスポンス | HTTP 409, `{"error":"ERR-BIZ-015","title":"lot_not_qc_passed"}` |
| DB 状態（後）| work_events に INSERT なし |
| 対応 FR | FR-IQ-009, BR-BUS-036 |

### TST-intg-023: AQL 判定（PASSED → lot_qc_states 更新）

| 項目 | 内容 |
|---|---|
| TST-ID | TST-intg-023 |
| HTTP | `POST /api/v1/iqc/incoming-inspections/{id}/judge` |
| 前提条件 | n=5 のサンプルを全部 defect_flag=false で登録済み（不良数=0 ≤ Ac=1）|
| 期待レスポンス | HTTP 200, `{"qc_status":"PASSED"}` |
| DB 状態（後）| lot_qc_states.qc_status = 'PASSED'、incoming_inspections.judged_at が設定される |
| 対応 FR | FR-IQ-007, FR-IQ-008 |

### TST-intg-024: 特採承認（CONDITIONAL_PASS）

| 項目 | 内容 |
|---|---|
| TST-ID | TST-intg-024 |
| HTTP | `POST /api/v1/iqc/incoming-inspections/{id}/concession` |
| 前提条件 | qc_status = 'REJECTED'、quality_admin ロールの電子サイン存在 |
| 期待レスポンス | HTTP 201, `{"approval_id":"<uuid>","valid_until":"2026-08-31"}` |
| DB 状態（後）| concession_approvals に Append-only 1 レコード、lot_qc_states.qc_status = 'CONDITIONAL_PASS' |
| 対応 FR | FR-IQ-010, BR-BUS-037 |

### TST-intg-025: 測定数不足（ERR-VAL-030）

| 項目 | 内容 |
|---|---|
| TST-ID | TST-intg-025 |
| HTTP | `POST /api/v1/iqc/incoming-inspections/{id}/judge` |
| 前提条件 | sample_size_n=5 だが measurements が 3 件のみ |
| 期待レスポンス | HTTP 422, `{"error":"ERR-VAL-030","title":"measurement_count_below_n"}` |
| 対応 FR | FR-IQ-007 |

---

## 6. リワーク API（TST-intg-026〜030）

### TST-intg-026: ディスポジション登録（ゴールデンパス）

| 項目 | 内容 |
|---|---|
| TST-ID | TST-intg-026 |
| HTTP | `POST /api/v1/dispositions` |
| リクエストボディ | `{"nonconformity_id":"<nc>","decision":"REWORK","decision_reason":"修正可能","quality_admin_sign_id":"<qa>","supervisor_sign_id":"<sup>"}` |
| 前提条件 | 2 つの異なる worker_id の電子サイン存在 |
| 期待レスポンス | HTTP 201, `{"disposition_id":"<uuid>","decided_at":"<timestamp>"}` |
| DB 状態（後）| dispositions に Append-only 1 レコード |
| 対応 FR | FR-ST-013, BR-BUS-040 |

### TST-intg-027: Two-Person Integrity 違反（ERR-BIZ-021）

| 項目 | 内容 |
|---|---|
| TST-ID | TST-intg-027 |
| HTTP | `POST /api/v1/dispositions` |
| 前提条件 | quality_admin_sign_id と supervisor_sign_id が同一 worker_id |
| 期待レスポンス | HTTP 422, `{"error":"ERR-BIZ-021","title":"disposition_same_signer"}` |
| DB 状態（後）| dispositions に INSERT なし（DB トリガによる拒否）|
| 対応 NFR | NFR-SEC-048 |

```rust
#[sqlx::test]
async fn test_tst_intg_027_two_person_integrity(pool: PgPool) {
    let same_worker_qa_sign = create_electronic_sign(&pool, "worker_001", "quality_admin").await;
    let same_worker_sup_sign = create_electronic_sign(&pool, "worker_001", "supervisor").await;
    // 同一 worker_id で 2 つの電子サインを作成

    let payload = DispositionPayload {
        nonconformity_id: create_test_nonconformity(&pool).await,
        decision: "REWORK".to_string(),
        decision_reason: "テスト判定".to_string(),
        quality_admin_sign_id: same_worker_qa_sign,
        supervisor_sign_id: same_worker_sup_sign,
    };

    let response = app.post("/api/v1/dispositions").json(&payload).await;
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);

    let error: ProblemDetails = response.json().await;
    assert_eq!(error.error_id, "ERR-BIZ-021");

    // DB に INSERT されていないことを確認
    let count: i64 = sqlx::query_scalar!("SELECT COUNT(*) FROM dispositions")
        .fetch_one(&pool).await.unwrap().unwrap();
    assert_eq!(count, 0);
}
```

### TST-intg-028: リワーク上限超過（ERR-BIZ-022）

| 項目 | 内容 |
|---|---|
| TST-ID | TST-intg-028 |
| HTTP | `POST /api/v1/reworks` |
| 前提条件 | 同一 parent_lot_id に CFG-026（デフォルト 3）件のリワーク完了済み |
| 期待レスポンス | HTTP 409, `{"error":"ERR-BIZ-022","title":"rework_max_count_exceeded"}` |
| 対応 FR | FR-ST-014 |

### TST-intg-029: 再検査者同一（ERR-BIZ-023）

| 項目 | 内容 |
|---|---|
| TST-ID | TST-intg-029 |
| HTTP | `POST /api/v1/rework-verifications` |
| 前提条件 | リワーク実施者（rework_case_id の worker_id）と同一ユーザーが再検査を試みる |
| 期待レスポンス | HTTP 422, `{"error":"ERR-BIZ-023","title":"rework_verifier_same_as_worker"}` |
| 対応 NFR | NFR-SEC-048 |

### TST-intg-030: 廃却記録（Append-only）

| 項目 | 内容 |
|---|---|
| TST-ID | TST-intg-030 |
| HTTP | `POST /api/v1/scrap-records` |
| 前提条件 | rework.status = 'DISPOSITION_DECIDED' かつ decision = 'SCRAP' |
| 期待レスポンス | HTTP 201, `{"rework_id":"<uuid>","recorded_at":"<timestamp>"}` |
| DB 状態（後）| scrap_records に Append-only 1 レコード、reworks.status = 'CLOSED_SCRAP' に更新 |
| 対応 FR | FR-MA-017, BR-BUS-043 |

---

## 参照業界分析

### 必須
- [`90_業界分析/21_電子記録の法規制とALCOA+.md`](../../90_業界分析/21_電子記録の法規制とALCOA+.md)

### 関連
- [`90_業界分析/06_品質管理とトレーサビリティ.md`](../../90_業界分析/06_品質管理とトレーサビリティ.md)
