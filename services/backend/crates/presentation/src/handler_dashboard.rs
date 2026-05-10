//! 班長監視ダッシュボード REST ハンドラ
//!
//! 対応 §: ロードマップ §3.2.1.2 §10.1 §16 §20.1

use axum::{extract::{Extension, State}, Json};
use serde::Serialize;
use wna_domain::PasswordHasher;

use crate::api_error::ApiError;
use crate::app_state::AppState;
use crate::middleware_request_id::RequestId;

#[derive(Serialize)]
pub struct DashboardTaskDto {
    pub id: String,
    pub title: Option<String>,
    pub state: String,
    pub device_id: String,
    pub responsible_user: Option<String>,
    pub current_step_id: Option<String>,
    pub updated_at: String,
}

pub async fn list_dashboard_tasks<H>(
    State(s): State<AppState<H>>,
    rid: Option<Extension<RequestId>>,
) -> Result<Json<Vec<DashboardTaskDto>>, ApiError>
where H: PasswordHasher + Send + Sync + Clone + 'static {
    let rid_ref = rid.as_ref().map(|e| &e.0);
    let rows = s.master_repo.list_tasks().await.map_err(|e| {
        let err = ApiError::server("dashboard_list_failed").with_detail(e.to_string());
        if let Some(r) = rid_ref { err.with_request_id(r.0.clone()) } else { err }
    })?;
    Ok(Json(rows.into_iter().map(|r| DashboardTaskDto {
        id: r.id, title: r.title, state: r.state, device_id: r.device_id,
        responsible_user: r.responsible_user, current_step_id: r.current_step_id,
        updated_at: r.updated_at.to_rfc3339(),
    }).collect()))
}
