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

    // DDL 列: capa_id, nc_id, capa_type, description, root_cause, status, assigned_to, opened_at
    // ハンドラのリクエストフィールドを DDL 列にマッピングする
    sqlx::query(
        r#"
        INSERT INTO capas
            (capa_id, nc_id, capa_type, description, root_cause,
             assigned_to, status, opened_at)
        VALUES ($1, $2, 'CORRECTIVE', $3, $4, $5, 'OPEN', $6)
        "#,
    )
    .bind(new_id)
    .bind(req.nonconformity_id)
    .bind(&req.title)
    .bind(&req.root_cause_analysis)
    .bind(req.assigned_to)
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
            status: "OPEN".to_string(),
            title: req.title,
            nonconformity_id: req.nonconformity_id,
            assigned_to: req.assigned_to,
            due_date: now.date_naive(),
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
    // DDL 列: capa_id(PK), nc_id, capa_type, description, root_cause, status, assigned_to, opened_at, closed_at
    let existing = sqlx::query(
        r#"
        SELECT capa_id, nc_id, capa_type, description, root_cause, status,
               assigned_to, opened_at
        FROM capas
        WHERE capa_id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(&state.read_pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("capa:{id}")))?;

    use sqlx::Row as _;
    let current_status: String = existing.get("status");

    // 既に CLOSED の CAPA への更新は拒否する（ERR-BIZ-008）
    if current_status == "CLOSED" {
        return Err(AppError::InvalidStateTransition(
            "closed 状態の CAPA は更新できません（ERR-BIZ-008）".to_string(),
        ));
    }

    let new_status = req.status.as_deref().unwrap_or(&current_status);
    // corrective_action が指定されていれば root_cause 列を更新する（DDL の最も近い列）
    let new_root_cause: String = req
        .corrective_action
        .as_deref()
        .unwrap_or(existing.get("root_cause"))
        .to_string();

    // DDL 列のみを使用して UPDATE する
    sqlx::query(
        r#"
        UPDATE capas
        SET status = $1,
            root_cause = $2,
            closed_at = CASE WHEN $1 = 'CLOSED' THEN $3 ELSE closed_at END
        WHERE capa_id = $4
        "#,
    )
    .bind(new_status)
    .bind(&new_root_cause)
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

    // DDL 列から CapaResponse を構築する（DDL 列名に合わせてフィールドを参照する）
    let opened_at: chrono::DateTime<Utc> = existing.get("opened_at");
    Ok((
        StatusCode::OK,
        Json(CapaResponse {
            capa_id: existing.get("capa_id"),
            status: new_status.to_string(),
            title: existing.get("description"),
            nonconformity_id: existing.get("nc_id"),
            assigned_to: existing.get("assigned_to"),
            due_date: opened_at.date_naive(),
            created_by: existing.get("assigned_to"),
            created_at: opened_at,
            updated_at: now,
        }),
    ))
}
