// SSE 配信 API（API-sync-004）ハンドラ（06_SSE配信API仕様.md）
//
// GET /api/v1/sse/assignments — SSE 接続（text/event-stream）

use axum::{
    Extension,
    extract::State,
    response::{
        Sse,
        sse::{Event, KeepAlive},
    },
};
use chrono::Utc;
use futures::stream::{self, Stream};
use std::convert::Infallible;
use std::time::Duration;
use tokio_stream::StreamExt;
use uuid::Uuid;

use crate::{error::AppError, state::AppState};
use wnav_auth::CurrentUser;

/// GET /api/v1/sse/assignments — SSE 接続（API-sync-004）
///
/// 端末が長時間接続を確立し、作業割当イベントをリアルタイムに受信する。
/// 接続確立後、まず未配信の割当を初期配信し、以降は SSE ストリームで Push する。
/// keep_alive_sec=25 (CFG-029) ごとに keepalive イベントを送信する。
#[utoipa::path(
    get,
    path = "/api/v1/sse/assignments",
    operation_id = "sseAssignments",
    responses(
        (status = 200, description = "SSE ストリーム接続成功"),
        (status = 401, description = "認証エラー"),
        (status = 422, description = "terminal_id クレームなし"),
    ),
    security(("bearer_auth" = [])),
    tag = "sse",
)]
pub async fn sse_assignments(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, AppError> {
    // terminal_id は device_id クレームとして取得する
    let terminal_id = current_user.device_id.ok_or_else(|| {
        AppError::InvalidFormat(Some(vec![crate::error::Violation {
            field: "terminal_id".to_string(),
            message: "JWT に terminal_id クレームが存在しません。".to_string(),
        }]))
    })?;

    let keep_alive_sec = state.config.sse.keep_alive_sec;

    // 未配信の割当を初期取得する（接続確立時の初期配信）
    let initial_assignments = fetch_pending_assignments(&state, terminal_id).await;

    // 初期配信イベントを生成する
    let initial_events: Vec<Result<Event, Infallible>> = initial_assignments
        .into_iter()
        .map(|assignment| {
            let event_id = Uuid::now_v7().to_string();
            let data = serde_json::to_string(&assignment).unwrap_or_default();
            Ok(Event::default()
                .id(event_id)
                .event("assignment.created")
                .data(data))
        })
        .collect();

    // 定期 keepalive イベントストリームを生成する（CFG-029: 25 秒間隔）
    let keepalive_stream = tokio_stream::wrappers::IntervalStream::new(
        tokio::time::interval(Duration::from_secs(keep_alive_sec)),
    )
    .map(move |_| {
        let event_id = Uuid::now_v7().to_string();
        let data = serde_json::json!({
            "timestamp": Utc::now().to_rfc3339(),
        })
        .to_string();

        Ok::<Event, Infallible>(
            Event::default()
                .id(event_id)
                .event("keepalive")
                .data(data),
        )
    });

    // 初期配信 + keepalive を結合したストリームを返す
    let combined = stream::iter(initial_events).chain(keepalive_stream);

    tracing::info!(
        log_id = "LOG-SSE-001",
        terminal_id = %terminal_id,
        "SSE 接続を確立した"
    );

    Ok(Sse::new(combined).keep_alive(
        KeepAlive::new()
            .interval(Duration::from_secs(keep_alive_sec))
            .text("keepalive"),
    ))
}

/// 未配信の作業割当を取得する（初期配信用）
async fn fetch_pending_assignments(
    state: &AppState,
    terminal_id: Uuid,
) -> Vec<serde_json::Value> {
    let rows = sqlx::query_as::<_, (Uuid, Uuid, Option<Uuid>, Option<chrono::DateTime<Utc>>, i32, String, chrono::DateTime<Utc>)>(
        r"
        SELECT id, sop_id, lot_id, due_at, priority, status, received_at
        FROM work_assignments
        WHERE target_terminal_id = $1
          AND status IN ('pending', 'dispatched')
        ORDER BY received_at ASC
        LIMIT 100
        ",
    )
    .bind(terminal_id)
    .fetch_all(&state.read_pool)
    .await
    .unwrap_or_default();

    rows.into_iter()
        .map(|(id, sop_id, lot_id, due_at, priority, status, received_at)| {
            serde_json::json!({
                "assignment_id": id,
                "sop_id": sop_id,
                "lot_id": lot_id,
                "due_at": due_at,
                "priority": priority,
                "status": status,
                "received_at": received_at,
            })
        })
        .collect()
}
