// JWT 検証 AuthMiddleware（master-api 専用）
//
// aud = "master-api" のトークンのみを受け付ける。
// 検証成功時は CurrentUser を Request Extension に注入する。
// /healthz・/api/v1/auth/login・/api/v1/public/config は認証スキップパス。

use axum::{
    body::Body,
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use std::sync::Arc;
use wnav_auth::{CurrentUser, JwtKeyStore};

use crate::error::AppError;

/// 認証スキップパスの判定（公開エンドポイント一覧）。
///
/// - `/healthz`: ヘルスチェック（監視システムが認証なしで叩く）
/// - `/api/v1/auth/login`: ログイン（認証前なのでスキップ必須）
/// - `/api/v1/public/config`: SPA 起動時設定取得（非機密・認証不要）
fn is_public_path(path: &str) -> bool {
    matches!(
        path,
        "/healthz" | "/api/v1/auth/login" | "/api/v1/public/config"
    )
}

/// Authorization: Bearer ヘッダからトークン文字列を抽出するヘルパー。
fn extract_bearer_token(req: &Request<Body>) -> Option<&str> {
    req.headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
}

/// JWT 検証 Tower ミドルウェア（master-api 専用）。
///
/// JwtKeyStore の expected_audience = "master-api" のため、
/// terminal-api 宛てトークンはここで自動的に拒否される。
pub async fn auth_middleware(
    State(key_store): State<Arc<JwtKeyStore>>,
    mut req: Request<Body>,
    next: Next,
) -> Result<Response, AppError> {
    let path = req.uri().path().to_string();

    // 公開エンドポイントは認証をスキップする
    if is_public_path(&path) {
        return Ok(next.run(req).await);
    }

    // Bearer トークンを取得する（存在しない場合は 401）
    let token = extract_bearer_token(&req).ok_or(AppError::Unauthorized)?;
    let token = token.to_string();

    // JWT を検証して Claims を取得する（aud = "master-api" の検証を含む）
    let claims = key_store
        .verify(&token)
        .await
        .map_err(AppError::from)?;

    // Claims から CurrentUser を生成して Request Extension に注入する
    let current_user = CurrentUser {
        user_id: claims.sub,
        roles: claims.roles,
        factory_id: claims.factory_id,
        device_id: claims.device_id,
        jti: claims.jti,
    };

    tracing::debug!(
        event = "auth.success",
        user_id = %current_user.user_id,
        "JWT 認証に成功しました",
    );

    req.extensions_mut().insert(current_user);
    Ok(next.run(req).await)
}
