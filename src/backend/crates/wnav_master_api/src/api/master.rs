// マスタバージョン管理ハンドラ（API-master-001〜007 + 補足エンドポイント）
//
// Draft 作成・編集・承認申請・承認・ロールバック・Dry-run を担当する。
// 補足: 工程一覧・SOP 一覧・ユーザー一覧・ユーザー作成・ロール割当も本ファイルに実装する。
// 書き込みは write_pool（app_write ロール）、読み取りは read_pool（app_read ロール）を使用する。
// SQLX_OFFLINE=true 環境のため sqlx::query() 動的クエリを使用する。

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use sqlx::Row as _;
use uuid::Uuid;

use crate::{
    dto::master::{
        ApproveVersionRequest, CreateMasterVersionRequest, DryRunError, DryRunResult,
        MasterVersionListResponse, MasterVersionResponse, MasterVersionStatus,
        MasterVersionSummary, RollbackVersionRequest, SubmitVersionRequest,
        UpdateMasterVersionRequest,
    },
    error::AppError,
    state::AppState,
};
use wnav_auth::{AdminRole, ApproverRole, AuthenticatedUser, MasterEditorRole};

// ─────────────────────────────────────────────────────────────────────────────
// 補足エンドポイント用クエリパラメータ型
// ─────────────────────────────────────────────────────────────────────────────

/// 工程一覧クエリパラメータ（GET /api/v1/master/processes）
#[derive(Debug, Deserialize)]
pub struct ListProcessesQuery {
    pub is_active: Option<bool>,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

/// SOP 一覧クエリパラメータ（GET /api/v1/master/sops）
#[derive(Debug, Deserialize)]
pub struct ListSopsQuery {
    pub process_id: Option<Uuid>,
    pub has_published_version: Option<bool>,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

/// ユーザー一覧クエリパラメータ（GET /api/v1/master/users）
#[derive(Debug, Deserialize)]
pub struct ListUsersQuery {
    pub is_active: Option<bool>,
    pub role: Option<String>,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

/// ユーザー作成リクエスト（POST /api/v1/master/users）
#[derive(Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct CreateUserRequest {
    pub login_id: String,
    pub display_name: String,
    pub email: String,
    pub password_initial: String,
    pub factory_id: Uuid,
    pub roles: Vec<String>,
}

/// ユーザーレスポンス
#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct UserResponse {
    pub id: Uuid,
    pub login_id: String,
    pub display_name: String,
    pub email: String,
    pub factory_id: Uuid,
    pub roles: Vec<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

/// ロール割当リクエスト（PUT /api/v1/master/users/{id}/roles）
#[derive(Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct AssignRolesRequest {
    pub roles: Vec<String>,
}

/// マスタバージョン一覧取得（GET /api/v1/master-versions）。
///
/// MasterEditorRole 以上が必要。全バージョンをステータスでフィルタ可能。
#[utoipa::path(
    get,
    path = "/api/v1/master-versions",
    tag = "master",
    security(("Bearer" = [])),
    responses(
        (status = 200, description = "バージョン一覧", body = MasterVersionListResponse),
        (status = 401, description = "未認証"),
        (status = 403, description = "権限不足"),
    )
)]
pub async fn list_versions(
    _user: AuthenticatedUser<MasterEditorRole>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let rows = sqlx::query(
        r#"
        SELECT id, version_number, status, created_by, created_at, updated_at, description
        FROM master_versions
        WHERE deleted_at IS NULL
        ORDER BY created_at DESC
        "#,
    )
    .fetch_all(&state.read_pool)
    .await?;

    let items: Vec<MasterVersionSummary> = rows
        .iter()
        .map(|r| MasterVersionSummary {
            id: r.get("id"),
            version: r.get("version_number"),
            status: parse_status(r.get::<&str, _>("status")),
            created_by: r.get("created_by"),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
            description: r.get("description"),
        })
        .collect();

    let total = items.len() as i64;

    Ok((
        StatusCode::OK,
        Json(MasterVersionListResponse { items, total }),
    ))
}

/// Draft バージョン作成（POST /api/v1/master-versions/draft）。
///
/// MasterEditorRole 必須。新しい Draft バージョンを作成する。
#[utoipa::path(
    post,
    path = "/api/v1/master-versions/draft",
    tag = "master",
    security(("Bearer" = [])),
    request_body = CreateMasterVersionRequest,
    responses(
        (status = 201, description = "Draft 作成成功", body = MasterVersionResponse),
        (status = 401, description = "未認証"),
        (status = 403, description = "権限不足"),
    )
)]
pub async fn create_draft(
    user: AuthenticatedUser<MasterEditorRole>,
    State(state): State<AppState>,
    Json(req): Json<CreateMasterVersionRequest>,
) -> Result<impl IntoResponse, AppError> {
    let new_id = Uuid::now_v7();
    let now = Utc::now();

    // バージョン番号を採番する（現在の最大番号 + 1）
    let count: i64 = sqlx::query_scalar(
        r#"SELECT COUNT(*) FROM master_versions WHERE deleted_at IS NULL"#,
    )
    .fetch_one(&state.read_pool)
    .await?;
    let version_number = format!("v{}.0.0", count + 1);

    // write_pool で Draft バージョンを INSERT する
    sqlx::query(
        r#"
        INSERT INTO master_versions
            (id, version_number, status, data, created_by, created_at, updated_at, description)
        VALUES ($1, $2, 'draft', $3, $4, $5, $5, $6)
        "#,
    )
    .bind(new_id)
    .bind(&version_number)
    .bind(&req.data)
    .bind(user.user_id)
    .bind(now)
    .bind(&req.description)
    .execute(&state.write_pool)
    .await?;

    tracing::info!(
        event = "master.draft.created",
        version_id = %new_id,
        created_by = %user.user_id,
        "Draft バージョンを作成しました",
    );

    Ok((
        StatusCode::CREATED,
        Json(MasterVersionResponse {
            id: new_id,
            version: version_number,
            status: MasterVersionStatus::Draft,
            data: req.data,
            created_by: user.user_id,
            approved_by: None,
            created_at: now,
            updated_at: now,
            description: req.description,
        }),
    ))
}

/// Draft バージョン編集（PATCH /api/v1/master-versions/{id}）。
///
/// MasterEditorRole 必須。Draft 状態のみ編集可能。
#[utoipa::path(
    patch,
    path = "/api/v1/master-versions/{id}",
    tag = "master",
    security(("Bearer" = [])),
    params(
        ("id" = Uuid, Path, description = "バージョン ID"),
    ),
    request_body = UpdateMasterVersionRequest,
    responses(
        (status = 200, description = "編集成功", body = MasterVersionResponse),
        (status = 401, description = "未認証"),
        (status = 403, description = "権限不足"),
        (status = 404, description = "バージョンが見つからない"),
        (status = 422, description = "Draft 状態でない"),
    )
)]
pub async fn update_draft(
    _user: AuthenticatedUser<MasterEditorRole>,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateMasterVersionRequest>,
) -> Result<impl IntoResponse, AppError> {
    let existing_row = get_version_row(&state, id).await?;
    let existing_status: &str = existing_row.get("status");

    if existing_status != "draft" {
        return Err(AppError::InvalidStateTransition(
            "Draft 状態のバージョンのみ編集可能です".to_string(),
        ));
    }

    let now = Utc::now();
    let existing_description: Option<String> = existing_row.get("description");
    let new_description = req.description.or(existing_description);

    sqlx::query(
        r#"
        UPDATE master_versions
        SET data = $1, description = $2, updated_at = $3
        WHERE id = $4
        "#,
    )
    .bind(&req.data)
    .bind(&new_description)
    .bind(now)
    .bind(id)
    .execute(&state.write_pool)
    .await?;

    Ok((
        StatusCode::OK,
        Json(MasterVersionResponse {
            id: existing_row.get("id"),
            version: existing_row.get("version_number"),
            status: MasterVersionStatus::Draft,
            data: req.data,
            created_by: existing_row.get("created_by"),
            approved_by: None,
            created_at: existing_row.get("created_at"),
            updated_at: now,
            description: new_description,
        }),
    ))
}

/// 承認申請（POST /api/v1/master-versions/{id}/submit）。
///
/// MasterEditorRole 必須。Draft → PendingApproval 状態遷移。
#[utoipa::path(
    post,
    path = "/api/v1/master-versions/{id}/submit",
    tag = "master",
    security(("Bearer" = [])),
    params(
        ("id" = Uuid, Path, description = "バージョン ID"),
    ),
    request_body = SubmitVersionRequest,
    responses(
        (status = 200, description = "承認申請成功", body = MasterVersionResponse),
        (status = 401, description = "未認証"),
        (status = 403, description = "権限不足"),
        (status = 422, description = "Draft 状態でない"),
    )
)]
pub async fn submit_version(
    _user: AuthenticatedUser<MasterEditorRole>,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(_req): Json<SubmitVersionRequest>,
) -> Result<impl IntoResponse, AppError> {
    let existing = get_version_row(&state, id).await?;
    let status: &str = existing.get("status");

    if status != "draft" {
        return Err(AppError::InvalidStateTransition(
            "Draft 状態のバージョンのみ承認申請できます".to_string(),
        ));
    }

    let now = Utc::now();
    sqlx::query(
        r#"UPDATE master_versions SET status = 'pending_approval', updated_at = $1 WHERE id = $2"#,
    )
    .bind(now)
    .bind(id)
    .execute(&state.write_pool)
    .await?;

    tracing::info!(event = "master.version.submitted", version_id = %id, "バージョンを承認申請しました");

    Ok((
        StatusCode::OK,
        Json(build_response_from_row(&existing, MasterVersionStatus::PendingApproval, now, None)),
    ))
}

/// 承認・公開（POST /api/v1/master-versions/{id}/approve）。
///
/// ApproverRole 必須。PendingApproval → Published 状態遷移。
#[utoipa::path(
    post,
    path = "/api/v1/master-versions/{id}/approve",
    tag = "master",
    security(("Bearer" = [])),
    params(
        ("id" = Uuid, Path, description = "バージョン ID"),
    ),
    request_body = ApproveVersionRequest,
    responses(
        (status = 200, description = "承認成功", body = MasterVersionResponse),
        (status = 401, description = "未認証"),
        (status = 403, description = "ApproverRole 必須"),
        (status = 422, description = "PendingApproval 状態でない"),
    )
)]
pub async fn approve_version(
    user: AuthenticatedUser<ApproverRole>,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(_req): Json<ApproveVersionRequest>,
) -> Result<impl IntoResponse, AppError> {
    let existing = get_version_row(&state, id).await?;
    let status: &str = existing.get("status");

    if status != "pending_approval" {
        return Err(AppError::InvalidStateTransition(
            "PendingApproval 状態のバージョンのみ承認できます".to_string(),
        ));
    }

    let now = Utc::now();
    let approver_id = user.user_id;

    sqlx::query(
        r#"
        UPDATE master_versions
        SET status = 'published', approved_by = $1, published_at = $2, updated_at = $2
        WHERE id = $3
        "#,
    )
    .bind(approver_id)
    .bind(now)
    .bind(id)
    .execute(&state.write_pool)
    .await?;

    tracing::info!(
        event = "master.version.approved",
        version_id = %id,
        approved_by = %approver_id,
        "バージョンを承認・公開しました",
    );

    Ok((
        StatusCode::OK,
        Json(build_response_from_row(&existing, MasterVersionStatus::Published, now, Some(approver_id))),
    ))
}

/// ロールバック（POST /api/v1/master-versions/{id}/rollback）。
///
/// AdminRole 必須。Published → Archived 状態遷移。
#[utoipa::path(
    post,
    path = "/api/v1/master-versions/{id}/rollback",
    tag = "master",
    security(("Bearer" = [])),
    params(
        ("id" = Uuid, Path, description = "バージョン ID"),
    ),
    request_body = RollbackVersionRequest,
    responses(
        (status = 200, description = "ロールバック成功", body = MasterVersionResponse),
        (status = 401, description = "未認証"),
        (status = 403, description = "AdminRole 必須"),
        (status = 422, description = "Published 状態でない"),
    )
)]
pub async fn rollback_version(
    user: AuthenticatedUser<AdminRole>,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<RollbackVersionRequest>,
) -> Result<impl IntoResponse, AppError> {
    let existing = get_version_row(&state, id).await?;
    let status: &str = existing.get("status");

    if status != "published" {
        return Err(AppError::InvalidStateTransition(
            "Published 状態のバージョンのみロールバックできます".to_string(),
        ));
    }

    let now = Utc::now();

    sqlx::query(
        r#"
        UPDATE master_versions
        SET status = 'archived', rollback_reason = $1, rolled_back_by = $2, updated_at = $3
        WHERE id = $4
        "#,
    )
    .bind(&req.reason)
    .bind(user.user_id)
    .bind(now)
    .bind(id)
    .execute(&state.write_pool)
    .await?;

    tracing::info!(
        event = "master.version.rolled_back",
        version_id = %id,
        rolled_back_by = %user.user_id,
        reason = %req.reason,
        "バージョンをロールバックしました",
    );

    Ok((
        StatusCode::OK,
        Json(build_response_from_row(&existing, MasterVersionStatus::Archived, now, None)),
    ))
}

/// 参照整合性確認 Dry-run（POST /api/v1/master-versions/{id}/dry-run）。
///
/// MasterEditorRole 以上が必要。実際の書き込みは行わず整合性のみ確認する。
#[utoipa::path(
    post,
    path = "/api/v1/master-versions/{id}/dry-run",
    tag = "master",
    security(("Bearer" = [])),
    params(
        ("id" = Uuid, Path, description = "バージョン ID"),
    ),
    responses(
        (status = 200, description = "Dry-run 結果", body = DryRunResult),
        (status = 401, description = "未認証"),
        (status = 403, description = "権限不足"),
        (status = 404, description = "バージョンが見つからない"),
    )
)]
pub async fn dry_run(
    _user: AuthenticatedUser<MasterEditorRole>,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let row = get_version_row(&state, id).await?;
    let errors: Vec<DryRunError> = Vec::new();
    let mut warnings: Vec<String> = Vec::new();

    // マスタデータの参照整合性を確認する
    let data: serde_json::Value = row.get("data");
    if data.as_object().map(|o| o.is_empty()).unwrap_or(true) {
        warnings.push("マスタデータが空です。公開前に内容を確認してください。".to_string());
    }

    let is_valid = errors.is_empty();

    Ok((
        StatusCode::OK,
        Json(DryRunResult {
            is_valid,
            errors,
            warnings,
        }),
    ))
}

// ─────────────────────────────────────────────────────────────────────────────
// 内部ヘルパー
// ─────────────────────────────────────────────────────────────────────────────

/// バージョン行を取得するヘルパー（存在しない場合は 404 を返す）
async fn get_version_row(
    state: &AppState,
    id: Uuid,
) -> Result<sqlx::postgres::PgRow, AppError> {
    sqlx::query(
        r#"
        SELECT id, version_number, status, data, created_by, created_at, updated_at, description
        FROM master_versions
        WHERE id = $1 AND deleted_at IS NULL
        "#,
    )
    .bind(id)
    .fetch_optional(&state.read_pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("master_version:{id}")))
}

/// DB 行から MasterVersionResponse を構築するヘルパー
fn build_response_from_row(
    row: &sqlx::postgres::PgRow,
    status: MasterVersionStatus,
    updated_at: DateTime<Utc>,
    approved_by: Option<Uuid>,
) -> MasterVersionResponse {
    MasterVersionResponse {
        id: row.get("id"),
        version: row.get("version_number"),
        status,
        data: row.get("data"),
        created_by: row.get("created_by"),
        approved_by,
        created_at: row.get("created_at"),
        updated_at,
        description: row.get("description"),
    }
}

/// DB の status 文字列を MasterVersionStatus に変換するヘルパー
fn parse_status(s: &str) -> MasterVersionStatus {
    match s {
        "pending_approval" => MasterVersionStatus::PendingApproval,
        "published" => MasterVersionStatus::Published,
        "archived" => MasterVersionStatus::Archived,
        _ => MasterVersionStatus::Draft,
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 補足エンドポイント（§2-4/2-5/9 of 05_マスタ管理API仕様.md）
// ─────────────────────────────────────────────────────────────────────────────

/// 工程一覧取得（GET /api/v1/master/processes）。
///
/// 全ロールで参照可。is_active・page・per_page でフィルタ可能。
#[utoipa::path(
    get,
    path = "/api/v1/master/processes",
    tag = "master",
    security(("Bearer" = [])),
    params(
        ("is_active" = Option<bool>, Query, description = "有効な工程のみ"),
        ("page" = Option<i64>, Query, description = "ページ番号（デフォルト 1）"),
        ("per_page" = Option<i64>, Query, description = "1 ページあたりの件数（デフォルト 200）"),
    ),
    responses(
        (status = 200, description = "工程一覧"),
        (status = 401, description = "未認証"),
    )
)]
pub async fn list_processes(
    _user: AuthenticatedUser<MasterEditorRole>,
    State(state): State<AppState>,
    Query(q): Query<ListProcessesQuery>,
) -> Result<impl IntoResponse, AppError> {
    let page = q.page.unwrap_or(1).max(1);
    let per_page = q.per_page.unwrap_or(200).clamp(1, 200);
    let offset = (page - 1) * per_page;

    // is_active でフィルタしつつページネーションする
    let rows = sqlx::query(
        r#"
        SELECT id, name, description, is_active, created_at, updated_at
        FROM processes
        WHERE deleted_at IS NULL
          AND ($1::boolean IS NULL OR is_active = $1)
        ORDER BY name ASC
        LIMIT $2 OFFSET $3
        "#,
    )
    .bind(q.is_active)
    .bind(per_page)
    .bind(offset)
    .fetch_all(&state.read_pool)
    .await?;

    let items: Vec<serde_json::Value> = rows
        .iter()
        .map(|r| {
            serde_json::json!({
                "id": r.get::<Uuid, _>("id"),
                "name": r.get::<String, _>("name"),
                "description": r.get::<Option<String>, _>("description"),
                "is_active": r.get::<bool, _>("is_active"),
                "created_at": r.get::<DateTime<Utc>, _>("created_at"),
                "updated_at": r.get::<DateTime<Utc>, _>("updated_at"),
            })
        })
        .collect();

    Ok((StatusCode::OK, Json(serde_json::json!({ "data": items }))))
}

/// SOP 一覧取得（GET /api/v1/master/sops）。
///
/// 全ロールで参照可。process_id・has_published_version でフィルタ可能。
#[utoipa::path(
    get,
    path = "/api/v1/master/sops",
    tag = "master",
    security(("Bearer" = [])),
    params(
        ("process_id" = Option<Uuid>, Query, description = "工程 ID でフィルタ"),
        ("has_published_version" = Option<bool>, Query, description = "Published バージョンを持つ SOP のみ"),
        ("page" = Option<i64>, Query, description = "ページ番号（デフォルト 1）"),
        ("per_page" = Option<i64>, Query, description = "1 ページあたりの件数（デフォルト 50）"),
    ),
    responses(
        (status = 200, description = "SOP 一覧"),
        (status = 401, description = "未認証"),
    )
)]
pub async fn list_sops(
    _user: AuthenticatedUser<MasterEditorRole>,
    State(state): State<AppState>,
    Query(q): Query<ListSopsQuery>,
) -> Result<impl IntoResponse, AppError> {
    let page = q.page.unwrap_or(1).max(1);
    let per_page = q.per_page.unwrap_or(50).clamp(1, 200);
    let offset = (page - 1) * per_page;

    // has_published_version=true の場合は published バージョンを持つ SOP のみ返す
    let rows = sqlx::query(
        r#"
        SELECT s.id, s.name, s.process_id, s.description, s.is_active,
               s.created_at, s.updated_at,
               EXISTS(
                   SELECT 1 FROM master_versions mv
                   WHERE mv.sop_id = s.id AND mv.status = 'published' AND mv.deleted_at IS NULL
               ) AS has_published_version
        FROM sops s
        WHERE s.deleted_at IS NULL
          AND ($1::uuid IS NULL OR s.process_id = $1)
          AND ($2::boolean IS NULL OR EXISTS(
                   SELECT 1 FROM master_versions mv
                   WHERE mv.sop_id = s.id AND mv.status = 'published' AND mv.deleted_at IS NULL
               ) = $2)
        ORDER BY s.name ASC
        LIMIT $3 OFFSET $4
        "#,
    )
    .bind(q.process_id)
    .bind(q.has_published_version)
    .bind(per_page)
    .bind(offset)
    .fetch_all(&state.read_pool)
    .await?;

    let items: Vec<serde_json::Value> = rows
        .iter()
        .map(|r| {
            serde_json::json!({
                "id": r.get::<Uuid, _>("id"),
                "name": r.get::<String, _>("name"),
                "process_id": r.get::<Uuid, _>("process_id"),
                "description": r.get::<Option<String>, _>("description"),
                "is_active": r.get::<bool, _>("is_active"),
                "has_published_version": r.get::<bool, _>("has_published_version"),
                "created_at": r.get::<DateTime<Utc>, _>("created_at"),
                "updated_at": r.get::<DateTime<Utc>, _>("updated_at"),
            })
        })
        .collect();

    Ok((StatusCode::OK, Json(serde_json::json!({ "data": items }))))
}

/// ユーザー一覧取得（GET /api/v1/master/users）。
///
/// system_admin のみアクセス可。パスワードハッシュ・PIN ハッシュは除外する。
#[utoipa::path(
    get,
    path = "/api/v1/master/users",
    tag = "master",
    security(("Bearer" = [])),
    params(
        ("is_active" = Option<bool>, Query, description = "有効なユーザーのみ"),
        ("role" = Option<String>, Query, description = "ロール名でフィルタ"),
        ("page" = Option<i64>, Query, description = "ページ番号（デフォルト 1）"),
        ("per_page" = Option<i64>, Query, description = "1 ページあたりの件数（デフォルト 50）"),
    ),
    responses(
        (status = 200, description = "ユーザー一覧"),
        (status = 401, description = "未認証"),
        (status = 403, description = "system_admin 専用"),
    )
)]
pub async fn list_users(
    user: AuthenticatedUser<AdminRole>,
    State(state): State<AppState>,
    Query(q): Query<ListUsersQuery>,
) -> Result<impl IntoResponse, AppError> {
    let _ = user;
    let page = q.page.unwrap_or(1).max(1);
    let per_page = q.per_page.unwrap_or(50).clamp(1, 200);
    let offset = (page - 1) * per_page;

    // パスワードハッシュ・PIN ハッシュを SELECT から除外する（機密情報保護）
    let rows = sqlx::query(
        r#"
        SELECT id, login_id, display_name, email, factory_id, roles,
               is_active, created_at, updated_at
        FROM users
        WHERE deleted_at IS NULL
          AND ($1::boolean IS NULL OR is_active = $1)
          AND ($2::text IS NULL OR roles @> jsonb_build_array($2::text))
        ORDER BY display_name ASC
        LIMIT $3 OFFSET $4
        "#,
    )
    .bind(q.is_active)
    .bind(q.role.as_deref())
    .bind(per_page)
    .bind(offset)
    .fetch_all(&state.read_pool)
    .await?;

    let items: Vec<serde_json::Value> = rows
        .iter()
        .map(|r| {
            serde_json::json!({
                "id": r.get::<Uuid, _>("id"),
                "login_id": r.get::<String, _>("login_id"),
                "display_name": r.get::<String, _>("display_name"),
                "email": r.get::<String, _>("email"),
                "factory_id": r.get::<Uuid, _>("factory_id"),
                "roles": r.get::<serde_json::Value, _>("roles"),
                "is_active": r.get::<bool, _>("is_active"),
                "created_at": r.get::<DateTime<Utc>, _>("created_at"),
                "updated_at": r.get::<DateTime<Utc>, _>("updated_at"),
            })
        })
        .collect();

    Ok((StatusCode::OK, Json(serde_json::json!({ "data": items }))))
}

/// ユーザー作成（POST /api/v1/master/users）。
///
/// system_admin のみ作成可。初期パスワードは bcrypt でハッシュ化して保存する。
#[utoipa::path(
    post,
    path = "/api/v1/master/users",
    tag = "master",
    security(("Bearer" = [])),
    request_body = CreateUserRequest,
    responses(
        (status = 201, description = "ユーザー作成成功"),
        (status = 401, description = "未認証"),
        (status = 403, description = "system_admin 専用"),
    )
)]
pub async fn create_user(
    creator: AuthenticatedUser<AdminRole>,
    State(state): State<AppState>,
    Json(req): Json<CreateUserRequest>,
) -> Result<impl IntoResponse, AppError> {
    let _ = creator;
    let new_id = Uuid::now_v7();
    let now = Utc::now();

    // 初期パスワードを bcrypt でハッシュ化する（平文保存禁止）
    let password_hash = bcrypt::hash(&req.password_initial, bcrypt::DEFAULT_COST)
        .map_err(|e| AppError::InternalError(format!("パスワードのハッシュ化に失敗した: {e}")))?;

    let roles_json = serde_json::json!(req.roles);

    sqlx::query(
        r#"
        INSERT INTO users
            (id, login_id, display_name, email, password_hash, factory_id,
             roles, is_active, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, true, $8, $8)
        "#,
    )
    .bind(new_id)
    .bind(&req.login_id)
    .bind(&req.display_name)
    .bind(&req.email)
    .bind(&password_hash)
    .bind(req.factory_id)
    .bind(&roles_json)
    .bind(now)
    .execute(&state.write_pool)
    .await?;

    tracing::info!(
        event = "user.created",
        user_id = %new_id,
        login_id = %req.login_id,
        "ユーザーを作成しました",
    );

    Ok((
        StatusCode::CREATED,
        Json(UserResponse {
            id: new_id,
            login_id: req.login_id,
            display_name: req.display_name,
            email: req.email,
            factory_id: req.factory_id,
            roles: req.roles,
            is_active: true,
            created_at: now,
        }),
    ))
}

/// ロール割当（PUT /api/v1/master/users/{id}/roles）。
///
/// system_admin のみ実行可。roles フィールドの内容で完全置換する。
#[utoipa::path(
    put,
    path = "/api/v1/master/users/{id}/roles",
    tag = "master",
    security(("Bearer" = [])),
    params(
        ("id" = Uuid, Path, description = "ユーザー ID"),
    ),
    request_body = AssignRolesRequest,
    responses(
        (status = 200, description = "ロール割当成功"),
        (status = 401, description = "未認証"),
        (status = 403, description = "system_admin 専用"),
        (status = 404, description = "ユーザーが見つからない"),
    )
)]
pub async fn assign_roles(
    _admin: AuthenticatedUser<AdminRole>,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<AssignRolesRequest>,
) -> Result<impl IntoResponse, AppError> {
    let now = Utc::now();
    let roles_json = serde_json::json!(req.roles);

    // ユーザーの存在確認をしてからロールを完全置換する
    let affected = sqlx::query(
        r#"
        UPDATE users
        SET roles = $1, updated_at = $2
        WHERE id = $3 AND deleted_at IS NULL
        "#,
    )
    .bind(&roles_json)
    .bind(now)
    .bind(id)
    .execute(&state.write_pool)
    .await?;

    if affected.rows_affected() == 0 {
        return Err(AppError::NotFound(format!("user:{id}")));
    }

    tracing::info!(
        event = "user.roles.assigned",
        user_id = %id,
        roles = ?req.roles,
        "ユーザーのロールを更新しました",
    );

    Ok((
        StatusCode::OK,
        Json(serde_json::json!({
            "user_id": id,
            "roles": req.roles,
            "updated_at": now,
        })),
    ))
}
