//! 認証 REST ハンドラ
//!
//! 対応 §: ロードマップ §10.5 §11.4.1 §27 F-006 §20.1
//!
//! 失敗は ApiError へ正規化し、UI 側でローカライズ可能にする。
//! 認証系は user enumeration を避けるため、UserNotFound と PasswordMismatch を
//! 同じ kind=auth (401 invalid_credentials) に丸める。

use axum::{extract::{Extension, State}, Json};
use serde::{Deserialize, Serialize};
use wna_adapter::AuditEntry;
use wna_domain::{PasswordHasher, UserId};
use wna_usecase::{LoginCommand, LoginError};
use chrono::Utc;

use crate::api_error::ApiError;
use crate::app_state::AppState;
use crate::middleware_auth::AuthContext;
use crate::middleware_request_id::RequestId;

#[derive(Debug, Clone, Deserialize)]
pub struct LoginRequest {
    pub user_id: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct LoginResponse {
    pub user_id: String,
    pub display_name: String,
    pub session_token: String,
}

fn enrich(err: ApiError, rid: Option<&RequestId>) -> ApiError {
    if let Some(r) = rid { err.with_request_id(r.0.clone()) } else { err }
}

pub async fn post_login<H>(
    State(state): State<AppState<H>>,
    rid: Option<Extension<RequestId>>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, ApiError>
where
    H: PasswordHasher + Send + Sync + Clone + 'static,
{
    let rid_ref = rid.as_ref().map(|e| &e.0);
    let user_id = UserId::new(req.user_id.clone())
        .map_err(|_| enrich(ApiError::bad_request("invalid_user_id"), rid_ref))?;
    let cmd = LoginCommand { user_id, plaintext_password: req.password };
    match state.login_uc.execute(cmd).await {
        Ok(session) => {
            let _ = state.audit_repo.append(&AuditEntry {
                actor_id: session.user().id().as_str().to_string(),
                action: "login".to_string(),
                target_id: Some(session.user().id().as_str().to_string()),
                terminal_time: Some(Utc::now()),
                payload: Some("{\"result\":\"ok\"}".to_string()),
            }).await;
            Ok(Json(LoginResponse {
                user_id: session.user().id().as_str().to_string(),
                display_name: session.user().display_name().to_string(),
                session_token: session.token().as_str().to_string(),
            }))
        }
        Err(e) => {
            let _ = state.audit_repo.append(&AuditEntry {
                actor_id: req.user_id.clone(),
                action: "login_failed".to_string(),
                target_id: Some(req.user_id),
                terminal_time: Some(Utc::now()),
                payload: Some("{\"result\":\"deny\"}".to_string()),
            }).await;
            Err(enrich(login_error_to_api(e), rid_ref))
        }
    }
}

fn login_error_to_api<C, S>(e: LoginError<C, S>) -> ApiError
where
    C: std::error::Error + Send + Sync + 'static,
    S: std::error::Error + Send + Sync + 'static,
{
    // user enumeration を避けるため、UserNotFound と PasswordMismatch は
    // 同一の invalid_credentials へ丸める（タイミング差は usecase 側で吸収）。
    match e {
        LoginError::UserNotFound
        | LoginError::CredentialNotFound
        | LoginError::PasswordMismatch => ApiError::auth("invalid_credentials"),
        LoginError::AccountDisabled => ApiError::forbidden("account_disabled"),
        other => ApiError::server("login_failed").with_detail(format!("{:?}", other)),
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct MeResponse {
    pub user_id: String,
    pub session_age_seconds: i64,
}

pub async fn me<H>(
    Extension(ctx): Extension<AuthContext>,
) -> Json<MeResponse>
where
    H: PasswordHasher + Send + Sync + Clone + 'static,
{
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs() as i64).unwrap_or(0);
    Json(MeResponse {
        user_id: ctx.user_id.clone(),
        session_age_seconds: (now - ctx.issued_at).max(0),
    })
}
