// 非適合品登録ハンドラ（terminal-api 担当分: 補足仕様）
//
// 現場端末からの非適合品起票を担当する。
// event_insert_pool（app_event_insert ロール）を使用する。

use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use chrono::Utc;
use uuid::Uuid;

use crate::{
    dto::reworks::{NonconformityResponse, RegisterNonconformityRequest},
    error::AppError,
    state::AppState,
};

/// 非適合品登録（POST /api/v1/nonconformities）。
///
/// 現場端末からの非適合品起票。Append-only で TBL-013（nonconformities）に記録する。
/// event_insert_pool に INSERT する（app_event_insert ロール）。
#[utoipa::path(
    post,
    path = "/api/v1/nonconformities",
    operation_id = "registerNonconformity",
    request_body = RegisterNonconformityRequest,
    responses(
        (status = 201, description = "非適合品登録成功", body = NonconformityResponse),
        (status = 401, description = "認証エラー"),
        (status = 422, description = "バリデーションエラー"),
    ),
    security(("bearer_auth" = [])),
    tag = "nonconformities",
)]
pub async fn register_nonconformity(
    State(state): State<AppState>,
    Json(req): Json<RegisterNonconformityRequest>,
) -> Result<impl IntoResponse, AppError> {
    let new_id = Uuid::now_v7();
    let now = Utc::now();

    let evidence_ids_json = serde_json::json!(req.evidence_ids.unwrap_or_default());

    sqlx::query(
        r#"
        INSERT INTO nonconformities
            (id, alert_id, work_execution_id, lot_id, nc_type, description,
             discovered_by, evidence_ids, status, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, 'open', $9, $9)
        "#,
    )
    .bind(new_id)
    .bind(req.alert_id)
    .bind(req.work_execution_id)
    .bind(req.lot_id)
    .bind(&req.nc_type)
    .bind(&req.description)
    .bind(req.discovered_by)
    .bind(&evidence_ids_json)
    .bind(now)
    .execute(&state.event_insert_pool)
    .await
    .map_err(|_| AppError::DatabaseError)?;

    tracing::info!(
        event = "nonconformity.registered",
        nonconformity_id = %new_id,
        nc_type = %req.nc_type,
        "非適合品を登録しました",
    );

    Ok((
        StatusCode::CREATED,
        Json(NonconformityResponse {
            nonconformity_id: new_id,
            nc_type: req.nc_type,
            status: "open".to_string(),
            created_at: now,
        }),
    ))
}
