# 11 wnav_work_assignment_webhook 詳細設計（MOD-BE-011 / MOD-BE-012）

> **配置**:
> - MOD-BE-011（work_assignment_receiver）は **`crates/wnav_master_api/`** に属するモジュールである。外部システムからの作業割当 Webhook を受信し、HMAC 検証・冪等性保証・DBへの記録・SSE キュー登録を担う。
> - MOD-BE-012（assignment_dispatcher）は **`crates/wnav_terminal_api/`** に属するモジュールである。端末向け SSE コネクション管理・割当配信・再接続時の pending 再送・keep-alive を担う。
> - BAT-014（sse_dispatch_retry_scheduler）は `wnav_terminal_api` 内の tokio task として動作し、delivery_status='sent' 未確認行を周期リトライする。

本章は MOD-BE-011 `wnav_webhook::work_assignment_receiver` と MOD-BE-012 `wnav_sse::assignment_dispatcher` の詳細設計を確定する。両モジュールは連携して FR-SY-013（外部システムからの作業割当受信）および FR-NV-014（端末への割当リアルタイム配信）を実現する。

---

## 1. モジュール概要

| 項目 | MOD-BE-011 | MOD-BE-012 |
|---|---|---|
| MOD-ID | MOD-BE-011 | MOD-BE-012 |
| 物理名 | work_assignment_receiver | assignment_dispatcher |
| ファイルパス（予定） | `crates/wnav_master_api/webhook/work_assignment.rs` | `crates/wnav_terminal_api/sse/assignment_dispatcher.rs` |
| 所属バイナリ | wnav_master_api（ポート 8081） | wnav_terminal_api（ポート 8080） |
| 関連 FR | FR-SY-013 | FR-NV-014 |
| 関連 API | API-webhook-001（受信エンドポイント） | API-sync-004（SSE エンドポイント） |
| 関連 TBL | TBL-027, TBL-035, TBL-052, TBL-053 | TBL-052, TBL-053 |
| 関連 BAT | — | BAT-014（sse_dispatch_retry_scheduler） |

---

## 2. MOD-BE-011: work_assignment_receiver

### 2-1. 構造体定義

```rust
// crates/wnav_master_api/webhook/work_assignment.rs

use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::broadcast;

/// 作業割当 Webhook 受信ハンドラの共有状態。
/// Axum の `Extension` または `State` として DI する。
pub struct WorkAssignmentReceiverState {
    pub db: Arc<PgPool>,
    /// SSE ディスパッチャへの通知チャネル（MSG-006）
    pub sse_tx: broadcast::Sender<AssignmentCreatedSignal>,
    pub config: WorkAssignmentReceiverConfig,
}

/// 受信ハンドラの設定
#[derive(Debug, Clone, serde::Deserialize)]
pub struct WorkAssignmentReceiverConfig {
    /// HMAC-SHA256 署名秘密鍵（hex エンコード 256bit 以上推奨）
    /// KEY-010 に対応する。設定キー: `webhook.signing_secret_hex`
    pub signing_secret_hex: String,
}

/// SSE ディスパッチャへ送信するシグナル（MSG-006）
#[derive(Debug, Clone)]
pub struct AssignmentCreatedSignal {
    pub assignment_id: uuid::Uuid,
    pub target_terminal_id: uuid::Uuid,
}
```

### 2-2. リクエスト / レスポンス型

```rust
// crates/wnav_master_api/webhook/work_assignment.rs（続き）

use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Webhook リクエストボディ（外部システムから受信）
#[derive(Debug, serde::Deserialize)]
pub struct WorkAssignmentRequest {
    /// 外部システムが管理する SOP キー（TBL-027 で UUID に解決）
    pub work_pattern_key: String,
    /// 外部システムが管理する端末キー（TBL-027 で UUID に解決）
    pub target_terminal_key: String,
    /// 外部システムが管理するロット ID（任意。TBL-027 で UUID に解決）
    pub lot_id_ext: Option<String>,
    /// 期限（RFC 3339）
    pub due_at: Option<DateTime<Utc>>,
    /// 優先度（1 = 最高）
    pub priority: i32,
    /// 推奨作業員外部キー（任意）
    pub suggested_worker_key: Option<String>,
}

/// Webhook 受信成功レスポンス（202 Accepted）
#[derive(Debug, serde::Serialize)]
pub struct WorkAssignmentAcceptedResponse {
    pub assignment_id: Uuid,
    pub status: String,   // "accepted"
}
```

### 2-3. 処理フロー

```rust
// crates/wnav_master_api/webhook/work_assignment.rs（続き）

use axum::{
    extract::{State, Json},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};

/// POST /api/v1/webhooks/work-assignments
/// Axum ハンドラ。WorkAssignmentReceiverState を State として受け取る。
pub async fn receive_work_assignment(
    State(state): State<Arc<WorkAssignmentReceiverState>>,
    headers: HeaderMap,
    body_bytes: axum::body::Bytes,
) -> impl IntoResponse {
    // ステップ 1: リクエスト受信（Axum フレームワークが担当）

    // ステップ 2: X-WNAV-Signature ヘッダ検証（KEY-010 の HMAC-SHA256）
    let signature_header = match headers.get("X-WNAV-Signature") {
        Some(v) => match v.to_str() {
            Ok(s) => s.to_string(),
            Err(_) => return (StatusCode::UNAUTHORIZED, "invalid signature header").into_response(),
        },
        None => return (StatusCode::UNAUTHORIZED, "missing X-WNAV-Signature").into_response(),
    };

    let secret = match hex::decode(&state.config.signing_secret_hex) {
        Ok(s) => s,
        Err(_) => {
            tracing::error!(
                log_id = "LOG-WH-010",
                event_name = "work_assignment_receiver.config_error",
                error = "signing_secret_hex is not valid hex",
            );
            return (StatusCode::INTERNAL_SERVER_ERROR, "config error").into_response();
        }
    };

    let expected_sig = signature_header
        .strip_prefix("sha256=")
        .unwrap_or(&signature_header);

    if !crate::sign::verify_signature(&secret, &body_bytes, expected_sig) {
        tracing::warn!(
            log_id = "LOG-WH-011",
            event_name = "work_assignment_receiver.signature_mismatch",
        );
        // ERR-BIZ-027: 署名検証失敗
        return (StatusCode::UNAUTHORIZED, "signature mismatch").into_response();
    }

    // ステップ 3: Idempotency-Key ヘッダ検証（TBL-035 で重複チェック）
    let idempotency_key = match headers.get("Idempotency-Key") {
        Some(v) => match v.to_str() {
            Ok(s) => s.to_string(),
            Err(_) => {
                return (StatusCode::BAD_REQUEST, "invalid Idempotency-Key").into_response()
            }
        },
        None => return (StatusCode::BAD_REQUEST, "missing Idempotency-Key").into_response(),
    };

    let is_duplicate = sqlx::query_scalar!(
        "SELECT EXISTS(SELECT 1 FROM idempotency_keys WHERE idempotency_key = $1)",
        idempotency_key
    )
    .fetch_one(state.db.as_ref())
    .await
    .unwrap_or(Some(false))
    .unwrap_or(false);

    if is_duplicate {
        // ERR-BIZ-028: 冪等性キー重複
        return (StatusCode::CONFLICT, "duplicate idempotency key").into_response();
    }

    // ステップ 4: リクエストボディを WorkAssignmentRequest に deserialize
    let request: WorkAssignmentRequest = match serde_json::from_slice(&body_bytes) {
        Ok(r) => r,
        Err(e) => {
            tracing::warn!(
                log_id = "LOG-WH-012",
                event_name = "work_assignment_receiver.deserialize_error",
                error = %e,
            );
            // ERR-VAL-032: リクエストボディ不正
            return (StatusCode::BAD_REQUEST, "invalid request body").into_response();
        }
    };

    // ステップ 5: external_key_binding（TBL-027）で外部キーを UUID に解決
    let resolution = resolve_external_keys(&state.db, &request).await;

    let (work_pattern_id, target_terminal_id, lot_id) = match resolution {
        Ok(ids) => ids,
        Err(unresolved_key) => {
            // 解決失敗: pending_resolution に記録して 422 を返す
            let _ = sqlx::query!(
                r#"
                INSERT INTO pending_resolutions (unresolved_key, raw_payload, created_at)
                VALUES ($1, $2, NOW())
                "#,
                unresolved_key,
                serde_json::to_value(&body_bytes.to_vec()).unwrap_or_default(),
            )
            .execute(state.db.as_ref())
            .await;

            tracing::warn!(
                log_id = "LOG-WH-013",
                event_name = "work_assignment_receiver.key_resolution_failed",
                unresolved_key = %unresolved_key,
            );
            return (StatusCode::UNPROCESSABLE_ENTITY, "external key resolution failed")
                .into_response();
        }
    };

    // ステップ 6 / 7 / 8: トランザクション内でアトミックに処理
    let assignment_id = uuid::Uuid::now_v7();

    let result = execute_assignment_transaction(
        &state.db,
        &state.sse_tx,
        assignment_id,
        work_pattern_id,
        target_terminal_id,
        lot_id,
        &request,
        &idempotency_key,
    )
    .await;

    match result {
        Ok(()) => {
            tracing::info!(
                log_id = "LOG-WH-014",
                event_name = "work_assignment_receiver.accepted",
                assignment_id = %assignment_id,
                target_terminal_id = %target_terminal_id,
            );
            // ステップ 9: 202 Accepted を返却
            (
                StatusCode::ACCEPTED,
                Json(WorkAssignmentAcceptedResponse {
                    assignment_id,
                    status: "accepted".to_string(),
                }),
            )
                .into_response()
        }
        Err(e) => {
            tracing::error!(
                log_id = "LOG-WH-015",
                event_name = "work_assignment_receiver.transaction_error",
                error = %e,
            );
            (StatusCode::INTERNAL_SERVER_ERROR, "transaction failed").into_response()
        }
    }
}
```

### 2-4. トランザクション実装（ステップ 6/7/8）

```rust
// crates/wnav_master_api/webhook/work_assignment.rs（続き）

use tokio::sync::broadcast;

/// ステップ 6/7/8 をアトミックに実行する sqlx トランザクション。
///
/// - ステップ 6: work_assignments（TBL-052）に INSERT（status='pending'）
/// - ステップ 7: sse_dispatch_log（TBL-053）に INSERT（delivery_status='queued'）
/// - ステップ 8: 内部チャネル（MSG-006）に assignment_created を送信
async fn execute_assignment_transaction(
    db: &PgPool,
    sse_tx: &broadcast::Sender<AssignmentCreatedSignal>,
    assignment_id: uuid::Uuid,
    work_pattern_id: uuid::Uuid,
    target_terminal_id: uuid::Uuid,
    lot_id: Option<uuid::Uuid>,
    request: &WorkAssignmentRequest,
    idempotency_key: &str,
) -> Result<(), sqlx::Error> {
    let mut tx = db.begin().await?;

    // ステップ 6: work_assignments INSERT（status='pending'）
    sqlx::query!(
        r#"
        INSERT INTO work_assignments (
            assignment_id, work_pattern_id, target_terminal_id, lot_id,
            due_at, priority, suggested_worker_key,
            status, received_at, idempotency_key
        ) VALUES (
            $1, $2, $3, $4,
            $5, $6, $7,
            'pending', NOW(), $8
        )
        "#,
        assignment_id,
        work_pattern_id,
        target_terminal_id,
        lot_id,
        request.due_at,
        request.priority,
        request.suggested_worker_key.as_deref(),
        idempotency_key,
    )
    .execute(&mut *tx)
    .await?;

    // ステップ 7: sse_dispatch_log INSERT（delivery_status='queued'）
    sqlx::query!(
        r#"
        INSERT INTO sse_dispatch_log (
            log_id, assignment_id, terminal_id,
            delivery_status, queued_at, retry_count
        ) VALUES (
            $1, $2, $3,
            'queued', NOW(), 0
        )
        "#,
        uuid::Uuid::now_v7(),
        assignment_id,
        target_terminal_id,
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    // ステップ 8: 内部チャネル（MSG-006）にシグナル送信（SSE dispatcher に通知）
    // トランザクション commit 後に送信して二重送信リスクを回避する
    let _ = sse_tx.send(AssignmentCreatedSignal {
        assignment_id,
        target_terminal_id,
    });

    Ok(())
}
```

### 2-5. 外部キー解決（TBL-027）

```rust
// crates/wnav_master_api/webhook/work_assignment.rs（続き）

/// external_key_bindings（TBL-027）を使い、外部キーを UUID に変換する。
///
/// 戻り値: Ok((work_pattern_id, target_terminal_id, lot_id)) | Err(unresolved_key)
async fn resolve_external_keys(
    db: &PgPool,
    request: &WorkAssignmentRequest,
) -> Result<(uuid::Uuid, uuid::Uuid, Option<uuid::Uuid>), String> {
    // work_pattern_key の解決
    let work_pattern_id: Option<uuid::Uuid> = sqlx::query_scalar!(
        "SELECT internal_id FROM external_key_bindings WHERE external_key = $1 AND entity_type = 'work_pattern'",
        request.work_pattern_key
    )
    .fetch_optional(db)
    .await
    .map_err(|e| e.to_string())?;

    let work_pattern_id = work_pattern_id
        .ok_or_else(|| request.work_pattern_key.clone())?;

    // target_terminal_key の解決
    let target_terminal_id: Option<uuid::Uuid> = sqlx::query_scalar!(
        "SELECT internal_id FROM external_key_bindings WHERE external_key = $1 AND entity_type = 'terminal'",
        request.target_terminal_key
    )
    .fetch_optional(db)
    .await
    .map_err(|e| e.to_string())?;

    let target_terminal_id = target_terminal_id
        .ok_or_else(|| request.target_terminal_key.clone())?;

    // lot_id_ext の解決（任意）
    let lot_id = if let Some(ref lot_ext) = request.lot_id_ext {
        let id: Option<uuid::Uuid> = sqlx::query_scalar!(
            "SELECT internal_id FROM external_key_bindings WHERE external_key = $1 AND entity_type = 'lot'",
            lot_ext
        )
        .fetch_optional(db)
        .await
        .map_err(|e| e.to_string())?;

        Some(id.ok_or_else(|| lot_ext.clone())?)
    } else {
        None
    };

    Ok((work_pattern_id, target_terminal_id, lot_id))
}
```

### 2-6. エラーハンドリング仕様

| エラーコード | 発生条件 | HTTP ステータス | 対応方針 |
|---|---|---|---|
| ERR-VAL-032 | リクエストボディ不正（deserialize 失敗） | 400 Bad Request | エラーメッセージを返す。ログ LOG-WH-012 に記録 |
| ERR-BIZ-027 | 署名検証失敗（X-WNAV-Signature 不一致） | 401 Unauthorized | ログ LOG-WH-011 に記録。詳細は返さない（セキュリティ） |
| ERR-BIZ-028 | Idempotency-Key 重複（TBL-035 で検出） | 409 Conflict | 既存の結果を示すレスポンスを返す（冪等性保証） |
| — | external_key_binding 解決失敗 | 422 Unprocessable Entity | pending_resolution に記録してログ LOG-WH-013 に記録 |

### 2-7. テスト要件

| テスト種別 | 内容 | 対応識別子 |
|---|---|---|
| 単体テスト | HMAC-SHA256 署名生成と検証の正常系・異常系（正しいシークレット・誤ったシークレット・改ざんボディ） | TST-unit-BE-031〜033（予定） |
| 統合テスト | Idempotency-Key 重複時の 409 レスポンスと TBL-035 への記録確認 | TST-intg-024（予定） |
| 統合テスト | 正常受信時の TBL-052/TBL-053 INSERT とトランザクション整合性確認 | TST-intg-025（予定） |

---

## 3. MOD-BE-012: assignment_dispatcher

### 3-1. 構造体定義

```rust
// crates/wnav_terminal_api/sse/assignment_dispatcher.rs

use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::broadcast;
use uuid::Uuid;

/// SSE ディスパッチャの共有状態。
/// `wnav_terminal_api` の `main.rs` で生成し、Axum の State として DI する。
pub struct AssignmentDispatcherState {
    pub db: Arc<PgPool>,
    /// 割当作成シグナルを全 SSE セッションにブロードキャストするチャネル（MSG-006）
    pub signal_tx: broadcast::Sender<AssignmentCreatedSignal>,
    pub config: AssignmentDispatcherConfig,
}

/// SSE ディスパッチャの設定
#[derive(Debug, Clone, serde::Deserialize)]
pub struct AssignmentDispatcherConfig {
    /// keep-alive コメント送信間隔（秒）。CFG-029 に対応する。
    pub keepalive_interval_secs: u64,
    /// リトライ最大回数。CFG-030 に対応する。
    pub max_retry_count: i32,
    /// BAT-014 リトライスケジューラのポーリング間隔（秒）
    pub retry_scheduler_interval_secs: u64,
}

impl Default for AssignmentDispatcherConfig {
    fn default() -> Self {
        Self {
            keepalive_interval_secs: 30,
            max_retry_count: 5,
            retry_scheduler_interval_secs: 60,
        }
    }
}

/// 各 SSE セッションが保持する端末情報
#[derive(Debug, Clone)]
pub struct SseSessionContext {
    pub terminal_id: Uuid,
    /// `Last-Event-ID` ヘッダの値（再接続時に設定）
    pub last_event_id: Option<String>,
}
```

### 3-2. SSE ハンドラ

```rust
// crates/wnav_terminal_api/sse/assignment_dispatcher.rs（続き）

use axum::{
    extract::{Query, State},
    http::HeaderMap,
    response::sse::{Event, KeepAlive, Sse},
};
use futures_util::stream::{self, StreamExt};
use std::convert::Infallible;
use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;

/// GET /api/v1/sse/assignments?terminal_id={uuid}
///
/// アーキテクチャ:
/// - `tokio::sync::broadcast` チャネルで受信シグナルを全 SSE セッションに伝達
/// - 接続時: TBL-052 から pending/dispatched 行を received_at ASC で全件配信
/// - `Last-Event-ID` ヘッダが存在する場合: UUID v7 タイムスタンプ比較で差分配信
/// - keep-alive: CFG-029 秒ごとにコメント行送信
pub async fn sse_assignments_handler(
    State(state): State<Arc<AssignmentDispatcherState>>,
    headers: HeaderMap,
    Query(params): Query<SseQueryParams>,
) -> Sse<impl futures_util::Stream<Item = Result<Event, Infallible>>> {
    let terminal_id = params.terminal_id;
    let last_event_id = headers
        .get("Last-Event-ID")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let db = state.db.clone();
    let signal_rx = state.signal_tx.subscribe();
    let keepalive_secs = state.config.keepalive_interval_secs;

    // 接続時の初期配信: TBL-052 から pending/dispatched 行を取得
    let initial_events = fetch_pending_assignments(&db, terminal_id, last_event_id.as_deref())
        .await
        .unwrap_or_default();

    let initial_stream = stream::iter(initial_events.into_iter().map(Ok));

    // ブロードキャスト受信ストリーム（シグナル受信 → DB から該当行を取得して配信）
    let signal_stream = BroadcastStream::new(signal_rx)
        .filter_map(move |sig_result| {
            let db = db.clone();
            async move {
                match sig_result {
                    Ok(sig) if sig.target_terminal_id == terminal_id => {
                        build_sse_event_for_assignment(&db, sig.assignment_id).await
                    }
                    _ => None,
                }
            }
        })
        .map(Ok);

    let combined = initial_stream.chain(signal_stream);

    Sse::new(combined).keep_alive(
        KeepAlive::new()
            .interval(std::time::Duration::from_secs(keepalive_secs))
            .text("keep-alive"),
    )
}

#[derive(Debug, serde::Deserialize)]
pub struct SseQueryParams {
    pub terminal_id: Uuid,
}
```

### 3-3. 初期配信クエリ

```rust
// crates/wnav_terminal_api/sse/assignment_dispatcher.rs（続き）

use axum::response::sse::Event;

/// 接続時・再接続時に配信する pending/dispatched 割当を取得する。
///
/// `last_event_id` が存在する場合は UUID v7 のタイムスタンプ部分を比較し、
/// その UUID 以降に作成された行のみを返す（差分配信）。
async fn fetch_pending_assignments(
    db: &PgPool,
    terminal_id: Uuid,
    last_event_id: Option<&str>,
) -> Result<Vec<Event>, sqlx::Error> {
    // last_event_id が存在する場合は UUID v7 タイムスタンプで絞り込む
    // UUID v7 は時系列ソート可能であるため、文字列比較で「以降」を判定できる
    let rows = match last_event_id {
        Some(since_id) => {
            sqlx::query!(
                r#"
                SELECT assignment_id, work_pattern_id, lot_id, due_at, priority,
                       status, suggested_worker_key, received_at
                FROM work_assignments
                WHERE target_terminal_id = $1
                  AND status IN ('pending', 'dispatched')
                  AND assignment_id > $2::uuid
                ORDER BY received_at ASC
                "#,
                terminal_id,
                since_id,
            )
            .fetch_all(db)
            .await?
        }
        None => {
            sqlx::query!(
                r#"
                SELECT assignment_id, work_pattern_id, lot_id, due_at, priority,
                       status, suggested_worker_key, received_at
                FROM work_assignments
                WHERE target_terminal_id = $1
                  AND status IN ('pending', 'dispatched')
                ORDER BY received_at ASC
                "#,
                terminal_id,
            )
            .fetch_all(db)
            .await?
        }
    };

    let events = rows
        .into_iter()
        .map(|row| {
            let data = serde_json::json!({
                "assignment_id": row.assignment_id,
                "work_pattern_id": row.work_pattern_id,
                "lot_id": row.lot_id,
                "due_at": row.due_at,
                "priority": row.priority,
                "status": row.status,
                "suggested_worker_key": row.suggested_worker_key,
            });
            Event::default()
                .id(row.assignment_id.to_string())
                .event("assignment.created")
                .data(data.to_string())
        })
        .collect();

    Ok(events)
}

/// シグナル受信時に単一割当の SSE イベントを構築する。
async fn build_sse_event_for_assignment(
    db: &PgPool,
    assignment_id: Uuid,
) -> Option<Event> {
    let row = sqlx::query!(
        r#"
        SELECT assignment_id, work_pattern_id, lot_id, due_at, priority,
               status, suggested_worker_key
        FROM work_assignments
        WHERE assignment_id = $1
        "#,
        assignment_id,
    )
    .fetch_optional(db)
    .await
    .ok()??;

    let data = serde_json::json!({
        "assignment_id": row.assignment_id,
        "work_pattern_id": row.work_pattern_id,
        "lot_id": row.lot_id,
        "due_at": row.due_at,
        "priority": row.priority,
        "status": row.status,
        "suggested_worker_key": row.suggested_worker_key,
    });

    Some(
        Event::default()
            .id(assignment_id.to_string())
            .event("assignment.created")
            .data(data.to_string()),
    )
}
```

### 3-4. BAT-014: sse_dispatch_retry_scheduler

```rust
// crates/wnav_terminal_api/sse/assignment_dispatcher.rs（続き）

use tokio::time::{interval, Duration};

/// BAT-014: delivery_status='sent' 未確認行を周期的にリトライするスケジューラ。
///
/// 連携設計:
/// - 1 分周期（CFG-030 / config.retry_scheduler_interval_secs）でポーリング
/// - retry_count <= CFG-030（config.max_retry_count）の行を対象
/// - sse_dispatch_log（TBL-053）の delivery_status を確認し、
///   'sent' 状態で ack が来ていない行を再度 'queued' に戻してシグナル送信
/// - 'ack' 状態の行は操作しない
/// - retry_count > max_retry_count の行は 'failed' に更新してアラート
///
/// `wnav_terminal_api` の `main.rs` で
/// `tokio::spawn(run_retry_scheduler(state))` として起動する。
pub async fn run_retry_scheduler(state: Arc<AssignmentDispatcherState>) {
    tracing::info!(
        log_id = "LOG-BAT-014",
        event_name = "sse_retry_scheduler.started",
    );

    let mut ticker = interval(Duration::from_secs(
        state.config.retry_scheduler_interval_secs,
    ));

    loop {
        ticker.tick().await;

        match retry_unacknowledged_dispatches(&state).await {
            Ok(count) => {
                tracing::info!(
                    log_id = "LOG-BAT-014",
                    event_name = "sse_retry_scheduler.cycle",
                    retried_count = count,
                );
            }
            Err(e) => {
                tracing::error!(
                    log_id = "LOG-BAT-014",
                    event_name = "sse_retry_scheduler.error",
                    error = %e,
                );
            }
        }
    }
}

/// sent 状態で未確認の配信ログを再キューするコア処理。
async fn retry_unacknowledged_dispatches(
    state: &AssignmentDispatcherState,
) -> Result<u64, sqlx::Error> {
    let rows = sqlx::query!(
        r#"
        SELECT log_id, assignment_id, terminal_id, retry_count
        FROM sse_dispatch_log
        WHERE delivery_status = 'sent'
          AND retry_count <= $1
          AND sent_at < NOW() - INTERVAL '60 seconds'
        FOR UPDATE SKIP LOCKED
        LIMIT 100
        "#,
        state.config.max_retry_count,
    )
    .fetch_all(state.db.as_ref())
    .await?;

    let count = rows.len() as u64;

    for row in rows {
        // retry_count を +1 して 'queued' に戻す
        sqlx::query!(
            r#"
            UPDATE sse_dispatch_log
            SET delivery_status = 'queued',
                retry_count = $2
            WHERE log_id = $1
            "#,
            row.log_id,
            row.retry_count + 1,
        )
        .execute(state.db.as_ref())
        .await?;

        // SSE シグナルを再送信
        let _ = state.signal_tx.send(AssignmentCreatedSignal {
            assignment_id: row.assignment_id,
            target_terminal_id: row.terminal_id,
        });
    }

    // retry_count > max_retry_count の行を 'failed' に移行
    sqlx::query!(
        r#"
        UPDATE sse_dispatch_log
        SET delivery_status = 'failed'
        WHERE delivery_status = 'sent'
          AND retry_count > $1
        "#,
        state.config.max_retry_count,
    )
    .execute(state.db.as_ref())
    .await?;

    Ok(count)
}
```

### 3-5. delivery_status 状態遷移

| 遷移 | 条件 |
|---|---|
| queued → sent | SSE セッションが event を送信した時点（fire-and-forget）|
| sent → ack | ハンディ端末が `POST /api/v1/sse/assignments/{log_id}/ack` を送信した時点 |
| sent → queued | BAT-014 が ack 未確認を検出してリトライキューに戻した時点 |
| sent → failed | retry_count > CFG-030 に達した時点 |

---

## 4. 関連テーブル参照

| TBL-ID | テーブル名 | 用途 |
|---|---|---|
| TBL-027 | external_key_bindings | 外部キー → 内部 UUID 変換（work_pattern / terminal / lot） |
| TBL-035 | idempotency_keys | Webhook 冪等性保証 |
| TBL-052 | work_assignments | 割当本体（status: pending / dispatched / in_progress / completed / cancelled） |
| TBL-053 | sse_dispatch_log | SSE 配信ログ（delivery_status: queued / sent / ack / failed） |

---

## 5. MSG・CFG・KEY 参照

| 識別子 | 内容 |
|---|---|
| MSG-006 | `broadcast::Sender<AssignmentCreatedSignal>` — work_assignment_receiver から assignment_dispatcher へのシグナル |
| CFG-029 | SSE keep-alive 間隔（秒、デフォルト 30） |
| CFG-030 | SSE リトライ最大回数（デフォルト 5） |
| KEY-010 | Webhook HMAC-SHA256 署名秘密鍵（256bit 以上推奨、hex エンコード） |
| BAT-014 | sse_dispatch_retry_scheduler — delivery_status='sent' 未確認行を 1 分周期でリトライ |
| API-sync-004 | `GET /api/v1/sse/assignments` — SSE エンドポイント |
| API-sync-005 | `GET /api/v1/work-assignments` — Pull モード用ポーリングエンドポイント |

---

**本節で確定した方針**
- **MOD-BE-011（work_assignment_receiver）は `wnav_master_api`（ポート 8081）に属し、HMAC-SHA256 検証・冪等性チェック・TBL-052/TBL-053 への INSERT を単一 sqlx トランザクションでアトミックに実行する設計を確定した。**
- **external_key_binding 解決失敗時は pending_resolution に記録して 422 を返し、外部システムが再試行できる設計を確定した。**
- **MOD-BE-012（assignment_dispatcher）は `wnav_terminal_api`（ポート 8080）に属し、`tokio::sync::broadcast` チャネルで全 SSE セッションにシグナルを伝達する設計を確定した。**
- **再接続時は `Last-Event-ID` ヘッダの UUID v7 タイムスタンプで差分配信し、漏れ受信を防ぐ設計を確定した。**
- **BAT-014（sse_dispatch_retry_scheduler）は `SELECT ... FOR UPDATE SKIP LOCKED` で競合なく処理し、`wnav_terminal_api` の `main.rs` で `tokio::spawn` して起動することを確定した。**

---

## 参照業界分析

### 必須
- [`90_業界分析/09_セキュリティとアクセス制御.md`](../../90_業界分析/09_セキュリティとアクセス制御.md)

### 関連
- [`90_業界分析/06_品質管理とトレーサビリティ.md`](../../90_業界分析/06_品質管理とトレーサビリティ.md)
- [`90_業界分析/17_サプライチェーンと作業依存性.md`](../../90_業界分析/17_サプライチェーンと作業依存性.md)
