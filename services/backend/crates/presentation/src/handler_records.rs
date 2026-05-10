//! 実績追記 REST ハンドラ
//!
//! 対応 §: ロードマップ §10.6.1 §11.4.1 §20.1

use axum::{extract::{Extension, Path, State}, http::StatusCode, Json};
use serde::Deserialize;
use wna_adapter::AuditEntry;
use wna_domain::{DeviceId, LamportTimestamp, PasswordHasher, TaskId};
use wna_usecase::{AppendRecordCommand, RecordRepository};
use chrono::Utc;

use crate::api_error::ApiError;
use crate::app_state::AppState;
use crate::middleware_auth::AuthContext;
use crate::middleware_request_id::RequestId;

#[derive(Deserialize)]
pub struct AppendRecordRequest {
    pub device_id: String,
    pub lamport: u64,
    pub payload: serde_json::Value,
}

fn enrich(err: ApiError, rid: Option<&RequestId>) -> ApiError {
    if let Some(r) = rid { err.with_request_id(r.0.clone()) } else { err }
}

pub async fn append_record<H>(
    State(state): State<AppState<H>>,
    Extension(ctx): Extension<AuthContext>,
    rid: Option<Extension<RequestId>>,
    Path(task_id): Path<String>,
    Json(req): Json<AppendRecordRequest>,
) -> Result<StatusCode, ApiError>
where H: PasswordHasher + Send + Sync + Clone + 'static {
    let rid_ref = rid.as_ref().map(|e| &e.0);
    let task = TaskId::new(task_id.clone())
        .map_err(|_| enrich(ApiError::bad_request("invalid_task_id"), rid_ref))?;
    let device = DeviceId::new(req.device_id)
        .map_err(|_| enrich(ApiError::bad_request("invalid_device_id"), rid_ref))?;
    let lamport = LamportTimestamp::from_u64(req.lamport);
    let payload = req.payload.to_string();
    let cmd = AppendRecordCommand { task_id: task, device_id: device, lamport, payload: payload.clone() };
    state.task_repo.append(&cmd).await
        .map_err(|e| enrich(ApiError::server("record_append_failed").with_detail(e.to_string()), rid_ref))?;
    let _ = state.audit_repo.append(&AuditEntry {
        actor_id: ctx.user_id, action: "append_record".to_string(),
        target_id: Some(task_id), terminal_time: Some(Utc::now()),
        payload: Some(payload),
    }).await;
    Ok(StatusCode::CREATED)
}
