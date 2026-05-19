// 入荷検査（IQC）ハンドラ（master-api 担当分: API-iqc-004〜005・API-dispositions-001）
//
// master-api は合否判定（judge）・特採承認（concession）・ディスポジション登録を担当する。
// 入荷検査登録（API-iqc-001）と測定値入力（API-iqc-003）は terminal-api が担当する:
//   - POST /api/v1/iqc/incoming-inspections     → terminal_api/src/api/iqc.rs
//   - POST /api/v1/iqc/incoming-inspections/{id}/measurements → terminal_api/src/api/iqc.rs
// Two-Person Integrity（FR-AU-007）はディスポジション登録で必須。

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::Utc;
use sqlx::Row as _;
use uuid::Uuid;

use crate::{
    dto::iqc::{
        ApproveInspectionRequest, AqlJudgment, CreateDispositionRequest,
        DispositionResponse, IqcInspectionResponse, IqcStatus,
    },
    error::AppError,
    state::AppState,
};
use wnav_auth::{ApproverRole, AuthenticatedUser, MasterEditorRole, evaluate_roles};

/// AQL 合否判定（POST /api/v1/iqc/incoming-inspections/{id}/judge）。
///
/// MasterEditorRole 以上が必要。品質管理者による AQL 合否判定。
#[utoipa::path(
    post,
    path = "/api/v1/iqc/incoming-inspections/{id}/judge",
    tag = "iqc",
    security(("Bearer" = [])),
    params(("id" = Uuid, Path, description = "検査 ID")),
    responses(
        (status = 200, description = "検査完了", body = IqcInspectionResponse),
        (status = 401, description = "未認証"),
        (status = 403, description = "権限不足"),
        (status = 404, description = "検査が見つからない"),
    )
)]
pub async fn submit_inspection(
    _user: AuthenticatedUser<MasterEditorRole>,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let row = sqlx::query(
        r#"
        SELECT id, lot_id, supplier_id, part_number, received_quantity, sample_size,
               status, created_by, created_at, current_hash
        FROM iqc_inspections WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(&state.read_pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("iqc_inspection:{id}")))?;

    let total_defects: i64 = sqlx::query_scalar(
        r#"SELECT COALESCE(SUM(defect_count), 0) FROM iqc_measurements WHERE inspection_id = $1"#,
    )
    .bind(id)
    .fetch_one(&state.read_pool)
    .await?;

    let (aql_judgment, new_status, iqc_status) = if total_defects == 0 {
        (AqlJudgment::Accept, "passed", IqcStatus::Passed)
    } else {
        (AqlJudgment::Reject, "failed", IqcStatus::Failed)
    };

    let now = Utc::now();

    sqlx::query(
        r#"
        UPDATE iqc_inspections
        SET status = $1, aql_judgment = $2, total_defects = $3, completed_at = $4
        WHERE id = $5
        "#,
    )
    .bind(new_status)
    .bind(format!("{aql_judgment:?}").to_lowercase())
    .bind(total_defects)
    .bind(now)
    .bind(id)
    .execute(&state.write_pool)
    .await?;

    Ok((
        StatusCode::OK,
        Json(IqcInspectionResponse {
            id: row.get("id"),
            lot_id: row.get("lot_id"),
            supplier_id: row.get("supplier_id"),
            part_number: row.get("part_number"),
            received_quantity: row.get("received_quantity"),
            sample_size: row.get("sample_size"),
            status: iqc_status,
            aql_judgment: Some(aql_judgment),
            total_defects: Some(total_defects),
            created_by: row.get("created_by"),
            created_at: row.get("created_at"),
            completed_at: Some(now),
            current_hash: row.get("current_hash"),
        }),
    ))
}

/// 特採承認（POST /api/v1/iqc/incoming-inspections/{id}/concession）。
///
/// ApproverRole 必須。品質管理者による特採（CONCESSION）承認。
#[utoipa::path(
    post,
    path = "/api/v1/iqc/incoming-inspections/{id}/concession",
    tag = "iqc",
    security(("Bearer" = [])),
    params(("id" = Uuid, Path, description = "検査 ID")),
    request_body = ApproveInspectionRequest,
    responses(
        (status = 200, description = "特採承認成功", body = IqcInspectionResponse),
        (status = 401, description = "未認証"),
        (status = 403, description = "ApproverRole 必須"),
        (status = 404, description = "検査が見つからない"),
        (status = 422, description = "Failed 状態でない"),
    )
)]
pub async fn approve_inspection(
    user: AuthenticatedUser<ApproverRole>,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<ApproveInspectionRequest>,
) -> Result<impl IntoResponse, AppError> {
    let row = sqlx::query(
        r#"
        SELECT id, lot_id, supplier_id, part_number, received_quantity, sample_size,
               status, created_by, created_at, current_hash, total_defects
        FROM iqc_inspections WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(&state.read_pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("iqc_inspection:{id}")))?;

    let status: String = row.get("status");
    if status != "failed" {
        return Err(AppError::InvalidStateTransition(
            "Failed 状態の検査のみ特採承認できます".to_string(),
        ));
    }

    let now = Utc::now();

    sqlx::query(
        r#"
        UPDATE iqc_inspections
        SET status = 'concessionally_approved', approved_by = $1,
            concession_reason = $2, use_restrictions = $3, approved_at = $4
        WHERE id = $5
        "#,
    )
    .bind(user.user_id)
    .bind(&req.concession_reason)
    .bind(&req.use_restrictions)
    .bind(now)
    .bind(id)
    .execute(&state.write_pool)
    .await?;

    tracing::info!(
        event = "iqc.inspection.concessionally_approved",
        inspection_id = %id,
        approved_by = %user.user_id,
        approver_comment = ?req.approver_comment,
        "IQC 検査の特採承認を完了しました",
    );

    let total_defects: Option<i64> = row.get("total_defects");

    Ok((
        StatusCode::OK,
        Json(IqcInspectionResponse {
            id: row.get("id"),
            lot_id: row.get("lot_id"),
            supplier_id: row.get("supplier_id"),
            part_number: row.get("part_number"),
            received_quantity: row.get("received_quantity"),
            sample_size: row.get("sample_size"),
            status: IqcStatus::ConcessionallyApproved,
            aql_judgment: Some(AqlJudgment::Reject),
            total_defects,
            created_by: row.get("created_by"),
            created_at: row.get("created_at"),
            completed_at: Some(now),
            current_hash: row.get("current_hash"),
        }),
    ))
}

/// ディスポジション登録（POST /api/v1/dispositions）。
///
/// Two-Person Integrity 必須（FR-AU-007）。
#[utoipa::path(
    post,
    path = "/api/v1/dispositions",
    tag = "iqc",
    security(("Bearer" = [])),
    request_body = CreateDispositionRequest,
    responses(
        (status = 201, description = "ディスポジション登録成功", body = DispositionResponse),
        (status = 401, description = "未認証"),
        (status = 403, description = "権限不足"),
        (status = 422, description = "Two-Person Integrity 違反"),
    )
)]
pub async fn create_disposition(
    user: AuthenticatedUser<MasterEditorRole>,
    State(state): State<AppState>,
    Json(req): Json<CreateDispositionRequest>,
) -> Result<impl IntoResponse, AppError> {
    // Two-Person Integrity 検証（FR-AU-007）
    if user.user_id == req.approver_id {
        return Err(AppError::TwoPersonIntegrityViolation);
    }

    // 承認者が ApproverRole 以上であることを確認する
    let approver_row = sqlx::query(
        r#"SELECT roles FROM users WHERE id = $1 AND deleted_at IS NULL AND is_active = true"#,
    )
    .bind(req.approver_id)
    .fetch_optional(&state.read_pool)
    .await?
    .ok_or(AppError::Unauthorized)?;

    let roles_json: serde_json::Value = approver_row.get("roles");
    let approver_roles: Vec<String> = roles_json
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .filter_map(|v| v.as_str().map(ToString::to_string))
        .collect();

    if !evaluate_roles(&approver_roles, "quality_admin") {
        return Err(AppError::Forbidden);
    }

    let new_id = Uuid::now_v7();
    let now = Utc::now();

    sqlx::query(
        r#"
        INSERT INTO dispositions
            (id, inspection_id, disposition_type, reason, created_by, approved_by, approver_comment, created_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        "#,
    )
    .bind(new_id)
    .bind(req.inspection_id)
    .bind(&req.disposition_type)
    .bind(&req.reason)
    .bind(user.user_id)
    .bind(req.approver_id)
    .bind(&req.approver_comment)
    .bind(now)
    .execute(&state.write_pool)
    .await?;

    tracing::info!(
        event = "iqc.disposition.created",
        disposition_id = %new_id,
        created_by = %user.user_id,
        approved_by = %req.approver_id,
        "ディスポジションを登録しました（Two-Person Integrity 検証済み）",
    );

    Ok((
        StatusCode::CREATED,
        Json(DispositionResponse {
            id: new_id,
            inspection_id: req.inspection_id,
            disposition_type: req.disposition_type,
            reason: req.reason,
            created_by: user.user_id,
            approved_by: req.approver_id,
            created_at: now,
        }),
    ))
}
