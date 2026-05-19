// 作業指示 API（API-work-orders-001〜002）ハンドラ（03_作業実行API仕様.md §1〜2）
//
// GET  /api/v1/work-orders      — 作業指示一覧取得
// GET  /api/v1/work-orders/{id} — 作業指示単件取得

use axum::{
    Extension, Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use chrono::Utc;
use uuid::Uuid;

use crate::{
    dto::{
        response_envelope::{PaginatedResponse, ApiResponse},
        work_orders::{CreateWorkOrderRequest, WorkOrderDto, WorkOrderQuery},
    },
    error::AppError,
    state::AppState,
};
use wnav_auth::CurrentUser;

/// GET /api/v1/work-orders — 作業指示一覧取得（API-work-orders-001）
#[utoipa::path(
    get,
    path = "/api/v1/work-orders",
    operation_id = "listWorkOrders",
    params(
        ("status" = Option<String>, Query, description = "ステータスフィルタ"),
        ("page" = Option<i64>, Query, description = "ページ番号"),
        ("per_page" = Option<i64>, Query, description = "1ページ件数"),
    ),
    responses(
        (status = 200, description = "作業指示一覧", body = PaginatedResponse<WorkOrderDto>),
        (status = 401, description = "認証エラー"),
    ),
    security(("bearer_auth" = [])),
    tag = "work-orders",
)]
pub async fn list_work_orders(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(query): Query<WorkOrderQuery>,
) -> Result<Json<PaginatedResponse<WorkOrderDto>>, AppError> {
    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(50).min(200).max(1);
    let offset = (page - 1) * per_page;

    // read_pool で作業指示一覧を取得する
    let rows = sqlx::query_as::<_, (Uuid, String, String, Uuid, String, Uuid, String, Option<Uuid>, Option<String>, Option<Uuid>, Option<String>, Option<chrono::DateTime<Utc>>, Option<chrono::DateTime<Utc>>, Option<Uuid>, chrono::DateTime<Utc>, chrono::DateTime<Utc>)>(
        r"
        SELECT
            wo.id,
            wo.work_order_number,
            wo.status,
            wo.process_id,
            COALESCE(p.name_json ->> 'ja', '') AS process_name,
            wo.sop_id,
            COALESCE(s.version, '') AS sop_version,
            wo.lot_id,
            l.lot_number,
            wo.product_id,
            COALESCE(pr.name_json ->> 'ja', '') AS product_name,
            wo.scheduled_start,
            wo.scheduled_end,
            wo.assigned_to,
            wo.created_at,
            wo.updated_at
        FROM work_orders wo
        LEFT JOIN processes p ON p.id = wo.process_id
        LEFT JOIN sops s ON s.id = wo.sop_id
        LEFT JOIN lots l ON l.id = wo.lot_id
        LEFT JOIN products pr ON pr.id = wo.product_id
        WHERE wo.factory_id = $1
          AND wo.deleted_at IS NULL
        ORDER BY wo.created_at DESC
        LIMIT $2 OFFSET $3
        ",
    )
    .bind(current_user.factory_id)
    .bind(per_page)
    .bind(offset)
    .fetch_all(&state.read_pool)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "work_orders 一覧取得に失敗した");
        AppError::DatabaseError
    })?;

    let items: Vec<WorkOrderDto> = rows
        .into_iter()
        .map(|(id, work_order_number, status, process_id, process_name, sop_id, sop_version, lot_id, lot_number, product_id, product_name, scheduled_start, scheduled_end, assigned_to, created_at, updated_at)| {
            WorkOrderDto {
                id,
                work_order_number,
                status,
                process_id,
                process_name,
                sop_id,
                sop_version,
                lot_id,
                lot_number,
                product_id,
                product_name,
                scheduled_start,
                scheduled_end,
                assigned_to,
                created_at,
                updated_at,
            }
        })
        .collect();

    // 全件数を取得する
    let total: i64 = sqlx::query_as::<_, (i64,)>(
        r"
        SELECT COUNT(*) FROM work_orders
        WHERE factory_id = $1 AND deleted_at IS NULL
        ",
    )
    .bind(current_user.factory_id)
    .fetch_one(&state.read_pool)
    .await
    .map(|(c,)| c)
    .unwrap_or(0);

    Ok(Json(PaginatedResponse::new(items, total, page, per_page)))
}

/// GET /api/v1/work-orders/{id} — 作業指示単件取得（API-work-orders-002）
#[utoipa::path(
    get,
    path = "/api/v1/work-orders/{id}",
    operation_id = "getWorkOrder",
    params(
        ("id" = Uuid, Path, description = "作業指示 ID"),
    ),
    responses(
        (status = 200, description = "作業指示取得成功", body = ApiResponse<WorkOrderDto>),
        (status = 404, description = "見つからない"),
    ),
    security(("bearer_auth" = [])),
    tag = "work-orders",
)]
pub async fn get_work_order(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<WorkOrderDto>>, AppError> {
    let row = sqlx::query_as::<_, (Uuid, String, String, Uuid, String, Uuid, String, Option<Uuid>, Option<String>, Option<Uuid>, Option<String>, Option<chrono::DateTime<Utc>>, Option<chrono::DateTime<Utc>>, Option<Uuid>, chrono::DateTime<Utc>, chrono::DateTime<Utc>)>(
        r"
        SELECT
            wo.id,
            wo.work_order_number,
            wo.status,
            wo.process_id,
            COALESCE(p.name_json ->> 'ja', '') AS process_name,
            wo.sop_id,
            COALESCE(s.version, '') AS sop_version,
            wo.lot_id,
            l.lot_number,
            wo.product_id,
            COALESCE(pr.name_json ->> 'ja', '') AS product_name,
            wo.scheduled_start,
            wo.scheduled_end,
            wo.assigned_to,
            wo.created_at,
            wo.updated_at
        FROM work_orders wo
        LEFT JOIN processes p ON p.id = wo.process_id
        LEFT JOIN sops s ON s.id = wo.sop_id
        LEFT JOIN lots l ON l.id = wo.lot_id
        LEFT JOIN products pr ON pr.id = wo.product_id
        WHERE wo.id = $1
          AND wo.factory_id = $2
          AND wo.deleted_at IS NULL
        LIMIT 1
        ",
    )
    .bind(id)
    .bind(current_user.factory_id)
    .fetch_optional(&state.read_pool)
    .await
    .map_err(|_| AppError::DatabaseError)?;

    let Some((id, work_order_number, status, process_id, process_name, sop_id, sop_version, lot_id, lot_number, product_id, product_name, scheduled_start, scheduled_end, assigned_to, created_at, updated_at)) = row else {
        return Err(AppError::NotFound);
    };

    let dto = WorkOrderDto {
        id,
        work_order_number,
        status,
        process_id,
        process_name,
        sop_id,
        sop_version,
        lot_id,
        lot_number,
        product_id,
        product_name,
        scheduled_start,
        scheduled_end,
        assigned_to,
        created_at,
        updated_at,
    };

    Ok(Json(ApiResponse::new(dto)))
}

/// POST /api/v1/work-orders — 作業指示作成（API-work-orders-002）
///
/// supervisor / master_admin / system_admin のみ作成可
#[utoipa::path(
    post,
    path = "/api/v1/work-orders",
    operation_id = "createWorkOrder",
    request_body = CreateWorkOrderRequest,
    responses(
        (status = 201, description = "作業指示作成成功", body = ApiResponse<WorkOrderDto>),
        (status = 403, description = "権限不足"),
        (status = 409, description = "業務ルール違反"),
        (status = 422, description = "バリデーションエラー"),
    ),
    security(("bearer_auth" = [])),
    tag = "work-orders",
)]
pub async fn create_work_order(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(body): Json<CreateWorkOrderRequest>,
) -> Result<(StatusCode, Json<ApiResponse<WorkOrderDto>>), AppError> {
    let _server_received_at = Utc::now().timestamp_millis();

    // RBAC: supervisor / master_admin / system_admin のみ作成可
    let has_permission = current_user.roles.iter().any(|r| {
        matches!(r.as_str(), "supervisor" | "master_admin" | "system_admin")
    });
    if !has_permission {
        return Err(AppError::Forbidden);
    }

    let new_id = Uuid::now_v7();
    let now = Utc::now();

    sqlx::query(
        r"
        INSERT INTO work_orders
            (id, work_order_number, status, process_id, sop_id, lot_id, product_id,
             scheduled_start, scheduled_end, assigned_to, factory_id, created_at, updated_at)
        VALUES ($1, $2, 'open', $3, $4, $5, $6, $7, $8, $9, $10, $11, $11)
        ",
    )
    .bind(new_id)
    .bind(&body.work_order_number)
    .bind(body.process_id)
    .bind(body.sop_id)
    .bind(body.lot_id)
    .bind(body.product_id)
    .bind(body.scheduled_start)
    .bind(body.scheduled_end)
    .bind(body.assigned_to)
    .bind(current_user.factory_id)
    .bind(now)
    .execute(&state.event_insert_pool)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "work_orders INSERT に失敗した");
        AppError::DatabaseError
    })?;

    let dto = WorkOrderDto {
        id: new_id,
        work_order_number: body.work_order_number,
        status: "open".to_string(),
        process_id: body.process_id,
        process_name: String::new(),
        sop_id: body.sop_id,
        sop_version: String::new(),
        lot_id: Some(body.lot_id),
        lot_number: None,
        product_id: Some(body.product_id),
        product_name: None,
        scheduled_start: Some(body.scheduled_start),
        scheduled_end: Some(body.scheduled_end),
        assigned_to: body.assigned_to,
        created_at: now,
        updated_at: now,
    };

    Ok((StatusCode::CREATED, Json(ApiResponse::new(dto))))
}
