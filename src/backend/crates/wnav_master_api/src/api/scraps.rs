// 廃却・返品記録ハンドラ
//
// SQLX_OFFLINE=true 環境のため sqlx::query() 動的クエリを使用する。

use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::Utc;
use uuid::Uuid;

use crate::{
    dto::scraps::{
        ReturnRecordRequest, ReturnRecordResponse, ScrapRecordRequest, ScrapRecordResponse,
    },
    error::AppError,
    state::AppState,
};
use wnav_auth::{AuthenticatedUser, MasterEditorRole};

/// 廃却記録（POST /api/v1/scrap-records）。
///
/// MasterEditorRole 以上が必要。
#[utoipa::path(
    post,
    path = "/api/v1/scrap-records",
    tag = "scraps",
    security(("Bearer" = [])),
    request_body = ScrapRecordRequest,
    responses(
        (status = 201, description = "廃却記録登録成功", body = ScrapRecordResponse),
        (status = 401, description = "未認証"),
        (status = 403, description = "権限不足"),
    )
)]
pub async fn create_scrap_record(
    user: AuthenticatedUser<MasterEditorRole>,
    State(state): State<AppState>,
    Json(req): Json<ScrapRecordRequest>,
) -> Result<impl IntoResponse, AppError> {
    let new_id = Uuid::now_v7();
    let now = Utc::now();

    sqlx::query(
        r#"
        INSERT INTO scrap_records
            (id, lot_id, quantity, reason_code, description, cost, scrapped_at, created_by, created_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        "#,
    )
    .bind(new_id)
    .bind(&req.lot_id)
    .bind(req.quantity)
    .bind(&req.reason_code)
    .bind(&req.description)
    .bind(req.cost)
    .bind(req.scrapped_at)
    .bind(user.user_id)
    .bind(now)
    .execute(&state.write_pool)
    .await?;

    tracing::info!(event = "scrap.record.created", scrap_id = %new_id, "廃却記録を登録しました");

    Ok((
        StatusCode::CREATED,
        Json(ScrapRecordResponse {
            id: new_id,
            lot_id: req.lot_id,
            quantity: req.quantity,
            reason_code: req.reason_code,
            created_by: user.user_id,
            created_at: now,
        }),
    ))
}

/// 返品記録（POST /api/v1/return-records）。
///
/// MasterEditorRole 以上が必要。
#[utoipa::path(
    post,
    path = "/api/v1/return-records",
    tag = "scraps",
    security(("Bearer" = [])),
    request_body = ReturnRecordRequest,
    responses(
        (status = 201, description = "返品記録登録成功", body = ReturnRecordResponse),
        (status = 401, description = "未認証"),
        (status = 403, description = "権限不足"),
    )
)]
pub async fn create_return_record(
    user: AuthenticatedUser<MasterEditorRole>,
    State(state): State<AppState>,
    Json(req): Json<ReturnRecordRequest>,
) -> Result<impl IntoResponse, AppError> {
    let new_id = Uuid::now_v7();
    let now = Utc::now();

    sqlx::query(
        r#"
        INSERT INTO return_records
            (id, lot_id, supplier_id, quantity, reason_code, description, returned_at, created_by, created_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        "#,
    )
    .bind(new_id)
    .bind(&req.lot_id)
    .bind(&req.supplier_id)
    .bind(req.quantity)
    .bind(&req.reason_code)
    .bind(&req.description)
    .bind(req.returned_at)
    .bind(user.user_id)
    .bind(now)
    .execute(&state.write_pool)
    .await?;

    tracing::info!(event = "return.record.created", return_id = %new_id, "返品記録を登録しました");

    Ok((
        StatusCode::CREATED,
        Json(ReturnRecordResponse {
            id: new_id,
            lot_id: req.lot_id,
            supplier_id: req.supplier_id,
            quantity: req.quantity,
            reason_code: req.reason_code,
            created_by: user.user_id,
            created_at: now,
        }),
    ))
}
