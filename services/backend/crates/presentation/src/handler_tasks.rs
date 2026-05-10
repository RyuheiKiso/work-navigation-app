//! タスク REST ハンドラ
//!
//! 対応 §: ロードマップ §10.1 §10.6 §3.6 §3.6.2 §10.5 §20.1
//!
//! 失敗時は [`ApiError`] を返し、RFC 7807 problem+json として serialise される。

use axum::{
    extract::{Extension, Path, Query, State},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use wna_adapter::{AuditEntry, TaskDto};
use wna_domain::{Evidence, PasswordHasher, TaskId, TaskRepository};
use wna_usecase::{StartTaskCommand, StartTaskError};
use chrono::{DateTime, Utc};

use crate::api_error::ApiError;
use crate::app_state::AppState;
use crate::middleware_auth::AuthContext;
use crate::middleware_request_id::RequestId;

/// 共通: ApiError を返す前にリクエスト ID を埋め込む
fn enrich(err: ApiError, rid: Option<&RequestId>) -> ApiError {
    if let Some(r) = rid {
        err.with_request_id(r.0.clone())
    } else {
        err
    }
}

pub async fn get_task<H>(
    State(state): State<AppState<H>>,
    rid: Option<Extension<RequestId>>,
    Path(id): Path<String>,
) -> Result<Json<TaskDto>, ApiError>
where H: PasswordHasher + Send + Sync + Clone + 'static {
    let rid_ref = rid.as_ref().map(|e| &e.0);
    let task_id = TaskId::new(id).map_err(|_| enrich(ApiError::bad_request("invalid_task_id"), rid_ref))?;
    match state.task_repo.find_by_id(&task_id).await {
        Ok(Some(task)) => Ok(Json(TaskDto::from_domain(&task))),
        Ok(None) => Err(enrich(ApiError::not_found("task_not_found"), rid_ref)),
        Err(e) => Err(enrich(
            ApiError::server("task_repo_error").with_detail(e.to_string()),
            rid_ref,
        )),
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

/// `GET /tasks` の増分同期パラメタ。
///
/// `?since=<RFC3339>` を渡すと、その時刻より新しい行だけが返る。
/// 端末側はレスポンス時のサーバ時刻（または最新 row の updated_at）を
/// 次回 `since` として使う。
#[derive(Debug, Deserialize)]
pub struct ListTasksQuery {
    pub since: Option<String>,
}

pub async fn list_tasks<H>(
    State(state): State<AppState<H>>,
    rid: Option<Extension<RequestId>>,
    headers: HeaderMap,
    Query(query): Query<ListTasksQuery>,
) -> Result<Response, ApiError>
where H: PasswordHasher + Send + Sync + Clone + 'static {
    let rid_ref = rid.as_ref().map(|e| &e.0);

    // since (クエリ) を優先し、無ければ If-Modified-Since (HTTP) を読む
    let cursor = parse_since(query.since.as_deref(), &headers)
        .map_err(|e: &'static str| enrich(ApiError::bad_request("invalid_since").with_detail(e.to_string()), rid_ref))?;

    let rows = state.master_repo.list_tasks_since(cursor).await
        .map_err(|e| enrich(ApiError::server("task_list_failed").with_detail(e.to_string()), rid_ref))?;

    // 全 row が cursor 以下なら 304 Not Modified（ボディなし）で帯域節約
    if cursor.is_some() && rows.is_empty() {
        let mut h = HeaderMap::new();
        if let Some(ref r) = rid_ref {
            if let Ok(v) = r.0.parse() {
                h.insert(crate::api_error::REQUEST_ID_HEADER, v);
            }
        }
        return Ok((StatusCode::NOT_MODIFIED, h).into_response());
    }

    let latest = rows.iter().map(|r| r.updated_at).max();
    let dtos: Vec<TaskListItemDto> = rows.into_iter().map(|r| TaskListItemDto {
        id: r.id, title: r.title, state: r.state, device_id: r.device_id,
        responsible_user: r.responsible_user, current_step_id: r.current_step_id,
        updated_at: r.updated_at.to_rfc3339(),
    }).collect();

    let mut response = Json(dtos).into_response();
    if let Some(ts) = latest {
        // RFC 7232 の Last-Modified は HTTP-date 形式（RFC 7231）が正だが、
        // 端末側の比較は ISO 8601 で行うため互換のため両方を考慮しやすい RFC3339 を採用する。
        if let Ok(v) = HeaderValue::from_str(&ts.to_rfc3339()) {
            response.headers_mut().insert(header::LAST_MODIFIED, v);
        }
    }
    if let Some(r) = rid_ref {
        if let Ok(v) = r.0.parse::<HeaderValue>() {
            response
                .headers_mut()
                .insert(crate::api_error::REQUEST_ID_HEADER, v);
        }
    }
    Ok(response)
}

fn parse_since(
    since_q: Option<&str>,
    headers: &HeaderMap,
) -> Result<Option<DateTime<Utc>>, &'static str> {
    if let Some(s) = since_q {
        if s.is_empty() {
            return Ok(None);
        }
        let dt = DateTime::parse_from_rfc3339(s).map_err(|_| "since must be RFC3339")?;
        return Ok(Some(dt.with_timezone(&Utc)));
    }
    if let Some(v) = headers.get(header::IF_MODIFIED_SINCE) {
        if let Ok(s) = v.to_str() {
            // RFC3339 と HTTP-date の両方を許容する寛容実装
            if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
                return Ok(Some(dt.with_timezone(&Utc)));
            }
            if let Ok(dt) = DateTime::parse_from_rfc2822(s) {
                return Ok(Some(dt.with_timezone(&Utc)));
            }
        }
    }
    Ok(None)
}

pub async fn start_task<H>(
    State(state): State<AppState<H>>,
    Extension(ctx): Extension<AuthContext>,
    rid: Option<Extension<RequestId>>,
    Path(id): Path<String>,
) -> Result<Json<TaskDto>, ApiError>
where H: PasswordHasher + Send + Sync + Clone + 'static {
    let rid_ref = rid.as_ref().map(|e| &e.0);
    let task_id = TaskId::new(id.clone())
        .map_err(|_| enrich(ApiError::bad_request("invalid_task_id"), rid_ref))?;
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
        Err(StartTaskError::NotFound) => Err(enrich(ApiError::not_found("task_not_found"), rid_ref)),
        Err(StartTaskError::Domain(e)) => Err(enrich(
            ApiError::conflict("invalid_state_transition").with_detail(format!("{:?}", e)),
            rid_ref,
        )),
        Err(e) => Err(enrich(
            ApiError::server("start_task_failed").with_detail(format!("{:?}", e)),
            rid_ref,
        )),
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
    rid: Option<&RequestId>,
    id: &str,
    action: &'static str,
    transition: impl FnOnce(&mut wna_domain::Task) -> Result<(), wna_domain::DomainError>,
) -> Result<Json<TaskDto>, ApiError>
where H: PasswordHasher + Send + Sync + Clone + 'static {
    let task_id = TaskId::new(id.to_string())
        .map_err(|_| enrich(ApiError::bad_request("invalid_task_id"), rid))?;
    let mut task = state.task_repo.find_by_id(&task_id).await
        .map_err(|e| enrich(ApiError::server("task_fetch_failed").with_detail(e.to_string()), rid))?
        .ok_or_else(|| enrich(ApiError::not_found("task_not_found"), rid))?;
    transition(&mut task)
        .map_err(|e| enrich(ApiError::conflict("invalid_state_transition").with_detail(format!("{:?}", e)), rid))?;
    state.task_repo.save(&task).await
        .map_err(|e| enrich(ApiError::server("task_save_failed").with_detail(e.to_string()), rid))?;
    let _ = state.audit_repo.append(&AuditEntry {
        actor_id: ctx.user_id.clone(), action: action.to_string(),
        target_id: Some(id.to_string()), terminal_time: Some(Utc::now()), payload: None,
    }).await;
    Ok(Json(TaskDto::from_domain(&task)))
}

pub async fn complete_task<H>(
    State(state): State<AppState<H>>,
    Extension(ctx): Extension<AuthContext>,
    rid: Option<Extension<RequestId>>,
    Path(id): Path<String>,
    Json(req): Json<CompleteRequest>,
) -> Result<Json<TaskDto>, ApiError>
where H: PasswordHasher + Send + Sync + Clone + 'static {
    let ev = Evidence {
        manually_marked: req.manually_marked.unwrap_or(false),
        photo_attached: req.photo_attached.unwrap_or(false),
    };
    transition(&state, &ctx, rid.as_ref().map(|e| &e.0), &id, "complete_task", |t| t.complete(&ev)).await
}

pub async fn suspend_task<H>(
    State(state): State<AppState<H>>,
    Extension(ctx): Extension<AuthContext>,
    rid: Option<Extension<RequestId>>,
    Path(id): Path<String>,
) -> Result<Json<TaskDto>, ApiError>
where H: PasswordHasher + Send + Sync + Clone + 'static {
    transition(&state, &ctx, rid.as_ref().map(|e| &e.0), &id, "suspend_task", |t| t.suspend()).await
}

pub async fn resume_task<H>(
    State(state): State<AppState<H>>,
    Extension(ctx): Extension<AuthContext>,
    rid: Option<Extension<RequestId>>,
    Path(id): Path<String>,
) -> Result<Json<TaskDto>, ApiError>
where H: PasswordHasher + Send + Sync + Clone + 'static {
    transition(&state, &ctx, rid.as_ref().map(|e| &e.0), &id, "resume_task", |t| t.resume()).await
}

pub async fn abort_task<H>(
    State(state): State<AppState<H>>,
    Extension(ctx): Extension<AuthContext>,
    rid: Option<Extension<RequestId>>,
    Path(id): Path<String>,
) -> Result<Json<TaskDto>, ApiError>
where H: PasswordHasher + Send + Sync + Clone + 'static {
    transition(&state, &ctx, rid.as_ref().map(|e| &e.0), &id, "abort_task", |t| t.abort()).await
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
    rid: Option<Extension<RequestId>>,
    Path(id): Path<String>,
) -> Result<Json<Vec<StepDto>>, ApiError>
where H: PasswordHasher + Send + Sync + Clone + 'static {
    let rid_ref = rid.as_ref().map(|e| &e.0);
    let rows = state.master_repo.list_steps(&id).await
        .map_err(|e| enrich(ApiError::server("step_list_failed").with_detail(e.to_string()), rid_ref))?;
    Ok(Json(rows.into_iter().map(|r| StepDto {
        id: r.id, sequence: r.sequence, label: r.label,
        completion_criteria: r.completion_criteria, standard_time_seconds: r.standard_time_seconds,
        done: r.done,
    }).collect()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_since_accepts_query_rfc3339() {
        let h = HeaderMap::new();
        let r = parse_since(Some("2026-05-10T10:30:00Z"), &h).unwrap();
        assert!(r.is_some());
    }

    #[test]
    fn parse_since_returns_none_for_empty_query() {
        let h = HeaderMap::new();
        assert!(parse_since(Some(""), &h).unwrap().is_none());
        assert!(parse_since(None, &h).unwrap().is_none());
    }

    #[test]
    fn parse_since_falls_back_to_if_modified_since() {
        let mut h = HeaderMap::new();
        h.insert(header::IF_MODIFIED_SINCE, "2026-05-10T10:30:00Z".parse().unwrap());
        assert!(parse_since(None, &h).unwrap().is_some());
    }

    #[test]
    fn parse_since_rejects_invalid_query() {
        let h = HeaderMap::new();
        assert!(parse_since(Some("not-a-date"), &h).is_err());
    }
}

pub async fn mark_step_done<H>(
    State(state): State<AppState<H>>,
    Extension(ctx): Extension<AuthContext>,
    rid: Option<Extension<RequestId>>,
    Path((task_id, step_id)): Path<(String, String)>,
) -> Result<axum::http::StatusCode, ApiError>
where H: PasswordHasher + Send + Sync + Clone + 'static {
    let rid_ref = rid.as_ref().map(|e| &e.0);
    state.master_repo.mark_step_done(&task_id, &step_id).await
        .map_err(|e| enrich(ApiError::server("step_done_failed").with_detail(e.to_string()), rid_ref))?;
    let _ = state.audit_repo.append(&AuditEntry {
        actor_id: ctx.user_id, action: "mark_step_done".to_string(),
        target_id: Some(format!("{task_id}:{step_id}")),
        terminal_time: Some(Utc::now()),
        payload: None,
    }).await;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
