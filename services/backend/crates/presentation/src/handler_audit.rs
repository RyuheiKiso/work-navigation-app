//! 監査ログ REST ハンドラ
//!
//! 対応 §: ロードマップ §11.4.1 §16 §20.1

use axum::{extract::{Extension, Query, State}, Json};
use serde::{Deserialize, Serialize};
use wna_domain::PasswordHasher;

use crate::api_error::ApiError;
use crate::app_state::AppState;
use crate::middleware_request_id::RequestId;

#[derive(Deserialize)]
pub struct AuditQuery { pub limit: Option<i64> }

#[derive(Serialize)]
pub struct AuditDto {
    pub id: String,
    pub actor_id: String,
    pub action: String,
    pub target_id: Option<String>,
    pub terminal_time: Option<String>,
    pub server_time: String,
    pub payload: Option<String>,
}

pub async fn list_audit<H>(
    State(state): State<AppState<H>>,
    rid: Option<Extension<RequestId>>,
    Query(q): Query<AuditQuery>,
) -> Result<Json<Vec<AuditDto>>, ApiError>
where H: PasswordHasher + Send + Sync + Clone + 'static {
    let rid_ref = rid.as_ref().map(|e| &e.0);
    let limit = q.limit.unwrap_or(100).min(500);
    let rows = state.audit_repo.list_recent(limit).await.map_err(|e| {
        let err = ApiError::server("audit_list_failed").with_detail(e.to_string());
        if let Some(r) = rid_ref { err.with_request_id(r.0.clone()) } else { err }
    })?;
    Ok(Json(rows.into_iter().map(|r| AuditDto {
        id: r.id, actor_id: r.actor_id, action: r.action,
        target_id: r.target_id,
        terminal_time: r.terminal_time.map(|t| t.to_rfc3339()),
        server_time: r.server_time.to_rfc3339(),
        payload: r.payload,
    }).collect()))
}
