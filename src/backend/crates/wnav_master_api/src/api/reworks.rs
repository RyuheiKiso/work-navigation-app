// リワークハンドラ（API-reworks-001 / API-rework-verifications-001）
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
    dto::reworks::{
        CreateReworkRequest, ReworkResponse, ReworkVerificationRequest,
        ReworkVerificationResponse,
    },
    error::AppError,
    state::AppState,
};
use wnav_auth::{AuthenticatedUser, MasterEditorRole};

/// リワーク登録（POST /api/v1/reworks）。
///
/// MasterEditorRole 以上が必要。
#[utoipa::path(
    post,
    path = "/api/v1/reworks",
    tag = "reworks",
    security(("Bearer" = [])),
    request_body = CreateReworkRequest,
    responses(
        (status = 201, description = "リワーク登録成功", body = ReworkResponse),
        (status = 401, description = "未認証"),
        (status = 403, description = "権限不足"),
    )
)]
pub async fn create_rework(
    user: AuthenticatedUser<MasterEditorRole>,
    State(state): State<AppState>,
    Json(req): Json<CreateReworkRequest>,
) -> Result<impl IntoResponse, AppError> {
    let new_id = Uuid::now_v7();
    let now = Utc::now();

    sqlx::query(
        r#"
        INSERT INTO reworks
            (id, case_id, instruction, rework_type, reason_code, description,
             planned_hours, status, created_by, created_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, 'pending', $8, $9)
        "#,
    )
    .bind(new_id)
    .bind(req.case_id)
    .bind(&req.instruction)
    .bind(&req.rework_type)
    .bind(&req.reason_code)
    .bind(&req.description)
    .bind(req.planned_hours)
    .bind(user.user_id)
    .bind(now)
    .execute(&state.write_pool)
    .await?;

    tracing::info!(event = "rework.created", rework_id = %new_id, "リワークを登録しました");

    Ok((
        StatusCode::CREATED,
        Json(ReworkResponse {
            id: new_id,
            case_id: req.case_id,
            instruction: req.instruction,
            rework_type: req.rework_type,
            reason_code: req.reason_code,
            status: "pending".to_string(),
            created_by: user.user_id,
            created_at: now,
        }),
    ))
}

/// リワーク検証（POST /api/v1/rework-verifications）。
///
/// Two-Person Integrity 必須（FR-AU-007）。
#[utoipa::path(
    post,
    path = "/api/v1/rework-verifications",
    tag = "reworks",
    security(("Bearer" = [])),
    request_body = ReworkVerificationRequest,
    responses(
        (status = 201, description = "リワーク検証登録成功", body = ReworkVerificationResponse),
        (status = 401, description = "未認証"),
        (status = 403, description = "権限不足"),
        (status = 422, description = "Two-Person Integrity 違反"),
    )
)]
pub async fn create_rework_verification(
    user: AuthenticatedUser<MasterEditorRole>,
    State(state): State<AppState>,
    Json(req): Json<ReworkVerificationRequest>,
) -> Result<impl IntoResponse, AppError> {
    // Two-Person Integrity 検証（FR-AU-007）
    if user.user_id == req.second_verifier_id {
        return Err(AppError::TwoPersonIntegrityViolation);
    }

    let rework_exists: bool = sqlx::query_scalar(
        r#"SELECT EXISTS(SELECT 1 FROM reworks WHERE id = $1)"#,
    )
    .bind(req.rework_id)
    .fetch_one(&state.read_pool)
    .await?;

    if !rework_exists {
        return Err(AppError::NotFound(format!("rework:{}", req.rework_id)));
    }

    let new_id = Uuid::now_v7();
    let now = Utc::now();

    sqlx::query(
        r#"
        INSERT INTO rework_verifications
            (id, rework_id, is_passed, comment, verified_by, second_verifier_id, verified_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        "#,
    )
    .bind(new_id)
    .bind(req.rework_id)
    .bind(req.is_passed)
    .bind(&req.comment)
    .bind(user.user_id)
    .bind(req.second_verifier_id)
    .bind(now)
    .execute(&state.write_pool)
    .await?;

    let new_status = if req.is_passed { "verified" } else { "failed_verification" };
    sqlx::query(r#"UPDATE reworks SET status = $1 WHERE id = $2"#)
        .bind(new_status)
        .bind(req.rework_id)
        .execute(&state.write_pool)
        .await?;

    tracing::info!(
        event = "rework.verified",
        verification_id = %new_id,
        rework_id = %req.rework_id,
        is_passed = req.is_passed,
        "リワーク検証を登録しました（Two-Person Integrity 検証済み）",
    );

    Ok((
        StatusCode::CREATED,
        Json(ReworkVerificationResponse {
            id: new_id,
            rework_id: req.rework_id,
            is_passed: req.is_passed,
            verified_by: user.user_id,
            second_verifier_id: req.second_verifier_id,
            verified_at: now,
        }),
    ))
}
