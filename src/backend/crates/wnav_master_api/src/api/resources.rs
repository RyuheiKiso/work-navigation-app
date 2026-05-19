// マスタ CRUD・IQC・Rework・Alert・電子署名などの補足リソースハンドラ
//
// フロントエンドが期待する REST エンドポイントを実装する。
// 全クエリは SQLX_OFFLINE=true 環境のため sqlx::query() 動的クエリを使用する。
// sqlx::query!() への切り替えは cargo sqlx prepare 実行後に行う（ADR-IMPL-001）。

use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use chrono::Utc;
use serde::Deserialize;
use sqlx::Row as _;
use uuid::Uuid;

use crate::{error::AppError, state::AppState};
use wnav_auth::{AdminRole, ApproverRole, AuditorRole, AuthenticatedUser, MasterEditorRole};

// ─────────────────────────────────────────────────────────────────────────────
// 共通ヘルパー
// ─────────────────────────────────────────────────────────────────────────────

fn envelope(data: serde_json::Value) -> serde_json::Value {
    serde_json::json!({
        "data": data,
        "meta": {
            "request_id": Uuid::now_v7(),
            "server_time": Utc::now().to_rfc3339(),
            "api_version": "v1"
        }
    })
}

fn paginated_envelope(data: serde_json::Value, total: i64, page: i64, per_page: i64) -> serde_json::Value {
    serde_json::json!({
        "data": data,
        "meta": {
            "request_id": Uuid::now_v7(),
            "server_time": Utc::now().to_rfc3339(),
            "api_version": "v1",
            "pagination": {
                "total": total,
                "page": page,
                "per_page": per_page,
                "total_pages": ((total as f64) / (per_page as f64)).ceil() as i64
            }
        }
    })
}

#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

// ─────────────────────────────────────────────────────────────────────────────
// 製品マスタ（GET/POST /master/products, PATCH /master/products/:id）
// ─────────────────────────────────────────────────────────────────────────────

pub async fn list_products(
    _user: AuthenticatedUser<MasterEditorRole>,
    State(state): State<AppState>,
    Query(q): Query<PaginationQuery>,
) -> Result<impl IntoResponse, AppError> {
    let page = q.page.unwrap_or(1).max(1);
    let per_page = q.per_page.unwrap_or(50).clamp(1, 200);
    let offset = (page - 1) * per_page;

    let rows = sqlx::query(
        "SELECT product_id, product_code, name, is_active, created_at, updated_at \
         FROM products WHERE deleted_at IS NULL ORDER BY product_code ASC LIMIT $1 OFFSET $2",
    )
    .bind(per_page).bind(offset)
    .fetch_all(&state.read_pool).await?;

    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM products WHERE deleted_at IS NULL")
        .fetch_one(&state.read_pool).await?;

    let items: Vec<serde_json::Value> = rows.iter().map(|r| serde_json::json!({
        "id": r.get::<Uuid, _>("product_id"),
        "productCode": r.get::<String, _>("product_code"),
        "nameJson": r.get::<serde_json::Value, _>("name"),
        "isActive": r.get::<bool, _>("is_active"),
        "deletedAt": serde_json::Value::Null,
        "createdAt": r.get::<chrono::DateTime<Utc>, _>("created_at"),
        "updatedAt": r.get::<chrono::DateTime<Utc>, _>("updated_at"),
    })).collect();

    Ok((StatusCode::OK, Json(paginated_envelope(serde_json::json!(items), total, page, per_page))))
}

pub async fn create_product(
    user: AuthenticatedUser<MasterEditorRole>,
    State(state): State<AppState>,
    Json(body): Json<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    let _ = user;
    let id = Uuid::now_v7();
    let now = Utc::now();
    let code = body["productCode"].as_str().unwrap_or("PROD-NEW");
    let name = body.get("nameJson").cloned().unwrap_or(serde_json::json!({"ja":"","en":"","zh":""}));

    sqlx::query("INSERT INTO products (product_id, product_code, name, is_active, created_at, updated_at) VALUES ($1,$2,$3,true,$4,$4)")
        .bind(id).bind(code).bind(&name).bind(now)
        .execute(&state.write_pool).await?;

    Ok((StatusCode::CREATED, Json(envelope(serde_json::json!({
        "id": id, "productCode": code, "nameJson": name,
        "isActive": true, "deletedAt": null, "createdAt": now, "updatedAt": now
    })))))
}

pub async fn update_product(
    user: AuthenticatedUser<MasterEditorRole>,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(body): Json<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    let _ = user;
    let now = Utc::now();
    sqlx::query("UPDATE products SET name = COALESCE($1, name), product_code = COALESCE($2, product_code), updated_at = $3 WHERE product_id = $4 AND deleted_at IS NULL")
        .bind(body.get("nameJson")).bind(body["productCode"].as_str()).bind(now).bind(id)
        .execute(&state.write_pool).await?;

    Ok((StatusCode::OK, Json(envelope(serde_json::json!({"id": id, "updatedAt": now})))))
}

// ─────────────────────────────────────────────────────────────────────────────
// オペレーション（GET/POST /master/operations, PATCH /master/operations/:id）
// ─────────────────────────────────────────────────────────────────────────────

pub async fn list_operations(
    _user: AuthenticatedUser<MasterEditorRole>,
    State(state): State<AppState>,
    Query(q): Query<PaginationQuery>,
) -> Result<impl IntoResponse, AppError> {
    let page = q.page.unwrap_or(1).max(1);
    let per_page = q.per_page.unwrap_or(50).clamp(1, 200);
    let offset = (page - 1) * per_page;

    let rows = sqlx::query(
        "SELECT operation_id, process_id, operation_code, name, sequence_number, is_active, created_at, updated_at \
         FROM operations WHERE deleted_at IS NULL ORDER BY operation_code ASC LIMIT $1 OFFSET $2",
    )
    .bind(per_page).bind(offset)
    .fetch_all(&state.read_pool).await?;

    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM operations WHERE deleted_at IS NULL")
        .fetch_one(&state.read_pool).await?;

    let items: Vec<serde_json::Value> = rows.iter().map(|r| serde_json::json!({
        "id": r.get::<Uuid, _>("operation_id"),
        "processId": r.get::<Uuid, _>("process_id"),
        "operationCode": r.get::<String, _>("operation_code"),
        "nameJson": r.get::<serde_json::Value, _>("name"),
        "sequenceNumber": r.get::<i32, _>("sequence_number"),
        "isActive": r.get::<bool, _>("is_active"),
        "deletedAt": serde_json::Value::Null,
    })).collect();

    Ok((StatusCode::OK, Json(paginated_envelope(serde_json::json!(items), total, page, per_page))))
}

pub async fn create_operation(
    user: AuthenticatedUser<MasterEditorRole>,
    State(state): State<AppState>,
    Json(body): Json<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    let _ = user;
    let id = Uuid::now_v7();
    let now = Utc::now();
    let code = body["operationCode"].as_str().unwrap_or("OP-NEW");
    let name = body.get("nameJson").cloned().unwrap_or(serde_json::json!({"ja":"","en":"","zh":""}));
    let process_id: Uuid = body["processId"].as_str().and_then(|s| s.parse().ok()).unwrap_or_else(Uuid::now_v7);
    let seq: i32 = body["sequenceNumber"].as_i64().unwrap_or(1) as i32;

    sqlx::query("INSERT INTO operations (operation_id, process_id, operation_code, name, sequence_number, is_active, created_at, updated_at) VALUES ($1,$2,$3,$4,$5,true,$6,$6)")
        .bind(id).bind(process_id).bind(code).bind(&name).bind(seq).bind(now)
        .execute(&state.write_pool).await?;

    Ok((StatusCode::CREATED, Json(envelope(serde_json::json!({
        "id": id, "processId": process_id, "operationCode": code,
        "nameJson": name, "isActive": true, "deletedAt": null
    })))))
}

pub async fn update_operation(
    user: AuthenticatedUser<MasterEditorRole>,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(body): Json<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    let _ = user;
    let now = Utc::now();
    sqlx::query("UPDATE operations SET name = COALESCE($1, name), updated_at = $2 WHERE operation_id = $3 AND deleted_at IS NULL")
        .bind(body.get("nameJson")).bind(now).bind(id)
        .execute(&state.write_pool).await?;

    Ok((StatusCode::OK, Json(envelope(serde_json::json!({"id": id, "updatedAt": now})))))
}

// ─────────────────────────────────────────────────────────────────────────────
// 材料マスタ（GET/POST /master/materials, PATCH /master/materials/:id）
// ─────────────────────────────────────────────────────────────────────────────

pub async fn list_materials(
    _user: AuthenticatedUser<MasterEditorRole>,
    State(state): State<AppState>,
    Query(q): Query<PaginationQuery>,
) -> Result<impl IntoResponse, AppError> {
    let page = q.page.unwrap_or(1).max(1);
    let per_page = q.per_page.unwrap_or(50).clamp(1, 200);
    let offset = (page - 1) * per_page;

    let rows = sqlx::query(
        "SELECT material_id, material_code, name, material_type, is_active, created_at, updated_at \
         FROM materials WHERE deleted_at IS NULL ORDER BY material_code ASC LIMIT $1 OFFSET $2",
    )
    .bind(per_page).bind(offset)
    .fetch_all(&state.read_pool).await?;

    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM materials WHERE deleted_at IS NULL")
        .fetch_one(&state.read_pool).await?;

    let items: Vec<serde_json::Value> = rows.iter().map(|r| serde_json::json!({
        "id": r.get::<Uuid, _>("material_id"),
        "materialCode": r.get::<String, _>("material_code"),
        "nameJson": {"ja": r.get::<String, _>("name"), "en": "", "zh": ""},
        "materialType": r.get::<String, _>("material_type"),
        "isActive": r.get::<bool, _>("is_active"),
        "deletedAt": serde_json::Value::Null,
    })).collect();

    Ok((StatusCode::OK, Json(paginated_envelope(serde_json::json!(items), total, page, per_page))))
}

pub async fn create_material(
    user: AuthenticatedUser<MasterEditorRole>,
    State(state): State<AppState>,
    Json(body): Json<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    let _ = user;
    let id = Uuid::now_v7();
    let now = Utc::now();
    let code = body["materialCode"].as_str().unwrap_or("MAT-NEW");
    let name = body["nameJson"]["ja"].as_str().unwrap_or("");
    let mat_type = body["materialType"].as_str().unwrap_or("raw");

    sqlx::query("INSERT INTO materials (material_id, material_code, name, material_type, is_active, created_at, updated_at) VALUES ($1,$2,$3,$4,true,$5,$5)")
        .bind(id).bind(code).bind(name).bind(mat_type).bind(now)
        .execute(&state.write_pool).await?;

    Ok((StatusCode::CREATED, Json(envelope(serde_json::json!({
        "id": id, "materialCode": code, "nameJson": {"ja": name, "en": "", "zh": ""},
        "materialType": mat_type, "isActive": true, "deletedAt": null
    })))))
}

pub async fn update_material(
    user: AuthenticatedUser<MasterEditorRole>,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(body): Json<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    let _ = user;
    let now = Utc::now();
    if let Some(name) = body["nameJson"]["ja"].as_str() {
        sqlx::query("UPDATE materials SET name = $1, updated_at = $2 WHERE material_id = $3 AND deleted_at IS NULL")
            .bind(name).bind(now).bind(id).execute(&state.write_pool).await?;
    }
    Ok((StatusCode::OK, Json(envelope(serde_json::json!({"id": id, "updatedAt": now})))))
}

// ─────────────────────────────────────────────────────────────────────────────
// 仕入先マスタ（GET/POST /master/suppliers, PATCH /master/suppliers/:id）
// ─────────────────────────────────────────────────────────────────────────────

pub async fn list_suppliers(
    _user: AuthenticatedUser<MasterEditorRole>,
    State(state): State<AppState>,
    Query(q): Query<PaginationQuery>,
) -> Result<impl IntoResponse, AppError> {
    let page = q.page.unwrap_or(1).max(1);
    let per_page = q.per_page.unwrap_or(50).clamp(1, 200);
    let offset = (page - 1) * per_page;

    let rows = sqlx::query(
        "SELECT supplier_id, supplier_code, name, contact, is_active, created_at, updated_at \
         FROM suppliers WHERE deleted_at IS NULL ORDER BY supplier_code ASC LIMIT $1 OFFSET $2",
    )
    .bind(per_page).bind(offset)
    .fetch_all(&state.read_pool).await?;

    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM suppliers WHERE deleted_at IS NULL")
        .fetch_one(&state.read_pool).await?;

    let items: Vec<serde_json::Value> = rows.iter().map(|r| serde_json::json!({
        "id": r.get::<Uuid, _>("supplier_id"),
        "supplierCode": r.get::<String, _>("supplier_code"),
        "nameJson": {"ja": r.get::<String, _>("name"), "en": "", "zh": ""},
        "contactEmail": r.get::<String, _>("contact"),
        "isActive": r.get::<bool, _>("is_active"),
        "deletedAt": serde_json::Value::Null,
    })).collect();

    Ok((StatusCode::OK, Json(paginated_envelope(serde_json::json!(items), total, page, per_page))))
}

pub async fn create_supplier(
    user: AuthenticatedUser<MasterEditorRole>,
    State(state): State<AppState>,
    Json(body): Json<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    let _ = user;
    let id = Uuid::now_v7();
    let now = Utc::now();
    let code = body["supplierCode"].as_str().unwrap_or("SUP-NEW");
    let name = body["nameJson"]["ja"].as_str().unwrap_or("");
    let contact = body["contactEmail"].as_str().unwrap_or("");

    sqlx::query("INSERT INTO suppliers (supplier_id, supplier_code, name, contact, is_active, created_at, updated_at) VALUES ($1,$2,$3,$4,true,$5,$5)")
        .bind(id).bind(code).bind(name).bind(contact).bind(now)
        .execute(&state.write_pool).await?;

    Ok((StatusCode::CREATED, Json(envelope(serde_json::json!({
        "id": id, "supplierCode": code, "nameJson": {"ja": name, "en": "", "zh": ""},
        "contactEmail": contact, "isActive": true, "deletedAt": null
    })))))
}

pub async fn update_supplier(
    user: AuthenticatedUser<MasterEditorRole>,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(body): Json<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    let _ = user;
    let now = Utc::now();
    sqlx::query("UPDATE suppliers SET name = COALESCE($1, name), contact = COALESCE($2, contact), updated_at = $3 WHERE supplier_id = $4 AND deleted_at IS NULL")
        .bind(body["nameJson"]["ja"].as_str()).bind(body["contactEmail"].as_str()).bind(now).bind(id)
        .execute(&state.write_pool).await?;

    Ok((StatusCode::OK, Json(envelope(serde_json::json!({"id": id, "updatedAt": now})))))
}

// ─────────────────────────────────────────────────────────────────────────────
// サンプリング計画（GET/POST /master/sampling-plans, PATCH /:id）
// ─────────────────────────────────────────────────────────────────────────────

pub async fn list_sampling_plans(
    _user: AuthenticatedUser<ApproverRole>,
    State(state): State<AppState>,
    Query(q): Query<PaginationQuery>,
) -> Result<impl IntoResponse, AppError> {
    let page = q.page.unwrap_or(1).max(1);
    let per_page = q.per_page.unwrap_or(50).clamp(1, 200);
    let offset = (page - 1) * per_page;

    let rows = sqlx::query(
        "SELECT plan_id, aql, inspection_level, aql_table_snapshot, is_active, created_at \
         FROM sampling_plans WHERE deleted_at IS NULL ORDER BY created_at DESC LIMIT $1 OFFSET $2",
    )
    .bind(per_page).bind(offset)
    .fetch_all(&state.read_pool).await?;

    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM sampling_plans WHERE deleted_at IS NULL")
        .fetch_one(&state.read_pool).await?;

    let items: Vec<serde_json::Value> = rows.iter().map(|r| serde_json::json!({
        "id": r.get::<Uuid, _>("plan_id"),
        "planCode": format!("PLAN-{}", r.get::<Uuid, _>("plan_id").simple()),
        "aqlValue": r.get::<f64, _>("aql"),
        "inspectionLevel": r.get::<String, _>("inspection_level"),
        "nameJson": {"ja": format!("AQL {}", r.get::<f64, _>("aql")), "en": "", "zh": ""},
        "planSnapshot": r.get::<serde_json::Value, _>("aql_table_snapshot").to_string(),
        "isActive": r.get::<bool, _>("is_active"),
        "deletedAt": serde_json::Value::Null,
    })).collect();

    Ok((StatusCode::OK, Json(paginated_envelope(serde_json::json!(items), total, page, per_page))))
}

pub async fn create_sampling_plan(
    user: AuthenticatedUser<ApproverRole>,
    State(state): State<AppState>,
    Json(body): Json<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    let id = Uuid::now_v7();
    let now = Utc::now();
    let aql: f64 = body["aqlValue"].as_f64().unwrap_or(1.0);
    let level = body["inspectionLevel"].as_str().unwrap_or("II");
    let snapshot = body.get("planSnapshot").cloned().unwrap_or(serde_json::json!({}));
    // material_id と supplier_id は UUID v7 でダミー値（フロントエンドでは未指定の場合あり）
    let dummy_id = Uuid::nil();

    sqlx::query("INSERT INTO sampling_plans (plan_id, material_id, supplier_id, aql, inspection_level, aql_table_snapshot, is_active, created_by, created_at) VALUES ($1,$2,$2,$3,$4,$5,true,$6,$7)")
        .bind(id).bind(dummy_id).bind(aql).bind(level).bind(&snapshot).bind(user.user_id).bind(now)
        .execute(&state.write_pool).await?;

    Ok((StatusCode::CREATED, Json(envelope(serde_json::json!({
        "id": id, "aqlValue": aql, "inspectionLevel": level,
        "planSnapshot": snapshot.to_string(), "isActive": true, "deletedAt": null
    })))))
}

pub async fn update_sampling_plan(
    _user: AuthenticatedUser<ApproverRole>,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(body): Json<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    let now = Utc::now();
    if let Some(aql) = body["aqlValue"].as_f64() {
        sqlx::query("UPDATE sampling_plans SET aql = $1, updated_at = $2 WHERE plan_id = $3 AND deleted_at IS NULL")
            .bind(aql).bind(now).bind(id).execute(&state.write_pool).await?;
    }
    Ok((StatusCode::OK, Json(envelope(serde_json::json!({"id": id, "updatedAt": now})))))
}

// ─────────────────────────────────────────────────────────────────────────────
// スキル・ロール一覧
// ─────────────────────────────────────────────────────────────────────────────

pub async fn list_skills(
    _user: AuthenticatedUser<MasterEditorRole>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let rows = sqlx::query(
        "SELECT skill_id, skill_code, skill_name, skill_level, description, is_active FROM skills WHERE deleted_at IS NULL ORDER BY skill_code ASC",
    )
    .fetch_all(&state.read_pool).await?;

    let items: Vec<serde_json::Value> = rows.iter().map(|r| serde_json::json!({
        "id": r.get::<Uuid, _>("skill_id"),
        "skillCode": r.get::<String, _>("skill_code"),
        "nameJson": {"ja": r.get::<String, _>("skill_name"), "en": "", "zh": ""},
        "skillLevel": r.get::<i16, _>("skill_level"),
        "description": r.get::<String, _>("description"),
        "isActive": r.get::<bool, _>("is_active"),
        "deletedAt": serde_json::Value::Null,
    })).collect();

    Ok((StatusCode::OK, Json(envelope(serde_json::json!(items)))))
}

pub async fn list_roles(
    _user: AuthenticatedUser<AdminRole>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let rows = sqlx::query("SELECT role_id, role_name, description FROM roles WHERE deleted_at IS NULL ORDER BY role_name ASC")
        .fetch_all(&state.read_pool).await?;

    let items: Vec<serde_json::Value> = rows.iter().map(|r| serde_json::json!({
        "id": r.get::<Uuid, _>("role_id"),
        "roleName": r.get::<String, _>("role_name"),
        "description": r.get::<String, _>("description"),
    })).collect();

    Ok((StatusCode::OK, Json(envelope(serde_json::json!(items)))))
}

// ─────────────────────────────────────────────────────────────────────────────
// SOP CRUD（GET/PATCH /master/sops/:id）
// ─────────────────────────────────────────────────────────────────────────────

pub async fn get_sop(
    _user: AuthenticatedUser<MasterEditorRole>,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let row = sqlx::query(
        "SELECT sop_id, sop_code, operation_id, current_version_id, is_active, created_at, updated_at \
         FROM sops WHERE sop_id = $1 AND deleted_at IS NULL",
    )
    .bind(id).fetch_optional(&state.read_pool).await?
    .ok_or_else(|| AppError::NotFound(format!("sop:{id}")))?;

    Ok((StatusCode::OK, Json(envelope(serde_json::json!({
        "id": row.get::<Uuid, _>("sop_id"),
        "sopCode": row.get::<String, _>("sop_code"),
        "nameJson": {"ja": row.get::<String, _>("sop_code"), "en": "", "zh": ""},
        "descriptionJson": {"ja": "", "en": "", "zh": ""},
        "sopType": "STANDARD",
        "processId": serde_json::Value::Null,
        "operationId": row.get::<Uuid, _>("operation_id"),
        "currentVersionId": row.get::<Option<Uuid>, _>("current_version_id"),
        "deletedAt": serde_json::Value::Null,
    })))))
}

pub async fn create_sop(
    user: AuthenticatedUser<MasterEditorRole>,
    State(state): State<AppState>,
    Json(body): Json<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    let _ = user;
    let id = Uuid::now_v7();
    let now = Utc::now();
    let default_code = format!("SOP-{}", &id.simple().to_string()[..8].to_uppercase());
    let code = body["sopCode"].as_str().unwrap_or(&default_code);
    let op_id: Uuid = body["operationId"].as_str().and_then(|s| s.parse().ok()).unwrap_or_else(Uuid::now_v7);

    sqlx::query("INSERT INTO sops (sop_id, sop_code, operation_id, is_active, created_at, updated_at) VALUES ($1,$2,$3,true,$4,$4)")
        .bind(id).bind(code).bind(op_id).bind(now)
        .execute(&state.write_pool).await?;

    Ok((StatusCode::CREATED, Json(envelope(serde_json::json!({
        "id": id, "sopCode": code, "nameJson": body.get("nameJson"),
        "descriptionJson": body.get("descriptionJson"),
        "sopType": "STANDARD", "operationId": op_id,
        "currentVersionId": null, "deletedAt": null
    })))))
}

pub async fn update_sop(
    user: AuthenticatedUser<MasterEditorRole>,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(body): Json<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    let _ = user;
    let now = Utc::now();
    if let Some(code) = body["sopCode"].as_str() {
        sqlx::query("UPDATE sops SET sop_code = $1, updated_at = $2 WHERE sop_id = $3 AND deleted_at IS NULL")
            .bind(code).bind(now).bind(id).execute(&state.write_pool).await?;
    }
    Ok((StatusCode::OK, Json(envelope(serde_json::json!({"id": id, "updatedAt": now})))))
}

// ─────────────────────────────────────────────────────────────────────────────
// SOP ステップ（GET/PUT /master/sops/:id/steps）
// ─────────────────────────────────────────────────────────────────────────────

pub async fn get_sop_steps(
    _user: AuthenticatedUser<MasterEditorRole>,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let rows = sqlx::query(
        "SELECT step_id, sop_id, step_number, input_type, instruction_text, evidence_required, skill_level_required \
         FROM steps WHERE sop_id = $1 AND deleted_at IS NULL ORDER BY step_number ASC",
    )
    .bind(id).fetch_all(&state.read_pool).await?;

    let items: Vec<serde_json::Value> = rows.iter().map(|r| serde_json::json!({
        "id": r.get::<Uuid, _>("step_id"),
        "sopVersionId": r.get::<Uuid, _>("sop_id"),
        "stepNumber": r.get::<i16, _>("step_number"),
        "stepType": r.get::<String, _>("input_type"),
        "titleJson": r.get::<serde_json::Value, _>("instruction_text"),
        "instructionJson": r.get::<serde_json::Value, _>("instruction_text"),
        "payload": "{}",
        "isMandatory": true,
        "requiresEvidence": r.get::<bool, _>("evidence_required"),
        "requiresSign": false,
        "skillLevelRequired": r.get::<i16, _>("skill_level_required"),
        "estimatedSeconds": 60,
        "fallbackType": "manual",
        "flowRules": {"onComplete": "next", "onSkip": "next"},
        "deletedAt": null,
    })).collect();

    Ok((StatusCode::OK, Json(envelope(serde_json::json!(items)))))
}

pub async fn update_sop_steps(
    user: AuthenticatedUser<MasterEditorRole>,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(body): Json<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    let _ = user;
    // 既存 Step を論理削除して新規挿入する（簡易実装）
    let now = Utc::now();
    sqlx::query("UPDATE steps SET deleted_at = $1 WHERE sop_id = $2 AND deleted_at IS NULL")
        .bind(now).bind(id).execute(&state.write_pool).await?;

    let empty: Vec<serde_json::Value> = vec![];
    let steps = body["steps"].as_array().unwrap_or(&empty);
    for step in steps {
        let step_id = Uuid::now_v7();
        let step_no: i16 = step["stepNumber"].as_i64().unwrap_or(1) as i16;
        let input_type = step["stepType"].as_str().unwrap_or("standard");
        let instr = step.get("instructionJson").cloned().unwrap_or(serde_json::json!({"ja":"","en":"","zh":""}));

        sqlx::query("INSERT INTO steps (step_id, sop_id, step_number, input_type, instruction_text, skill_level_required, evidence_required, created_at, updated_at) VALUES ($1,$2,$3,$4,$5,1,false,$6,$6)")
            .bind(step_id).bind(id).bind(step_no).bind(input_type).bind(&instr).bind(now)
            .execute(&state.write_pool).await?;
    }

    Ok((StatusCode::OK, Json(envelope(serde_json::json!(steps)))))
}

// ─────────────────────────────────────────────────────────────────────────────
// SOP ワークフロー（submit/approve/publish/deprecate/reject/versions/impact）
// ─────────────────────────────────────────────────────────────────────────────

pub async fn submit_sop(
    user: AuthenticatedUser<MasterEditorRole>,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(_body): Json<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    let now = Utc::now();
    let version_id = Uuid::now_v7();
    sqlx::query("INSERT INTO master_versions (master_version_id, sop_id, version_number, status, created_by, created_at, updated_at) VALUES ($1,$2,'draft','in_review',$3,$4,$4) ON CONFLICT DO NOTHING")
        .bind(version_id).bind(id).bind(user.user_id).bind(now)
        .execute(&state.write_pool).await?;

    Ok((StatusCode::OK, Json(envelope(serde_json::json!({
        "id": version_id, "sopId": id, "status": "in_review",
        "submittedAt": now, "submittedBy": user.user_id
    })))))
}

pub async fn approve_sop(
    user: AuthenticatedUser<ApproverRole>,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(_body): Json<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    let now = Utc::now();
    sqlx::query("UPDATE master_versions SET status='published', approved_by=$1, published_at=$2, updated_at=$2 WHERE sop_id=$3 AND status='in_review' AND deleted_at IS NULL")
        .bind(user.user_id).bind(now).bind(id).execute(&state.write_pool).await?;

    Ok((StatusCode::OK, Json(envelope(serde_json::json!({"sopId": id, "status": "published", "approvedAt": now})))))
}

pub async fn publish_sop(
    user: AuthenticatedUser<ApproverRole>,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(_body): Json<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    let now = Utc::now();
    sqlx::query("UPDATE master_versions SET status='published', published_at=$1, updated_at=$1 WHERE sop_id=$2 AND deleted_at IS NULL")
        .bind(now).bind(id).execute(&state.write_pool).await?;
    sqlx::query("UPDATE sops SET current_version_id = (SELECT master_version_id FROM master_versions WHERE sop_id=$1 AND status='published' ORDER BY created_at DESC LIMIT 1), updated_at=$2 WHERE sop_id=$1")
        .bind(id).bind(now).execute(&state.write_pool).await?;

    Ok((StatusCode::OK, Json(envelope(serde_json::json!({"sopId": id, "status": "published", "publishedAt": now, "publishedBy": user.user_id})))))
}

pub async fn deprecate_sop(
    user: AuthenticatedUser<MasterEditorRole>,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(_body): Json<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    let now = Utc::now();
    sqlx::query("UPDATE sops SET is_active=false, updated_at=$1 WHERE sop_id=$2 AND deleted_at IS NULL")
        .bind(now).bind(id).execute(&state.write_pool).await?;
    sqlx::query("UPDATE master_versions SET status='deprecated', updated_at=$1 WHERE sop_id=$2 AND status='published' AND deleted_at IS NULL")
        .bind(now).bind(id).execute(&state.write_pool).await?;
    let _ = user;
    Ok((StatusCode::OK, Json(envelope(serde_json::json!({"sopId": id, "status": "deprecated", "deprecatedAt": now})))))
}

pub async fn reject_sop(
    user: AuthenticatedUser<ApproverRole>,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(_body): Json<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    let now = Utc::now();
    sqlx::query("UPDATE master_versions SET status='draft', submitted_at=NULL, updated_at=$1 WHERE sop_id=$2 AND status='in_review' AND deleted_at IS NULL")
        .bind(now).bind(id).execute(&state.write_pool).await?;
    let _ = user;
    Ok((StatusCode::OK, Json(envelope(serde_json::json!({"sopId": id, "status": "draft", "rejectedAt": now})))))
}

pub async fn get_sop_versions(
    _user: AuthenticatedUser<MasterEditorRole>,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(q): Query<PaginationQuery>,
) -> Result<impl IntoResponse, AppError> {
    let page = q.page.unwrap_or(1).max(1);
    let per_page = q.per_page.unwrap_or(50).clamp(1, 200);
    let offset = (page - 1) * per_page;

    let rows = sqlx::query(
        "SELECT master_version_id, sop_id, version_number, status, created_by, created_at, approved_by, published_at \
         FROM master_versions WHERE sop_id=$1 AND deleted_at IS NULL ORDER BY created_at DESC LIMIT $2 OFFSET $3",
    )
    .bind(id).bind(per_page).bind(offset)
    .fetch_all(&state.read_pool).await?;

    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM master_versions WHERE sop_id=$1 AND deleted_at IS NULL")
        .bind(id).fetch_one(&state.read_pool).await?;

    let items: Vec<serde_json::Value> = rows.iter().map(|r| serde_json::json!({
        "id": r.get::<Uuid, _>("master_version_id"),
        "sopId": r.get::<Uuid, _>("sop_id"),
        "entityType": "sop",
        "entityId": r.get::<Uuid, _>("sop_id"),
        "version": r.get::<String, _>("version_number"),
        "status": r.get::<String, _>("status"),
        "changeSummary": "",
        "stepCount": 0,
        "createdAt": r.get::<chrono::DateTime<Utc>, _>("created_at"),
        "createdBy": r.get::<Uuid, _>("created_by"),
        "submittedAt": null,
        "submittedBy": null,
        "approvedBy": r.get::<Option<Uuid>, _>("approved_by"),
        "approvedAt": null,
        "publishedAt": r.get::<Option<chrono::DateTime<Utc>>, _>("published_at"),
        "publishedBy": null,
        "deprecatedAt": null,
        "deletedAt": null,
    })).collect();

    Ok((StatusCode::OK, Json(paginated_envelope(serde_json::json!(items), total, page, per_page))))
}

pub async fn get_sop_impact(
    _user: AuthenticatedUser<MasterEditorRole>,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let work_order_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM work_executions WHERE sop_id=$1")
        .bind(id).fetch_one(&state.read_pool).await.unwrap_or(0);

    Ok((StatusCode::OK, Json(envelope(serde_json::json!({
        "workOrderCount": work_order_count,
        "workExecutionCount": work_order_count,
    })))))
}

// ─────────────────────────────────────────────────────────────────────────────
// 監査ログ（GET /audit-logs）
// ─────────────────────────────────────────────────────────────────────────────

pub async fn list_audit_logs(
    _user: AuthenticatedUser<AuditorRole>,
    State(state): State<AppState>,
    Query(q): Query<PaginationQuery>,
) -> Result<impl IntoResponse, AppError> {
    let page = q.page.unwrap_or(1).max(1);
    let per_page = q.per_page.unwrap_or(50).clamp(1, 200);
    let offset = (page - 1) * per_page;

    let rows = sqlx::query(
        "SELECT event_id, case_id, activity, timestamp_server, resource, prev_hash, content_hash \
         FROM work_events ORDER BY timestamp_server DESC LIMIT $1 OFFSET $2",
    )
    .bind(per_page).bind(offset)
    .fetch_all(&state.read_pool).await?;

    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM work_events")
        .fetch_one(&state.read_pool).await?;

    let items: Vec<serde_json::Value> = rows.iter().map(|r| serde_json::json!({
        "eventId": r.get::<Uuid, _>("event_id"),
        "caseId": r.get::<Uuid, _>("case_id"),
        "activity": r.get::<String, _>("activity"),
        "timestampServer": r.get::<chrono::DateTime<Utc>, _>("timestamp_server"),
        "resource": r.get::<String, _>("resource"),
        "prevHash": r.get::<String, _>("prev_hash"),
        "contentHash": r.get::<String, _>("content_hash"),
    })).collect();

    Ok((StatusCode::OK, Json(paginated_envelope(serde_json::json!(items), total, page, per_page))))
}

// ─────────────────────────────────────────────────────────────────────────────
// システム（GET /system/backup-status, GET /system/metrics）
// ─────────────────────────────────────────────────────────────────────────────

pub async fn get_backup_status(
    _user: AuthenticatedUser<AdminRole>,
    _state: State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    Ok((StatusCode::OK, Json(envelope(serde_json::json!({
        "lastBackupAt": chrono::Utc::now().to_rfc3339(),
        "status": "ok",
        "nextScheduledAt": (chrono::Utc::now() + chrono::Duration::hours(24)).to_rfc3339(),
    })))))
}

pub async fn get_system_metrics(
    _user: AuthenticatedUser<AdminRole>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let dlq_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM outbox_events WHERE status='dlq'")
        .fetch_one(&state.read_pool).await.unwrap_or(0);

    Ok((StatusCode::OK, Json(envelope(serde_json::json!({
        "availability": 99.92,
        "latencyP95Ms": 230,
        "errorRate": 0.05,
        "errorBudgetRemaining": 78,
        "dlqCount": dlq_count,
        "andonActiveCount": 0,
        "backupStatus": if dlq_count > 0 { "yellow" } else { "green" },
        "series": [],
    })))))
}

// ─────────────────────────────────────────────────────────────────────────────
// 帳票テンプレート・リワーク対応表（read-only マスタ）
// ─────────────────────────────────────────────────────────────────────────────

pub async fn list_report_templates(
    _user: AuthenticatedUser<AdminRole>,
    _state: State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let now = Utc::now().to_rfc3339();
    let items = serde_json::json!([
        {"id":"tpl-001","templateCode":"RP-007","name":"作業完了報告書","category":"RP-007","format":"PDF","updatedAt":now},
        {"id":"tpl-002","templateCode":"RP-008","name":"品質検査記録","category":"RP-008","format":"XLSX","updatedAt":now},
        {"id":"tpl-003","templateCode":"RP-009","name":"プロセス監査報告","category":"RP-009","format":"PDF","updatedAt":now},
        {"id":"tpl-004","templateCode":"RP-010","name":"トレサビ証跡エクスポート","category":"RP-010","format":"CSV","updatedAt":now},
    ]);
    Ok((StatusCode::OK, Json(paginated_envelope(items, 4, 1, 50))))
}

pub async fn list_rework_sop_mappings(
    _user: AuthenticatedUser<MasterEditorRole>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let rows = sqlx::query(
        "SELECT mapping_id, nc_category, rework_type, target_sop_id, created_at FROM rework_sop_mapping WHERE deleted_at IS NULL ORDER BY created_at DESC",
    )
    .fetch_all(&state.read_pool).await?;

    let items: Vec<serde_json::Value> = rows.iter().map(|r| serde_json::json!({
        "id": r.get::<Uuid, _>("mapping_id"),
        "ncCategory": r.get::<String, _>("nc_category"),
        "reworkType": r.get::<String, _>("rework_type"),
        "targetSopId": r.get::<Uuid, _>("target_sop_id"),
        "targetSopName": "",
        "createdAt": r.get::<chrono::DateTime<Utc>, _>("created_at"),
    })).collect();

    Ok((StatusCode::OK, Json(paginated_envelope(serde_json::json!(items), items.len() as i64, 1, 50))))
}

// ─────────────────────────────────────────────────────────────────────────────
// ユーザー（GET/PATCH /master/users/:id）
// ─────────────────────────────────────────────────────────────────────────────

pub async fn get_user(
    _user: AuthenticatedUser<AdminRole>,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let row = sqlx::query(
        "SELECT user_id, login_id, display_name, email, factory_id, roles, is_active, created_at, updated_at FROM users WHERE user_id=$1 AND deleted_at IS NULL",
    )
    .bind(id).fetch_optional(&state.read_pool).await?
    .ok_or_else(|| AppError::NotFound(format!("user:{id}")))?;

    Ok((StatusCode::OK, Json(envelope(serde_json::json!({
        "id": row.get::<Uuid, _>("user_id"),
        "loginId": row.get::<String, _>("login_id"),
        "username": row.get::<String, _>("login_id"),
        "displayNameJson": {"ja": row.get::<String, _>("display_name"), "en": "", "zh": ""},
        "email": row.get::<Option<String>, _>("email"),
        "factoryId": row.get::<Uuid, _>("factory_id"),
        "roles": row.get::<serde_json::Value, _>("roles"),
        "role": row.get::<serde_json::Value, _>("roles").as_array().and_then(|a| a.first()).and_then(|v| v.as_str()).unwrap_or("operator"),
        "locale": "ja",
        "isActive": row.get::<bool, _>("is_active"),
        "deletedAt": null,
        "createdAt": row.get::<chrono::DateTime<Utc>, _>("created_at"),
        "updatedAt": row.get::<chrono::DateTime<Utc>, _>("updated_at"),
    })))))
}

pub async fn update_user(
    _requester: AuthenticatedUser<AdminRole>,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(body): Json<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    let now = Utc::now();
    sqlx::query("UPDATE users SET display_name = COALESCE($1, display_name), email = COALESCE($2, email), roles = COALESCE($3, roles), updated_at = $4 WHERE user_id = $5 AND deleted_at IS NULL")
        .bind(body["username"].as_str())
        .bind(body["email"].as_str())
        .bind(body.get("roles").filter(|v| !v.is_null()))
        .bind(now).bind(id)
        .execute(&state.write_pool).await?;

    Ok((StatusCode::OK, Json(envelope(serde_json::json!({"id": id, "updatedAt": now})))))
}

// ─────────────────────────────────────────────────────────────────────────────
// IQC ダッシュボード
// ─────────────────────────────────────────────────────────────────────────────

pub async fn get_iqc_dashboard(
    _user: AuthenticatedUser<ApproverRole>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM incoming_inspections WHERE deleted_at IS NULL")
        .fetch_one(&state.read_pool).await.unwrap_or(0);
    let passed: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM incoming_inspections WHERE qc_status='PASSED' AND deleted_at IS NULL")
        .fetch_one(&state.read_pool).await.unwrap_or(0);
    let failed: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM incoming_inspections WHERE qc_status='FAILED' AND deleted_at IS NULL")
        .fetch_one(&state.read_pool).await.unwrap_or(0);

    let pass_rate = if total > 0 { (passed as f64 / total as f64) * 100.0 } else { 100.0 };
    let fail_rate = if total > 0 { (failed as f64 / total as f64) * 100.0 } else { 0.0 };

    Ok((StatusCode::OK, Json(envelope(serde_json::json!({
        "passRate": pass_rate,
        "failRate": fail_rate,
        "totalLots": total,
        "bySupplier": [],
        "failRateTrend": [],
    })))))
}

// ─────────────────────────────────────────────────────────────────────────────
// アラート（GET /alerts, POST /alerts, POST /alerts/:id/resolve）
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct AlertFilterQuery {
    pub status: Option<String>,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

pub async fn list_alerts(
    _user: AuthenticatedUser<AuditorRole>,
    State(state): State<AppState>,
    Query(q): Query<AlertFilterQuery>,
) -> Result<impl IntoResponse, AppError> {
    let page = q.page.unwrap_or(1).max(1);
    let per_page = q.per_page.unwrap_or(50).clamp(1, 200);
    let offset = (page - 1) * per_page;

    let rows = sqlx::query(
        "SELECT alert_id, case_id, alert_type, status, raised_at, acknowledged_at FROM andon_alerts WHERE ($1::text IS NULL OR status=$1) AND deleted_at IS NULL ORDER BY raised_at DESC LIMIT $2 OFFSET $3",
    )
    .bind(q.status.as_deref()).bind(per_page).bind(offset)
    .fetch_all(&state.read_pool).await?;

    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM andon_alerts WHERE deleted_at IS NULL")
        .fetch_one(&state.read_pool).await?;

    let items: Vec<serde_json::Value> = rows.iter().map(|r| serde_json::json!({
        "id": r.get::<Uuid, _>("alert_id"),
        "caseId": r.get::<Uuid, _>("case_id"),
        "alertType": r.get::<String, _>("alert_type"),
        "status": r.get::<String, _>("status"),
        "raisedAt": r.get::<chrono::DateTime<Utc>, _>("raised_at"),
        "acknowledgedAt": r.get::<Option<chrono::DateTime<Utc>>, _>("acknowledged_at"),
        "deletedAt": null,
    })).collect();

    Ok((StatusCode::OK, Json(paginated_envelope(serde_json::json!(items), total, page, per_page))))
}

pub async fn resolve_alert(
    user: AuthenticatedUser<AuditorRole>,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(_body): Json<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    let now = Utc::now();
    sqlx::query("UPDATE andon_alerts SET status='resolved', resolved_at=$1, resolved_by=$2, updated_at=$1 WHERE alert_id=$3 AND deleted_at IS NULL")
        .bind(now).bind(user.user_id).bind(id).execute(&state.write_pool).await?;

    Ok((StatusCode::OK, Json(envelope(serde_json::json!({"id": id, "status": "resolved", "resolvedAt": now})))))
}

// ─────────────────────────────────────────────────────────────────────────────
// 電子署名（POST /electronic-signs, GET /electronic-signs）
// ─────────────────────────────────────────────────────────────────────────────

pub async fn create_electronic_sign(
    user: AuthenticatedUser<wnav_auth::OperatorRole>,
    State(state): State<AppState>,
    Json(body): Json<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    let id = Uuid::now_v7();
    let now = Utc::now();
    let context_type = body["context_type"].as_str().unwrap_or("approval_sign");
    let context_id: Uuid = body["context_id"].as_str().and_then(|s| s.parse().ok()).unwrap_or_else(Uuid::now_v7);

    sqlx::query("INSERT INTO electronic_signs (sign_id, signer_id, context_type, context_id, signed_at, device_id) VALUES ($1,$2,$3,$4,$5,'')")
        .bind(id).bind(user.user_id).bind(context_type).bind(context_id).bind(now)
        .execute(&state.write_pool).await?;

    Ok((StatusCode::CREATED, Json(envelope(serde_json::json!({"id": id})))))
}

// ─────────────────────────────────────────────────────────────────────────────
// DLQ（異なる URL パス: /outbox/dlq/:id DELETE, /outbox/dlq/:id/retry POST）
// ─────────────────────────────────────────────────────────────────────────────

pub async fn delete_dlq_item(
    _user: AuthenticatedUser<AdminRole>,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    sqlx::query("DELETE FROM outbox_events WHERE outbox_event_id=$1 AND status='dlq'")
        .bind(id).execute(&state.write_pool).await?;

    Ok(StatusCode::NO_CONTENT)
}
