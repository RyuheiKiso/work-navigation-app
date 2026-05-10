//! フロー REST ハンドラ
//!
//! 対応 §: ロードマップ §10.2.1 §11.4.1 §20.1

use axum::{extract::{Extension, Path, State}, Json};
use serde::{Deserialize, Serialize};
use wna_adapter::AuditEntry;
use wna_domain::PasswordHasher;
use chrono::Utc;

use crate::api_error::ApiError;
use crate::app_state::AppState;
use crate::middleware_auth::AuthContext;
use crate::middleware_request_id::RequestId;

fn enrich(err: ApiError, rid: Option<&RequestId>) -> ApiError {
    if let Some(r) = rid { err.with_request_id(r.0.clone()) } else { err }
}

#[derive(Serialize)]
pub struct FlowDto {
    pub id: String,
    pub version: i32,
    pub name: String,
    pub status: String,
    pub industry: Option<String>,
}

pub async fn list_flows<H>(
    State(s): State<AppState<H>>, rid: Option<Extension<RequestId>>,
) -> Result<Json<Vec<FlowDto>>, ApiError>
where H: PasswordHasher + Send + Sync + Clone + 'static {
    let rid_ref = rid.as_ref().map(|e| &e.0);
    let rows = s.master_repo.list_flows().await
        .map_err(|e| enrich(ApiError::server("flow_list_failed").with_detail(e.to_string()), rid_ref))?;
    Ok(Json(rows.into_iter().map(|r| FlowDto {
        id: r.id, version: r.version, name: r.name, status: r.status, industry: r.industry,
    }).collect()))
}

#[derive(Deserialize)]
pub struct PublishTrialRequest {
    pub version: Option<i32>,
    pub name: String,
    pub industry: Option<String>,
    pub body: serde_json::Value,
    pub pilot_device_ids: Vec<String>,
}

#[derive(Serialize)]
pub struct PublishTrialResponse {
    pub flow_id: String,
    pub version: i32,
    pub status: String,
    pub pilot_device_ids: Vec<String>,
}

pub async fn publish_trial<H>(
    State(s): State<AppState<H>>,
    Extension(ctx): Extension<AuthContext>,
    rid: Option<Extension<RequestId>>,
    Path(id): Path<String>,
    Json(req): Json<PublishTrialRequest>,
) -> Result<Json<PublishTrialResponse>, ApiError>
where H: PasswordHasher + Send + Sync + Clone + 'static {
    let rid_ref = rid.as_ref().map(|e| &e.0);
    if req.pilot_device_ids.is_empty() {
        return Err(enrich(ApiError::bad_request("pilot_device_ids_required"), rid_ref));
    }
    let version = req.version.unwrap_or(1);
    let body_json = req.body.to_string();
    s.master_repo.upsert_flow(&id, version, &req.name, req.industry.as_deref(), "trial", &body_json)
        .await
        .map_err(|e| enrich(ApiError::server("flow_publish_failed").with_detail(e.to_string()), rid_ref))?;
    let _ = s.audit_repo.append(&AuditEntry {
        actor_id: ctx.user_id, action: "publish_trial".to_string(),
        target_id: Some(format!("{id}@v{version}")),
        terminal_time: Some(Utc::now()),
        payload: Some(serde_json::json!({ "pilot": req.pilot_device_ids }).to_string()),
    }).await;
    Ok(Json(PublishTrialResponse {
        flow_id: id, version, status: "trial".to_string(), pilot_device_ids: req.pilot_device_ids,
    }))
}
