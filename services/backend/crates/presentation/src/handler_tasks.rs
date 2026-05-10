//! タスク REST ハンドラ
//!
//! 対応 §: ロードマップ §10.1 §10.6 §3.6 §3.6.2

use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use wna_adapter::{AuditEntry, TaskDto};
use wna_domain::{Evidence, PasswordHasher, TaskId, TaskRepository};
use wna_usecase::{StartTaskCommand, StartTaskError};
use chrono::Utc;

use crate::app_state::AppState;
use crate::middleware_auth::AuthContext;

pub async fn healthz() -> &'static str { "ok" }

pub async fn get_task<H>(
    State(state): State<AppState<H>>,
    Path(id): Path<String>,
) -> Result<Json<TaskDto>, StatusCode>
where H: PasswordHasher + Send + Sync + Clone + 'static {
    let task_id = TaskId::new(id).map_err(|_| StatusCode::BAD_REQUEST)?;
    match state.task_repo.find_by_id(&task_id).await {
        Ok(Some(task)) => Ok(Json(TaskDto::from_domain(&task))),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

#[derive(Debug, Serialize)]
pub struct TaskListItemDto {
    pub id: String,
    pub title: Option<String>,
    pub state: String,
    pub device_id: String,
    pub responsible_user: Option<String>,
    pub current_step_id: Option<String>,
    pub updated_at: String,
}

pub async fn list_tasks<H>(
    State(state): State<AppState<H>>,
) -> Result<Json<Vec<TaskListItemDto>>, StatusCode>
where H: PasswordHasher + Send + Sync + Clone + 'static {
    let rows = state.master_repo.list_tasks().await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(rows.into_iter().map(|r| TaskListItemDto {
        id: r.id, title: r.title, state: r.state, device_id: r.device_id,
        responsible_user: r.responsible_user, current_step_id: r.current_step_id,
        updated_at: r.updated_at.to_rfc3339(),
    }).collect()))
}

pub async fn start_task<H>(
    State(state): State<AppState<H>>,
    Extension(ctx): Extension<AuthContext>,
    Path(id): Path<String>,
) -> Result<Json<TaskDto>, StatusCode>
where H: PasswordHasher + Send + Sync + Clone + 'static {
    let task_id = TaskId::new(id.clone()).map_err(|_| StatusCode::BAD_REQUEST)?;
    let cmd = StartTaskCommand { task_id };
    match state.start_task_uc.execute(cmd).await {
        Ok(task) => {
            let _ = state.audit_repo.append(&AuditEntry {
                actor_id: ctx.user_id, action: "start_task".to_string(),
                target_id: Some(id), terminal_time: Some(Utc::now()),
                payload: None,
            }).await;
            Ok(Json(TaskDto::from_domain(&task)))
        }
        Err(StartTaskError::NotFound) => Err(StatusCode::NOT_FOUND),
        Err(StartTaskError::Domain(_)) => Err(StatusCode::CONFLICT),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

#[derive(Deserialize)]
pub struct CompleteRequest {
    pub manually_marked: Option<bool>,
    pub photo_attached: Option<bool>,
}

async fn transition<H>(
    state: &AppState<H>,
    ctx: &AuthContext,
    id: &str,
    action: &'static str,
    transition: impl FnOnce(&mut wna_domain::Task) -> Result<(), wna_domain::DomainError>,
) -> Result<Json<TaskDto>, StatusCode>
where H: PasswordHasher + Send + Sync + Clone + 'static {
    let task_id = TaskId::new(id.to_string()).map_err(|_| StatusCode::BAD_REQUEST)?;
    let mut task = state.task_repo.find_by_id(&task_id).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    transition(&mut task).map_err(|_| StatusCode::CONFLICT)?;
    state.task_repo.save(&task).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let _ = state.audit_repo.append(&AuditEntry {
        actor_id: ctx.user_id.clone(), action: action.to_string(),
        target_id: Some(id.to_string()), terminal_time: Some(Utc::now()), payload: None,
    }).await;
    Ok(Json(TaskDto::from_domain(&task)))
}

pub async fn complete_task<H>(
    State(state): State<AppState<H>>,
    Extension(ctx): Extension<AuthContext>,
    Path(id): Path<String>,
    Json(req): Json<CompleteRequest>,
) -> Result<Json<TaskDto>, StatusCode>
where H: PasswordHasher + Send + Sync + Clone + 'static {
    let ev = Evidence {
        manually_marked: req.manually_marked.unwrap_or(false),
        photo_attached: req.photo_attached.unwrap_or(false),
    };
    transition(&state, &ctx, &id, "complete_task", |t| t.complete(&ev)).await
}

pub async fn suspend_task<H>(
    State(state): State<AppState<H>>,
    Extension(ctx): Extension<AuthContext>,
    Path(id): Path<String>,
) -> Result<Json<TaskDto>, StatusCode>
where H: PasswordHasher + Send + Sync + Clone + 'static {
    transition(&state, &ctx, &id, "suspend_task", |t| t.suspend()).await
}

pub async fn resume_task<H>(
    State(state): State<AppState<H>>,
    Extension(ctx): Extension<AuthContext>,
    Path(id): Path<String>,
) -> Result<Json<TaskDto>, StatusCode>
where H: PasswordHasher + Send + Sync + Clone + 'static {
    transition(&state, &ctx, &id, "resume_task", |t| t.resume()).await
}

pub async fn abort_task<H>(
    State(state): State<AppState<H>>,
    Extension(ctx): Extension<AuthContext>,
    Path(id): Path<String>,
) -> Result<Json<TaskDto>, StatusCode>
where H: PasswordHasher + Send + Sync + Clone + 'static {
    transition(&state, &ctx, &id, "abort_task", |t| t.abort()).await
}

#[derive(Debug, Serialize)]
pub struct StepDto {
    pub id: String,
    pub sequence: i32,
    pub label: String,
    pub completion_criteria: String,
    pub standard_time_seconds: i32,
    pub done: bool,
}

pub async fn list_steps<H>(
    State(state): State<AppState<H>>,
    Path(id): Path<String>,
) -> Result<Json<Vec<StepDto>>, StatusCode>
where H: PasswordHasher + Send + Sync + Clone + 'static {
    let rows = state.master_repo.list_steps(&id).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(rows.into_iter().map(|r| StepDto {
        id: r.id, sequence: r.sequence, label: r.label,
        completion_criteria: r.completion_criteria, standard_time_seconds: r.standard_time_seconds,
        done: r.done,
    }).collect()))
}

pub async fn mark_step_done<H>(
    State(state): State<AppState<H>>,
    Extension(ctx): Extension<AuthContext>,
    Path((task_id, step_id)): Path<(String, String)>,
) -> Result<StatusCode, StatusCode>
where H: PasswordHasher + Send + Sync + Clone + 'static {
    state.master_repo.mark_step_done(&task_id, &step_id).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let _ = state.audit_repo.append(&AuditEntry {
        actor_id: ctx.user_id, action: "mark_step_done".to_string(),
        target_id: Some(format!("{task_id}:{step_id}")),
        terminal_time: Some(Utc::now()),
        payload: None,
    }).await;
    Ok(StatusCode::NO_CONTENT)
}
