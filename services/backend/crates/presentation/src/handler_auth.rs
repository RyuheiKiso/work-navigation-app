//! 認証 REST ハンドラ
//!
//! 対応 §: ロードマップ §10.5 §11.4.1 §27 F-006

use axum::{extract::{Extension, State}, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use wna_adapter::AuditEntry;
use wna_domain::{PasswordHasher, UserId};
use wna_usecase::{LoginCommand, LoginError};
use chrono::Utc;

use crate::app_state::AppState;
use crate::middleware_auth::AuthContext;

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

pub async fn post_login<H>(
    State(state): State<AppState<H>>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, StatusCode>
where
    H: PasswordHasher + Send + Sync + Clone + 'static,
{
    let user_id = UserId::new(req.user_id.clone()).map_err(|_| StatusCode::BAD_REQUEST)?;
    let cmd = LoginCommand { user_id, plaintext_password: req.password };
    match state.login_uc.execute(cmd).await {
        Ok(session) => {
            // 監査ログ
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
            // 失敗も監査
            let _ = state.audit_repo.append(&AuditEntry {
                actor_id: req.user_id.clone(),
                action: "login_failed".to_string(),
                target_id: Some(req.user_id),
                terminal_time: Some(Utc::now()),
                payload: Some("{\"result\":\"deny\"}".to_string()),
            }).await;
            match e {
                LoginError::UserNotFound | LoginError::CredentialNotFound => Err(StatusCode::NOT_FOUND),
                LoginError::AccountDisabled | LoginError::PasswordMismatch => Err(StatusCode::UNAUTHORIZED),
                _ => Err(StatusCode::INTERNAL_SERVER_ERROR),
            }
        }
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
