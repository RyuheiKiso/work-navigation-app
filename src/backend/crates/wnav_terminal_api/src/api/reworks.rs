// リワークハンドラ（terminal-api 担当分: API-reworks-001・API-rework-verifications-001）
//
// 現場端末からのリワーク作業開始と再検査記録を担当する。
// ディスポジション承認（master-api）を経た後に現場でリワークを実施する。
// Two-Person Integrity: 作業者 ≠ 検証者（BR-BUS-042）を強制する。
// event_insert_pool（app_event_insert ロール）を使用する。

use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use chrono::Utc;
use uuid::Uuid;

use crate::{
    dto::reworks::{
        CreateReworkRequest, CreateReworkVerificationRequest, ReworkResponse, VerificationResponse,
    },
    error::AppError,
    state::AppState,
};

/// リワーク作業開始（POST /api/v1/reworks）。
///
/// 現場端末からのリワーク作業開始記録。Idempotency-Key ヘッダ必須。
/// ディスポジションが REWORK 判定済みであることを確認してから開始する。
/// event_insert_pool に INSERT する（app_event_insert ロール）。
#[utoipa::path(
    post,
    path = "/api/v1/reworks",
    operation_id = "createRework",
    request_body = CreateReworkRequest,
    responses(
        (status = 201, description = "リワーク作業開始成功", body = ReworkResponse),
        (status = 401, description = "認証エラー"),
        (status = 404, description = "ディスポジションが見つからない"),
        (status = 409, description = "ビジネスルール違反"),
        (status = 422, description = "バリデーションエラー"),
    ),
    security(("bearer_auth" = [])),
    tag = "reworks",
)]
pub async fn create_rework(
    State(state): State<AppState>,
    Json(req): Json<CreateReworkRequest>,
) -> Result<impl IntoResponse, AppError> {
    // ディスポジションが REWORK 判定済みであることを確認する
    let disposition_valid: bool = sqlx::query_scalar(
        r#"
        SELECT EXISTS(
            SELECT 1 FROM dispositions
            WHERE id = $1 AND decision = 'REWORK'
        )
        "#,
    )
    .bind(req.disposition_id)
    .fetch_one(&state.read_pool)
    .await
    .map_err(|_| AppError::DatabaseError)?;

    if !disposition_valid {
        return Err(AppError::NotFound);
    }

    let new_id = Uuid::now_v7();
    let now = Utc::now();

    // リワークレコードを Append-only で挿入する
    sqlx::query(
        r#"
        INSERT INTO reworks
            (id, disposition_id, operator_id, instruction, status, created_at)
        VALUES ($1, $2, $3, $4, 'IN_PROGRESS', $5)
        "#,
    )
    .bind(new_id)
    .bind(req.disposition_id)
    .bind(req.operator_id)
    .bind(req.instruction.as_deref())
    .bind(now)
    .execute(&state.event_insert_pool)
    .await
    .map_err(|_| AppError::DatabaseError)?;

    tracing::info!(
        event = "rework.started",
        rework_id = %new_id,
        disposition_id = %req.disposition_id,
        operator_id = %req.operator_id,
        "リワーク作業を開始しました",
    );

    Ok((
        StatusCode::CREATED,
        Json(ReworkResponse {
            rework_id: new_id,
            disposition_id: req.disposition_id,
            operator_id: req.operator_id,
            status: "IN_PROGRESS".to_string(),
            created_at: now,
        }),
    ))
}

/// リワーク再検査記録（POST /api/v1/rework-verifications）。
///
/// 現場端末での再検査結果を Append-only で記録する。
/// Two-Person Integrity（BR-BUS-042）: verifier_id ≠ リワーク実施者。
/// event_insert_pool に INSERT する（app_event_insert ロール）。
#[utoipa::path(
    post,
    path = "/api/v1/rework-verifications",
    operation_id = "createReworkVerification",
    request_body = CreateReworkVerificationRequest,
    responses(
        (status = 201, description = "リワーク再検査記録成功", body = VerificationResponse),
        (status = 401, description = "認証エラー"),
        (status = 404, description = "リワークが見つからない"),
        (status = 409, description = "ビジネスルール違反"),
        (status = 422, description = "Two-Person Integrity 違反"),
    ),
    security(("bearer_auth" = [])),
    tag = "reworks",
)]
pub async fn create_rework_verification(
    State(state): State<AppState>,
    Json(req): Json<CreateReworkVerificationRequest>,
) -> Result<impl IntoResponse, AppError> {
    // リワークの存在確認と実施者 ID 取得
    let operator_id: Option<Uuid> =
        sqlx::query_scalar(r#"SELECT operator_id FROM reworks WHERE id = $1"#)
            .bind(req.rework_id)
            .fetch_optional(&state.read_pool)
            .await
            .map_err(|_| AppError::DatabaseError)?;

    let operator_id = operator_id.ok_or(AppError::NotFound)?;

    // Two-Person Integrity（BR-BUS-042）: 検証者はリワーク実施者と異なること（ERR-BIZ-023）
    if req.verifier_id == operator_id {
        // 再検査者同一エラー: StepSequenceViolation を使用する（ERR-BIZ-023 に対応）
        return Err(AppError::StepSequenceViolation);
    }

    let new_id = Uuid::now_v7();
    let now = Utc::now();

    // 再検査レコードを Append-only で挿入する
    sqlx::query(
        r#"
        INSERT INTO rework_verifications
            (id, rework_id, verifier_id, passed, comment, created_at)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
    )
    .bind(new_id)
    .bind(req.rework_id)
    .bind(req.verifier_id)
    .bind(req.passed)
    .bind(req.comment.as_deref())
    .bind(now)
    .execute(&state.event_insert_pool)
    .await
    .map_err(|_| AppError::DatabaseError)?;

    tracing::info!(
        event = "rework.verification.recorded",
        verification_id = %new_id,
        rework_id = %req.rework_id,
        verifier_id = %req.verifier_id,
        passed = req.passed,
        "リワーク再検査を記録しました（Two-Person Integrity 検証済み）",
    );

    Ok((
        StatusCode::CREATED,
        Json(VerificationResponse {
            verification_id: new_id,
            rework_id: req.rework_id,
            verifier_id: req.verifier_id,
            passed: req.passed,
            created_at: now,
        }),
    ))
}
