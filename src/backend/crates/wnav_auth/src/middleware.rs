// JWT 検証 Tower ミドルウェアと認証ログ記録（MOD-BE-005）
// auth_layer(): JWT 検証して CurrentUser を Request Extension に注入する
// auth_log_layer(): 認証成功・失敗をトレーシングログに記録する

use axum::{
    body::Body,
    extract::Request,
    http::{
        HeaderMap,
        StatusCode,
        header::{AUTHORIZATION, WWW_AUTHENTICATE},
    },
    middleware::Next,
    response::{IntoResponse, Response},
};

use crate::{current_user::CurrentUser, error::AuthError, jwt::JwtKeyStore};
use std::sync::Arc;

/// Bearer トークンを Authorization ヘッダから抽出するヘルパー。
fn extract_bearer_token(headers: &HeaderMap) -> Result<&str, AuthError> {
    // Authorization ヘッダを取得する
    let auth_header = headers
        .get(AUTHORIZATION)
        .ok_or(AuthError::MissingToken)?
        .to_str()
        .map_err(|_| AuthError::InvalidToken("Authorization header is not valid UTF-8".to_string()))?;

    // "Bearer " プレフィックスを確認してトークン部分を返す
    auth_header
        .strip_prefix("Bearer ")
        .ok_or(AuthError::MissingToken)
}

/// JWT 検証 Tower ミドルウェア（auth_layer）。
///
/// リクエストの Authorization: Bearer ヘッダから JWT を取得し、
/// RS256 で検証して `CurrentUser` を Request Extension に注入する。
/// 検証失敗時は RFC 7807 Problem Details 形式の 401 を返す。
#[tracing::instrument(skip_all, name = "auth_middleware")]
pub async fn auth_middleware(
    axum::extract::State(key_store): axum::extract::State<Arc<JwtKeyStore>>,
    mut req: Request<Body>,
    next: Next,
) -> Response {
    // Bearer トークンを抽出して JWT 検証を行う
    let token = match extract_bearer_token(req.headers()) {
        Ok(t) => t,
        Err(e) => {
            tracing::warn!(event = "auth.missing_token", error = %e);
            let mut response = e.into_response();
            response
                .headers_mut()
                .insert(WWW_AUTHENTICATE, r#"Bearer realm="wnav""#.parse().unwrap());
            return response;
        }
    };

    // JWT を検証して Claims を取得する
    let claims = match key_store.verify(token).await {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!(event = "auth.invalid_token", error = %e);
            let mut response = e.into_response();
            response
                .headers_mut()
                .insert(WWW_AUTHENTICATE, r#"Bearer realm="wnav", error="invalid_token""#.parse().unwrap());
            return response;
        }
    };

    // JWT Claims から CurrentUser を生成して Extension に挿入する
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
        factory_id = %current_user.factory_id,
    );

    req.extensions_mut().insert(current_user);
    next.run(req).await
}

/// 認証ログ記録 Tower ミドルウェア（auth_log_layer）。
///
/// 認証成功・失敗・ロックアウトを構造化ログとして記録する。
/// このミドルウェアは `auth_middleware` の前段に配置する。
#[tracing::instrument(skip_all, name = "auth_log_middleware")]
pub async fn auth_log_middleware(req: Request<Body>, next: Next) -> Response {
    let path = req.uri().path().to_string();
    let method = req.method().to_string();

    // ハンドラ実行前のロギング
    tracing::info!(
        event = "auth.request",
        method = %method,
        path = %path,
    );

    let response = next.run(req).await;
    let status = response.status();

    // レスポンスステータスに基づいて認証結果をログに記録する
    match status {
        StatusCode::OK | StatusCode::CREATED | StatusCode::NO_CONTENT => {
            tracing::info!(
                event = "auth.response.success",
                status = %status,
                path = %path,
            );
        }
        StatusCode::UNAUTHORIZED => {
            tracing::warn!(
                event = "auth.response.unauthorized",
                status = %status,
                path = %path,
            );
        }
        StatusCode::FORBIDDEN => {
            tracing::warn!(
                event = "auth.response.forbidden",
                status = %status,
                path = %path,
            );
        }
        _ => {
            tracing::debug!(
                event = "auth.response",
                status = %status,
                path = %path,
            );
        }
    }

    response
}
