//! マスタ CRUD REST ハンドラ
//!
//! 対応 §: ロードマップ §10.2.1 §10.3.6 §20.1
//!
//! 失敗は ApiError へ正規化し、UI 側でローカライズ可能にする。
//! 製品／設備／部材で構造が同形なので、内部ヘルパで共有する。

use axum::{extract::{Extension, Path, State}, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use wna_adapter::AuditEntry;
use wna_domain::PasswordHasher;
use chrono::Utc;

use crate::api_error::ApiError;
use crate::app_state::AppState;
use crate::middleware_auth::AuthContext;
use crate::middleware_request_id::RequestId;

#[derive(Serialize)]
pub struct MasterDto { pub code: String, pub name: String, pub extra: Option<String> }

#[derive(Deserialize)]
pub struct UpsertMaster { pub code: String, pub name: String, pub extra: Option<String> }

fn server(code: &'static str, e: impl std::fmt::Display, rid: Option<&RequestId>) -> ApiError {
    let err = ApiError::server(code).with_detail(e.to_string());
    if let Some(r) = rid { err.with_request_id(r.0.clone()) } else { err }
}

// ===== 製品 =====
pub async fn list_products<H>(
    State(s): State<AppState<H>>, rid: Option<Extension<RequestId>>,
) -> Result<Json<Vec<MasterDto>>, ApiError>
where H: PasswordHasher + Send + Sync + Clone + 'static {
    let rid_ref = rid.as_ref().map(|e| &e.0);
    let rows = s.master_repo.list_products().await
        .map_err(|e| server("master_list_failed", e, rid_ref))?;
    Ok(Json(rows.into_iter().map(|r| MasterDto { code: r.code, name: r.name, extra: r.extra }).collect()))
}

pub async fn upsert_product<H>(
    State(s): State<AppState<H>>, Extension(ctx): Extension<AuthContext>,
    rid: Option<Extension<RequestId>>, Json(req): Json<UpsertMaster>,
) -> Result<StatusCode, ApiError>
where H: PasswordHasher + Send + Sync + Clone + 'static {
    let rid_ref = rid.as_ref().map(|e| &e.0);
    s.master_repo.upsert_product(&req.code, &req.name, req.extra.as_deref()).await
        .map_err(|e| server("master_upsert_failed", e, rid_ref))?;
    let _ = s.audit_repo.append(&AuditEntry {
        actor_id: ctx.user_id, action: "upsert_product".to_string(),
        target_id: Some(req.code), terminal_time: Some(Utc::now()), payload: None,
    }).await;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn delete_product<H>(
    State(s): State<AppState<H>>, Extension(ctx): Extension<AuthContext>,
    rid: Option<Extension<RequestId>>, Path(code): Path<String>,
) -> Result<StatusCode, ApiError>
where H: PasswordHasher + Send + Sync + Clone + 'static {
    let rid_ref = rid.as_ref().map(|e| &e.0);
    s.master_repo.delete_product(&code).await
        .map_err(|e| server("master_delete_failed", e, rid_ref))?;
    let _ = s.audit_repo.append(&AuditEntry {
        actor_id: ctx.user_id, action: "delete_product".to_string(),
        target_id: Some(code), terminal_time: Some(Utc::now()), payload: None,
    }).await;
    Ok(StatusCode::NO_CONTENT)
}

// ===== 設備 =====
pub async fn list_equipments<H>(
    State(s): State<AppState<H>>, rid: Option<Extension<RequestId>>,
) -> Result<Json<Vec<MasterDto>>, ApiError>
where H: PasswordHasher + Send + Sync + Clone + 'static {
    let rid_ref = rid.as_ref().map(|e| &e.0);
    let rows = s.master_repo.list_equipments().await
        .map_err(|e| server("master_list_failed", e, rid_ref))?;
    Ok(Json(rows.into_iter().map(|r| MasterDto { code: r.code, name: r.name, extra: r.extra }).collect()))
}

pub async fn upsert_equipment<H>(
    State(s): State<AppState<H>>, Extension(ctx): Extension<AuthContext>,
    rid: Option<Extension<RequestId>>, Json(req): Json<UpsertMaster>,
) -> Result<StatusCode, ApiError>
where H: PasswordHasher + Send + Sync + Clone + 'static {
    let rid_ref = rid.as_ref().map(|e| &e.0);
    s.master_repo.upsert_equipment(&req.code, &req.name, req.extra.as_deref()).await
        .map_err(|e| server("master_upsert_failed", e, rid_ref))?;
    let _ = s.audit_repo.append(&AuditEntry {
        actor_id: ctx.user_id, action: "upsert_equipment".to_string(),
        target_id: Some(req.code), terminal_time: Some(Utc::now()), payload: None,
    }).await;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn delete_equipment<H>(
    State(s): State<AppState<H>>, Extension(ctx): Extension<AuthContext>,
    rid: Option<Extension<RequestId>>, Path(code): Path<String>,
) -> Result<StatusCode, ApiError>
where H: PasswordHasher + Send + Sync + Clone + 'static {
    let rid_ref = rid.as_ref().map(|e| &e.0);
    s.master_repo.delete_equipment(&code).await
        .map_err(|e| server("master_delete_failed", e, rid_ref))?;
    let _ = s.audit_repo.append(&AuditEntry {
        actor_id: ctx.user_id, action: "delete_equipment".to_string(),
        target_id: Some(code), terminal_time: Some(Utc::now()), payload: None,
    }).await;
    Ok(StatusCode::NO_CONTENT)
}

// ===== 部材 =====
pub async fn list_parts<H>(
    State(s): State<AppState<H>>, rid: Option<Extension<RequestId>>,
) -> Result<Json<Vec<MasterDto>>, ApiError>
where H: PasswordHasher + Send + Sync + Clone + 'static {
    let rid_ref = rid.as_ref().map(|e| &e.0);
    let rows = s.master_repo.list_parts().await
        .map_err(|e| server("master_list_failed", e, rid_ref))?;
    Ok(Json(rows.into_iter().map(|r| MasterDto { code: r.code, name: r.name, extra: r.extra }).collect()))
}

pub async fn upsert_part<H>(
    State(s): State<AppState<H>>, Extension(ctx): Extension<AuthContext>,
    rid: Option<Extension<RequestId>>, Json(req): Json<UpsertMaster>,
) -> Result<StatusCode, ApiError>
where H: PasswordHasher + Send + Sync + Clone + 'static {
    let rid_ref = rid.as_ref().map(|e| &e.0);
    s.master_repo.upsert_part(&req.code, &req.name, req.extra.as_deref()).await
        .map_err(|e| server("master_upsert_failed", e, rid_ref))?;
    let _ = s.audit_repo.append(&AuditEntry {
        actor_id: ctx.user_id, action: "upsert_part".to_string(),
        target_id: Some(req.code), terminal_time: Some(Utc::now()), payload: None,
    }).await;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn delete_part<H>(
    State(s): State<AppState<H>>, Extension(ctx): Extension<AuthContext>,
    rid: Option<Extension<RequestId>>, Path(code): Path<String>,
) -> Result<StatusCode, ApiError>
where H: PasswordHasher + Send + Sync + Clone + 'static {
    let rid_ref = rid.as_ref().map(|e| &e.0);
    s.master_repo.delete_part(&code).await
        .map_err(|e| server("master_delete_failed", e, rid_ref))?;
    let _ = s.audit_repo.append(&AuditEntry {
        actor_id: ctx.user_id, action: "delete_part".to_string(),
        target_id: Some(code), terminal_time: Some(Utc::now()), payload: None,
    }).await;
    Ok(StatusCode::NO_CONTENT)
}
