// 作業指示 Pull 補完 API（API-sync-005）ハンドラ（07_作業指示Pull補完API仕様.md）
//
// GET  /api/v1/work-assignments           — 作業割当一覧（カーソルページング）
// POST /api/v1/work-assignments/{id}/ack  — 割当確認応答（ACK）
//
// SQLX_OFFLINE=true 環境のため sqlx::query() を使用する。cargo sqlx prepare 後に sqlx::query! に切り替えること。

use axum::{
    Extension, Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use chrono::Utc;
use uuid::Uuid;

use crate::{
    dto::{
        response_envelope::ApiResponse,
        work_assignments::{
            WorkAssignmentAckData, WorkAssignmentAckRequest, WorkAssignmentDto, WorkAssignmentQuery,
        },
    },
    error::AppError,
    state::AppState,
};
use wnav_auth::CurrentUser;

/// GET /api/v1/work-assignments — 作業割当一覧取得（API-sync-005）
///
/// JWT の terminal_id クレームで自端末の割当のみを返す（他端末の割当は返さない）。
/// SSE 接続不可時のフォールバック・アプリ起動時の初期取得に使用する。
#[utoipa::path(
    get,
    path = "/api/v1/work-assignments",
    operation_id = "listWorkAssignments",
    params(
        ("status" = Option<String>, Query, description = "ステータスフィルタ（デフォルト: pending,dispatched）"),
        ("limit" = Option<i64>, Query, description = "最大取得件数（1〜200、デフォルト 50）"),
        ("after" = Option<Uuid>, Query, description = "カーソルページング用 UUID v7"),
    ),
    responses(
        (status = 200, description = "作業割当一覧"),
        (status = 401, description = "認証エラー"),
        (status = 422, description = "terminal_id クレームなし"),
    ),
    security(("bearer_auth" = [])),
    tag = "work-assignments",
)]
pub async fn list_work_assignments(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(query): Query<WorkAssignmentQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    // terminal_id は device_id クレームとして取得する
    let terminal_id = current_user.device_id.ok_or_else(|| {
        AppError::InvalidFormat(Some(vec![crate::error::Violation {
            field: "terminal_id".to_string(),
            message: "JWT に terminal_id クレームが存在しません。".to_string(),
        }]))
    })?;

    let limit = query.limit.unwrap_or(50).clamp(1, 200);
    let status_filter = query
        .status
        .unwrap_or_else(|| "pending,dispatched".to_string());

    let status_array: Vec<String> = status_filter
        .split(',')
        .map(|s| s.trim().to_string())
        .collect();

    // カーソルページング（limit + 1 件取得して has_more を判定する）
    let rows = sqlx::query_as::<
        _,
        (
            Uuid,
            Uuid,
            String,
            Option<Uuid>,
            Option<String>,
            Option<Uuid>,
            Option<Uuid>,
            Option<chrono::DateTime<Utc>>,
            i32,
            String,
            chrono::DateTime<Utc>,
        ),
    >(
        r"
        SELECT
            wa.id,
            wa.sop_id,
            COALESCE(s.name_json ->> 'ja', '') AS sop_name,
            wa.lot_id,
            l.lot_number,
            wa.suggested_worker_id,
            wa.suggested_equipment_id,
            wa.due_at,
            wa.priority,
            wa.status,
            wa.received_at
        FROM work_assignments wa
        LEFT JOIN sops s ON s.id = wa.sop_id
        LEFT JOIN lots l ON l.id = wa.lot_id
        WHERE wa.target_terminal_id = $1
          AND wa.status = ANY($2)
          AND ($3::uuid IS NULL OR wa.received_at > (
                SELECT received_at FROM work_assignments WHERE id = $3
              ))
        ORDER BY wa.received_at ASC
        LIMIT $4
        ",
    )
    .bind(terminal_id)
    .bind(&status_array)
    .bind(query.after)
    .bind(limit + 1)
    .fetch_all(&state.read_pool)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "work_assignments 取得に失敗した");
        AppError::DatabaseError
    })?;

    let has_more = rows.len() as i64 > limit;
    let items: Vec<WorkAssignmentDto> = rows
        .into_iter()
        .take(limit as usize)
        .map(
            |(
                id,
                sop_id,
                sop_name,
                lot_id,
                lot_number,
                suggested_worker_id,
                suggested_equipment_id,
                due_at,
                priority,
                status,
                received_at,
            )| {
                WorkAssignmentDto {
                    id,
                    sop_id,
                    sop_name,
                    lot_id,
                    lot_number,
                    suggested_worker_id,
                    suggested_equipment_id,
                    due_at,
                    priority,
                    status,
                    received_at,
                }
            },
        )
        .collect();

    let next_cursor = if has_more {
        items.last().map(|item| item.id)
    } else {
        None
    };

    let request_id = Uuid::now_v7();
    let response = serde_json::json!({
        "data": items,
        "meta": {
            "request_id": request_id,
            "server_time": Utc::now(),
            "api_version": "v1",
            "limit": limit,
            "has_more": has_more,
            "next_cursor": next_cursor,
        }
    });

    Ok(Json(response))
}

/// POST /api/v1/work-assignments/{id}/ack — 割当確認応答（ACK）
///
/// 端末が割当を受信・確認したことをサーバーに通知する。
/// TBL-053 の対応レコードを acknowledged 状態に更新する。
#[utoipa::path(
    post,
    path = "/api/v1/work-assignments/{id}/ack",
    operation_id = "ackWorkAssignment",
    params(
        ("id" = Uuid, Path, description = "作業割当 ID"),
    ),
    request_body = WorkAssignmentAckRequest,
    responses(
        (status = 200, description = "ACK 成功", body = ApiResponse<WorkAssignmentAckData>),
        (status = 404, description = "割当が見つからない"),
    ),
    security(("bearer_auth" = [])),
    tag = "work-assignments",
)]
pub async fn ack_work_assignment(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(_body): Json<WorkAssignmentAckRequest>,
) -> Result<(StatusCode, Json<ApiResponse<WorkAssignmentAckData>>), AppError> {
    let server_received_at = Utc::now();

    let terminal_id = current_user.device_id.ok_or(AppError::Unauthorized)?;

    // work_assignments の target_terminal_id と JWT.terminal_id が一致することを確認する
    let rows_affected = sqlx::query(
        r"
        UPDATE work_assignments
        SET status = 'acknowledged', acknowledged_at = $3
        WHERE id = $1 AND target_terminal_id = $2 AND status != 'acknowledged'
        ",
    )
    .bind(id)
    .bind(terminal_id)
    .bind(server_received_at)
    .execute(&state.event_insert_pool)
    .await
    .map_err(|_| AppError::DatabaseError)?
    .rows_affected();

    if rows_affected == 0 {
        return Err(AppError::NotFound);
    }

    // sse_dispatch_log を acknowledged に更新する
    let _ = sqlx::query(
        r"
        UPDATE sse_dispatch_log
        SET status = 'acknowledged', acknowledged_at = $2
        WHERE assignment_id = $1 AND target_terminal_id = $3
        ",
    )
    .bind(id)
    .bind(server_received_at)
    .bind(terminal_id)
    .execute(&state.event_insert_pool)
    .await;

    let data = WorkAssignmentAckData {
        assignment_id: id,
        status: "acknowledged".to_string(),
        acknowledged_at: server_received_at,
    };

    Ok((StatusCode::OK, Json(ApiResponse::new(data))))
}
