//! マスタ CRUD REST ハンドラ
//!
//! 対応 §: ロードマップ §10.2.1 §10.3.6

use axum::{extract::{Extension, Path, State}, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use wna_adapter::AuditEntry;
use wna_domain::PasswordHasher;
use chrono::Utc;

use crate::app_state::AppState;
use crate::middleware_auth::AuthContext;

#[derive(Serialize)]
pub struct MasterDto { pub code: String, pub name: String, pub extra: Option<String> }

#[derive(Deserialize)]
pub struct UpsertMaster { pub code: String, pub name: String, pub extra: Option<String> }

// ===== 製品 =====
pub async fn list_products<H>(State(s): State<AppState<H>>) -> Result<Json<Vec<MasterDto>>, StatusCode>
where H: PasswordHasher + Send + Sync + Clone + 'static {
    let rows = s.master_repo.list_products().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(rows.into_iter().map(|r| MasterDto { code: r.code, name: r.name, extra: r.extra }).collect()))
}

pub async fn upsert_product<H>(
    State(s): State<AppState<H>>,
    Extension(ctx): Extension<AuthContext>,
    Json(req): Json<UpsertMaster>,
) -> Result<StatusCode, StatusCode>
where H: PasswordHasher + Send + Sync + Clone + 'static {
    s.master_repo.upsert_product(&req.code, &req.name, req.extra.as_deref()).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let _ = s.audit_repo.append(&AuditEntry {
        actor_id: ctx.user_id, action: "upsert_product".to_string(),
        target_id: Some(req.code), terminal_time: Some(Utc::now()), payload: None,
    }).await;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn delete_product<H>(
    State(s): State<AppState<H>>,
    Extension(ctx): Extension<AuthContext>,
    Path(code): Path<String>,
) -> Result<StatusCode, StatusCode>
where H: PasswordHasher + Send + Sync + Clone + 'static {
    s.master_repo.delete_product(&code).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let _ = s.audit_repo.append(&AuditEntry {
        actor_id: ctx.user_id, action: "delete_product".to_string(),
        target_id: Some(code), terminal_time: Some(Utc::now()), payload: None,
    }).await;
    Ok(StatusCode::NO_CONTENT)
}

// ===== 設備 =====
pub async fn list_equipments<H>(State(s): State<AppState<H>>) -> Result<Json<Vec<MasterDto>>, StatusCode>
where H: PasswordHasher + Send + Sync + Clone + 'static {
    let rows = s.master_repo.list_equipments().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(rows.into_iter().map(|r| MasterDto { code: r.code, name: r.name, extra: r.extra }).collect()))
}

pub async fn upsert_equipment<H>(
    State(s): State<AppState<H>>, Extension(ctx): Extension<AuthContext>,
    Json(req): Json<UpsertMaster>,
) -> Result<StatusCode, StatusCode>
where H: PasswordHasher + Send + Sync + Clone + 'static {
    s.master_repo.upsert_equipment(&req.code, &req.name, req.extra.as_deref()).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let _ = s.audit_repo.append(&AuditEntry {
        actor_id: ctx.user_id, action: "upsert_equipment".to_string(),
        target_id: Some(req.code), terminal_time: Some(Utc::now()), payload: None,
    }).await;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn delete_equipment<H>(
    State(s): State<AppState<H>>, Extension(ctx): Extension<AuthContext>,
    Path(code): Path<String>,
) -> Result<StatusCode, StatusCode>
where H: PasswordHasher + Send + Sync + Clone + 'static {
    s.master_repo.delete_equipment(&code).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let _ = s.audit_repo.append(&AuditEntry {
        actor_id: ctx.user_id, action: "delete_equipment".to_string(),
        target_id: Some(code), terminal_time: Some(Utc::now()), payload: None,
    }).await;
    Ok(StatusCode::NO_CONTENT)
}

// ===== 部材 =====
pub async fn list_parts<H>(State(s): State<AppState<H>>) -> Result<Json<Vec<MasterDto>>, StatusCode>
where H: PasswordHasher + Send + Sync + Clone + 'static {
    let rows = s.master_repo.list_parts().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(rows.into_iter().map(|r| MasterDto { code: r.code, name: r.name, extra: r.extra }).collect()))
}

pub async fn upsert_part<H>(
    State(s): State<AppState<H>>, Extension(ctx): Extension<AuthContext>,
    Json(req): Json<UpsertMaster>,
) -> Result<StatusCode, StatusCode>
where H: PasswordHasher + Send + Sync + Clone + 'static {
    s.master_repo.upsert_part(&req.code, &req.name, req.extra.as_deref()).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let _ = s.audit_repo.append(&AuditEntry {
        actor_id: ctx.user_id, action: "upsert_part".to_string(),
        target_id: Some(req.code), terminal_time: Some(Utc::now()), payload: None,
    }).await;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn delete_part<H>(
    State(s): State<AppState<H>>, Extension(ctx): Extension<AuthContext>,
    Path(code): Path<String>,
) -> Result<StatusCode, StatusCode>
where H: PasswordHasher + Send + Sync + Clone + 'static {
    s.master_repo.delete_part(&code).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let _ = s.audit_repo.append(&AuditEntry {
        actor_id: ctx.user_id, action: "delete_part".to_string(),
        target_id: Some(code), terminal_time: Some(Utc::now()), payload: None,
    }).await;
    Ok(StatusCode::NO_CONTENT)
}
