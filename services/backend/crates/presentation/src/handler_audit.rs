//! 監査ログ REST ハンドラ
//!
//! 対応 §: ロードマップ §11.4.1 §16

use axum::{extract::{Query, State}, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use wna_domain::PasswordHasher;

use crate::app_state::AppState;

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
    Query(q): Query<AuditQuery>,
) -> Result<Json<Vec<AuditDto>>, StatusCode>
where H: PasswordHasher + Send + Sync + Clone + 'static {
    let limit = q.limit.unwrap_or(100).min(500);
    let rows = state.audit_repo.list_recent(limit).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(rows.into_iter().map(|r| AuditDto {
        id: r.id, actor_id: r.actor_id, action: r.action,
        target_id: r.target_id,
        terminal_time: r.terminal_time.map(|t| t.to_rfc3339()),
        server_time: r.server_time.to_rfc3339(),
        payload: r.payload,
    }).collect()))
}
