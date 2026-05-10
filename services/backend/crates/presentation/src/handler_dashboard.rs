//! 班長監視ダッシュボード REST ハンドラ
//!
//! 対応 §: ロードマップ §3.2.1.2 §10.1 §16

use axum::{extract::State, http::StatusCode, Json};
use serde::Serialize;
use wna_domain::PasswordHasher;

use crate::app_state::AppState;

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
) -> Result<Json<Vec<DashboardTaskDto>>, StatusCode>
where H: PasswordHasher + Send + Sync + Clone + 'static {
    let rows = s.master_repo.list_tasks().await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(rows.into_iter().map(|r| DashboardTaskDto {
        id: r.id, title: r.title, state: r.state, device_id: r.device_id,
        responsible_user: r.responsible_user, current_step_id: r.current_step_id,
        updated_at: r.updated_at.to_rfc3339(),
    }).collect()))
}
