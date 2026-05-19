// CAPA 管理ハンドラ（API-capa-001〜002）
//
// CAPA（是正処置・予防処置）の作成と更新を担当する。
// 管理系・承認系操作のため master-api が担当する。
// SQLX_OFFLINE=true 環境のため sqlx::query() 動的クエリを使用する。

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use chrono::Utc;
use uuid::Uuid;

use crate::{
    dto::capas::{CapaResponse, CreateCapaRequest, UpdateCapaRequest},
    error::AppError,
    state::AppState,
};
use wnav_auth::{ApproverRole, AuthenticatedUser};

/// CAPA 作成（POST /api/v1/capas）。
///
/// quality_admin / system_admin のみ作成可。TBL-014（capas）にレコードを挿入する。
#[utoipa::path(
    post,
    path = "/api/v1/capas",
    tag = "capas",
    security(("Bearer" = [])),
    request_body = CreateCapaRequest,
    responses(
        (status = 201, description = "CAPA 作成成功", body = CapaResponse),
        (status = 401, description = "未認証"),
        (status = 403, description = "quality_admin / system_admin 専用"),
    )
)]
pub async fn create_capa(
    _user: AuthenticatedUser<ApproverRole>,
    State(state): State<AppState>,
    Json(req): Json<CreateCapaRequest>,
) -> Result<impl IntoResponse, AppError> {
    let new_id = Uuid::now_v7();
    let now = Utc::now();

    sqlx::query(
        r#"
        INSERT INTO capas
            (id, nonconformity_id, title, root_cause_analysis, corrective_action,
             preventive_action, assigned_to, due_date, created_by, status,
             created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, 'open', $10, $10)
        "#,
    )
    .bind(new_id)
    .bind(req.nonconformity_id)
    .bind(&req.title)
    .bind(&req.root_cause_analysis)
    .bind(&req.corrective_action)
    .bind(req.preventive_action.as_deref())
    .bind(req.assigned_to)
    .bind(req.due_date)
    .bind(req.created_by)
    .bind(now)
    .execute(&state.write_pool)
    .await?;

    tracing::info!(
        event = "capa.created",
        capa_id = %new_id,
        created_by = %req.created_by,
        "CAPA を作成しました",
    );

    Ok((
        StatusCode::CREATED,
        Json(CapaResponse {
            capa_id: new_id,
            status: "open".to_string(),
            title: req.title,
            nonconformity_id: req.nonconformity_id,
            assigned_to: req.assigned_to,
            due_date: req.due_date,
            created_by: req.created_by,
            created_at: now,
            updated_at: now,
        }),
    ))
}

/// CAPA 更新（PATCH /api/v1/capas/{id}）。
///
/// status: "closed" への変更は quality_admin のみ可。
/// 既に closed の CAPA への PATCH は ERR-BIZ-008 で拒否する。
#[utoipa::path(
    patch,
    path = "/api/v1/capas/{id}",
    tag = "capas",
    security(("Bearer" = [])),
    params(
        ("id" = Uuid, Path, description = "CAPA ID"),
    ),
    request_body = UpdateCapaRequest,
    responses(
        (status = 200, description = "CAPA 更新成功", body = CapaResponse),
        (status = 401, description = "未認証"),
        (status = 403, description = "権限不足（closed への変更は quality_admin のみ）"),
        (status = 404, description = "CAPA が見つからない"),
        (status = 409, description = "CAPA が既に closed（ERR-BIZ-008）"),
    )
)]
pub async fn update_capa(
    _user: AuthenticatedUser<ApproverRole>,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateCapaRequest>,
) -> Result<impl IntoResponse, AppError> {
    let now = Utc::now();

    // 既存の CAPA を取得して closed チェックを行う
    let existing = sqlx::query(
        r#"
        SELECT id, title, nonconformity_id, status, assigned_to, due_date,
               created_by, corrective_action, preventive_action, created_at
        FROM capas
        WHERE id = $1 AND deleted_at IS NULL
        "#,
    )
    .bind(id)
    .fetch_optional(&state.read_pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("capa:{id}")))?;

    use sqlx::Row as _;
    let current_status: String = existing.get("status");

    // 既に closed の CAPA への更新は拒否する（ERR-BIZ-008）
    if current_status == "closed" {
        return Err(AppError::InvalidStateTransition(
            "closed 状態の CAPA は更新できません（ERR-BIZ-008）".to_string(),
        ));
    }

    let new_status = req.status.as_deref().unwrap_or(&current_status);
    let new_corrective_action: String = req
        .corrective_action
        .as_deref()
        .unwrap_or(existing.get("corrective_action"))
        .to_string();
    let new_due_date: chrono::NaiveDate = req.due_date.unwrap_or_else(|| existing.get("due_date"));

    sqlx::query(
        r#"
        UPDATE capas
        SET status = $1,
            corrective_action = $2,
            preventive_action = COALESCE($3, preventive_action),
            due_date = $4,
            progress_note = COALESCE($5, progress_note),
            updated_by = $6,
            updated_at = $7
        WHERE id = $8
        "#,
    )
    .bind(new_status)
    .bind(&new_corrective_action)
    .bind(req.preventive_action.as_deref())
    .bind(new_due_date)
    .bind(req.progress_note.as_deref())
    .bind(req.updated_by)
    .bind(now)
    .bind(id)
    .execute(&state.write_pool)
    .await?;

    tracing::info!(
        event = "capa.updated",
        capa_id = %id,
        updated_by = %req.updated_by,
        new_status = new_status,
        "CAPA を更新しました",
    );

    Ok((
        StatusCode::OK,
        Json(CapaResponse {
            capa_id: existing.get("id"),
            status: new_status.to_string(),
            title: existing.get("title"),
            nonconformity_id: existing.get("nonconformity_id"),
            assigned_to: existing.get("assigned_to"),
            due_date: new_due_date,
            created_by: existing.get("created_by"),
            created_at: existing.get("created_at"),
            updated_at: now,
        }),
    ))
}
