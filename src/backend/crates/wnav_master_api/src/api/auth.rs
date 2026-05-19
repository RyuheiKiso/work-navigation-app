// 認証ハンドラ（API-auth-001〜004）
//
// ログイン・トークン更新・ログアウト・鍵ローテーション。
// aud = "master-api" で JWT を発行する。

use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use uuid::Uuid;

use crate::{
    dto::auth::{
        KeyRotateRequest, KeyRotateResponse, LoginRequest, LoginResponse, LogoutRequest,
        RefreshRequest, RefreshResponse,
    },
    error::AppError,
    state::AppState,
};
use wnav_auth::{AdminRole, AuthenticatedUser, JwtIssueCmd, verify_password};

/// ログイン（POST /api/v1/auth/login）。
///
/// ログイン ID とパスワードを検証して JWT を発行する（aud = "master-api"）。
#[utoipa::path(
    post,
    path = "/api/v1/auth/login",
    tag = "auth",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "ログイン成功", body = LoginResponse),
        (status = 401, description = "認証失敗"),
    )
)]
pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<impl IntoResponse, AppError> {
    // ユーザー情報を read_pool で取得する
    let row = sqlx::query(
        r#"
        SELECT id, password_hash, roles, factory_id, is_active
        FROM users
        WHERE login_id = $1 AND deleted_at IS NULL
        "#,
    )
    .bind(&req.login_id)
    .fetch_optional(&state.read_pool)
    .await?;

    let user = row.ok_or(AppError::Unauthorized)?;

    use sqlx::Row as _;
    let is_active: bool = user.get("is_active");
    if !is_active {
        return Err(AppError::Unauthorized);
    }

    let password_hash: String = user.get("password_hash");
    let is_valid = verify_password(&req.password, &password_hash).map_err(|_| AppError::Unauthorized)?;
    if !is_valid {
        return Err(AppError::Unauthorized);
    }

    let user_id: Uuid = user.get("id");
    let factory_id: Uuid = user.get("factory_id");
    let roles_json: serde_json::Value = user.get("roles");
    let roles: Vec<String> = roles_json
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .filter_map(|v| v.as_str().map(ToString::to_string))
        .collect();

    // kid は config には存在しないため固定値を使用する（本番は設定化する）
    let kid = "2026-Q2".to_string();

    // JWT 発行コマンドを構築する（aud = "master-api" で発行する）
    let cmd = JwtIssueCmd {
        user_id,
        roles,
        factory_id,
        device_id: None, // master-api では端末 ID は不要
        kid,
        audience: "master-api".to_string(),
    };

    // TTL 8 時間（28800 秒）で JWT を発行する
    let access_token = state
        .key_store
        .issue(cmd, 28800)
        .map_err(|_| AppError::Internal("JWT 発行失敗".to_string()))?;

    // リフレッシュトークン（UUID v7 で代替。本番実装では別テーブルで管理する）
    let refresh_token = Uuid::now_v7().to_string();

    tracing::info!(
        event = "auth.login.success",
        user_id = %user_id,
        "ログインに成功しました",
    );

    Ok((
        StatusCode::OK,
        Json(LoginResponse {
            access_token,
            refresh_token,
            expires_in: 28800,
            token_type: "Bearer".to_string(),
        }),
    ))
}

/// トークン更新（POST /api/v1/auth/refresh）。
///
/// リフレッシュトークンを検証して新しいアクセストークンを発行する。
#[utoipa::path(
    post,
    path = "/api/v1/auth/refresh",
    tag = "auth",
    request_body = RefreshRequest,
    responses(
        (status = 200, description = "トークン更新成功", body = RefreshResponse),
        (status = 401, description = "リフレッシュトークン無効"),
    )
)]
pub async fn refresh(
    State(state): State<AppState>,
    Json(req): Json<RefreshRequest>,
) -> Result<impl IntoResponse, AppError> {
    let token_id = Uuid::parse_str(&req.refresh_token).map_err(|_| AppError::Unauthorized)?;

    let row = sqlx::query(
        r#"
        SELECT user_id, expires_at, is_revoked
        FROM refresh_tokens
        WHERE id = $1
        "#,
    )
    .bind(token_id)
    .fetch_optional(&state.read_pool)
    .await?;

    let token_row = row.ok_or(AppError::Unauthorized)?;

    use sqlx::Row as _;
    let is_revoked: bool = token_row.get("is_revoked");
    if is_revoked {
        return Err(AppError::Unauthorized);
    }
    let expires_at: chrono::DateTime<chrono::Utc> = token_row.get("expires_at");
    if expires_at < chrono::Utc::now() {
        return Err(AppError::JwtExpired);
    }
    let user_id: Uuid = token_row.get("user_id");

    let user_row = sqlx::query(
        r#"
        SELECT id, roles, factory_id, is_active
        FROM users
        WHERE id = $1 AND deleted_at IS NULL
        "#,
    )
    .bind(user_id)
    .fetch_optional(&state.read_pool)
    .await?
    .ok_or(AppError::Unauthorized)?;

    let is_active: bool = user_row.get("is_active");
    if !is_active {
        return Err(AppError::Unauthorized);
    }

    let factory_id: Uuid = user_row.get("factory_id");
    let roles_json: serde_json::Value = user_row.get("roles");
    let roles: Vec<String> = roles_json
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .filter_map(|v| v.as_str().map(ToString::to_string))
        .collect();

    let kid = "2026-Q2".to_string();
    let cmd = JwtIssueCmd {
        user_id,
        roles,
        factory_id,
        device_id: None,
        kid,
        audience: "master-api".to_string(),
    };

    let access_token = state
        .key_store
        .issue(cmd, 28800)
        .map_err(|_| AppError::Internal("JWT 発行失敗".to_string()))?;

    Ok((
        StatusCode::OK,
        Json(RefreshResponse {
            access_token,
            expires_in: 28800,
            token_type: "Bearer".to_string(),
        }),
    ))
}

/// ログアウト（POST /api/v1/auth/logout）。
///
/// リフレッシュトークンを失効させる。
#[utoipa::path(
    post,
    path = "/api/v1/auth/logout",
    tag = "auth",
    security(("Bearer" = [])),
    request_body = LogoutRequest,
    responses(
        (status = 204, description = "ログアウト成功"),
        (status = 401, description = "未認証"),
    )
)]
pub async fn logout(
    State(state): State<AppState>,
    Json(req): Json<LogoutRequest>,
) -> Result<impl IntoResponse, AppError> {
    let token_id = Uuid::parse_str(&req.refresh_token)
        .map_err(|_| AppError::Validation("refresh_token の形式が不正です".to_string()))?;

    // リフレッシュトークンを失効させる（write_pool で UPDATE）
    sqlx::query(
        r#"
        UPDATE refresh_tokens
        SET is_revoked = true, revoked_at = NOW()
        WHERE id = $1
        "#,
    )
    .bind(token_id)
    .execute(&state.write_pool)
    .await?;

    tracing::info!(event = "auth.logout", "ログアウトしました");

    Ok(StatusCode::NO_CONTENT)
}

/// JWT 鍵ローテーション（POST /api/v1/auth/keys/rotate）。
///
/// AdminRole 必須。Grace Period 中は旧鍵・新鍵の両方を受け入れる（BAT-010 連携）。
#[utoipa::path(
    post,
    path = "/api/v1/auth/keys/rotate",
    tag = "auth",
    security(("Bearer" = [])),
    request_body = KeyRotateRequest,
    responses(
        (status = 200, description = "鍵ローテーション成功", body = KeyRotateResponse),
        (status = 401, description = "未認証"),
        (status = 403, description = "AdminRole 必須"),
    )
)]
pub async fn rotate_keys(
    _user: AuthenticatedUser<AdminRole>,
    State(state): State<AppState>,
    Json(req): Json<KeyRotateRequest>,
) -> Result<impl IntoResponse, AppError> {
    // 新しい公開鍵を JwtKeyStore に追加する（Grace Period で旧鍵も残す）
    state
        .key_store
        .add_key(req.new_kid.clone(), req.new_public_key_pem.clone())
        .await;

    let old_key_expires_at = chrono::Utc::now().timestamp() + req.grace_period_sec as i64;

    tracing::info!(
        event = "auth.key_rotated",
        new_kid = %req.new_kid,
        grace_period_sec = req.grace_period_sec,
        "JWT 鍵ローテーションを実行しました",
    );

    Ok((
        StatusCode::OK,
        Json(KeyRotateResponse {
            new_kid: req.new_kid,
            message: "JWT key rotation completed. Old key will expire after grace period."
                .to_string(),
            old_key_expires_at,
        }),
    ))
}
