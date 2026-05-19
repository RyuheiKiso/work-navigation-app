// トレサビハンドラ（API-trace-001〜002）
//
// ケーストレース（順方向）とロットトレース（逆方向）を担当する。
// SQLX_OFFLINE=true 環境のため sqlx::query() 動的クエリを使用する。

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use sqlx::Row as _;
use uuid::Uuid;

use crate::{
    dto::trace::{CaseTraceResponse, LotTraceNode, LotTraceResponse, TraceEvent},
    error::AppError,
    state::AppState,
};
use wnav_auth::{AuditorRole, AuthenticatedUser};

/// ケーストレース（GET /api/v1/trace/case/{caseId}）。
///
/// AuditorRole 以上が必要。指定 case_id のイベントを時系列で返す。
#[utoipa::path(
    get,
    path = "/api/v1/trace/case/{caseId}",
    tag = "trace",
    security(("Bearer" = [])),
    params(
        ("caseId" = Uuid, Path, description = "Case ID"),
    ),
    responses(
        (status = 200, description = "ケーストレース", body = CaseTraceResponse),
        (status = 401, description = "未認証"),
        (status = 403, description = "権限不足"),
        (status = 404, description = "Case が見つからない"),
    )
)]
pub async fn case_trace(
    _user: AuthenticatedUser<AuditorRole>,
    State(state): State<AppState>,
    Path(case_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    // case の存在確認
    let case_exists: bool = sqlx::query_scalar(
        r#"SELECT EXISTS(SELECT 1 FROM work_cases WHERE id = $1)"#,
    )
    .bind(case_id)
    .fetch_one(&state.read_pool)
    .await?;

    if !case_exists {
        return Err(AppError::NotFound(format!("case:{case_id}")));
    }

    let rows = sqlx::query(
        r#"
        SELECT
            we.id, we.case_id, we.activity, we.server_received_at, we.client_recorded_at,
            we.worker_id, we.device_id, we.payload,
            (we.block_hash IS NOT NULL) AS hash_present,
            we.created_at
        FROM work_events we
        WHERE we.case_id = $1
        ORDER BY we.server_received_at ASC
        "#,
    )
    .bind(case_id)
    .fetch_all(&state.read_pool)
    .await?;

    let events: Vec<TraceEvent> = rows
        .iter()
        .map(|r| TraceEvent {
            id: r.get("id"),
            case_id: r.get("case_id"),
            activity: r.get("activity"),
            server_received_at: r.get("server_received_at"),
            client_recorded_at: r.get("client_recorded_at"),
            worker_id: r.get("worker_id"),
            device_id: r.get("device_id"),
            hash_valid: r.get::<bool, _>("hash_present"),
            payload: r.get::<Option<serde_json::Value>, _>("payload")
                .unwrap_or(serde_json::Value::Null),
            created_at: r.get("created_at"),
        })
        .collect();

    let chain_integrity = events.iter().all(|e| e.hash_valid);
    let broken_event_ids: Vec<Uuid> = events
        .iter()
        .filter(|e| !e.hash_valid)
        .map(|e| e.id)
        .collect();

    Ok((
        StatusCode::OK,
        Json(CaseTraceResponse {
            case_id,
            events,
            chain_integrity,
            broken_event_ids,
        }),
    ))
}

/// ロットトレース（GET /api/v1/trace/lot/{lotId}）。
///
/// AuditorRole 以上が必要。逆方向トレース結果を返す。
#[utoipa::path(
    get,
    path = "/api/v1/trace/lot/{lotId}",
    tag = "trace",
    security(("Bearer" = [])),
    params(
        ("lotId" = String, Path, description = "ロット ID"),
    ),
    responses(
        (status = 200, description = "ロットトレース", body = LotTraceResponse),
        (status = 401, description = "未認証"),
        (status = 403, description = "権限不足"),
        (status = 404, description = "ロットが見つからない"),
    )
)]
pub async fn lot_trace(
    _user: AuthenticatedUser<AuditorRole>,
    State(state): State<AppState>,
    Path(lot_id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let row = sqlx::query(
        r#"
        SELECT lot_id, lot_type, process_id, processed_from, processed_to
        FROM lot_records
        WHERE lot_id = $1
        "#,
    )
    .bind(&lot_id)
    .fetch_optional(&state.read_pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("lot:{lot_id}")))?;

    let case_ids: Vec<Uuid> = sqlx::query_scalar(
        r#"SELECT case_id FROM lot_case_mappings WHERE lot_id = $1"#,
    )
    .bind(&lot_id)
    .fetch_all(&state.read_pool)
    .await?;

    let upstream_lots: Vec<String> = sqlx::query_scalar(
        r#"SELECT upstream_lot_id FROM lot_lineage WHERE downstream_lot_id = $1"#,
    )
    .bind(&lot_id)
    .fetch_all(&state.read_pool)
    .await?;

    let downstream_lots: Vec<String> = sqlx::query_scalar(
        r#"SELECT downstream_lot_id FROM lot_lineage WHERE upstream_lot_id = $1"#,
    )
    .bind(&lot_id)
    .fetch_all(&state.read_pool)
    .await?;

    let nonconformance_count: i64 = sqlx::query_scalar(
        r#"SELECT COUNT(*) FROM nonconformances WHERE lot_id = $1"#,
    )
    .bind(&lot_id)
    .fetch_one(&state.read_pool)
    .await?;

    let nonconformance_lot_ids = if nonconformance_count > 0 {
        vec![lot_id.clone()]
    } else {
        vec![]
    };

    let node = LotTraceNode {
        lot_id: row.get("lot_id"),
        lot_type: row.get("lot_type"),
        case_ids,
        upstream_lots,
        downstream_lots,
        process_id: row.get("process_id"),
        processed_from: row.get("processed_from"),
        processed_to: row.get("processed_to"),
    };

    Ok((
        StatusCode::OK,
        Json(LotTraceResponse {
            lot_id,
            depth: 1,
            nodes: vec![node],
            nonconformance_lot_ids,
        }),
    ))
}
