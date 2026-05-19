// AuthMiddleware — JWT RS256 検証・CurrentUser 注入（MOD-BE-001 §2-2）
//
// aud = "terminal-api" を検証し、/healthz と /api/v1/auth/* はスキップする。
// JWT jti のブラックリストチェックは read_pool で行う。

use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};

use crate::{error::AppError, middleware::is_public_path, state::AppState};
use wnav_auth::CurrentUser;

/// JWT を検証し、成功時は CurrentUser を Request Extension に追加するミドルウェア。
///
/// 失敗時は即座に AppError::Unauthorized を返す。
/// `/healthz` と `/api/v1/auth/login` は検証をスキップする。
/// aud クレームが "terminal-api" であることを検証する（MOD-BE-001 §2-2）。
pub async fn auth_middleware(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let path = request.uri().path();

    // 認証スキップパスは素通りさせる
    if is_public_path(path) {
        return Ok(next.run(request).await);
    }

    // Authorization ヘッダから Bearer トークンを抽出する
    let token = crate::middleware::extract_bearer_token(&request).ok_or(AppError::Unauthorized)?;

    // JWT を検証して Claims を取得する（aud = "terminal-api" を確認する）
    let claims = state.jwt_key_store.verify(&token).await.map_err(|e| {
        tracing::warn!(
            log_id = "LOG-003",
            error = %e,
            "JWT 検証に失敗した"
        );
        AppError::Unauthorized
    })?;

    // jti ブラックリストチェック（ログアウト済みトークンの拒否）は
    // 本番では read_pool で TBL-032 を確認する。
    // 現時点では JWT の署名・有効期限・aud の検証のみ実施する。

    // 検証済みユーザー情報を Request Extension に注入する
    let current_user = CurrentUser {
        user_id: claims.sub,
        roles: claims.roles,
        factory_id: claims.factory_id,
        device_id: claims.device_id,
        jti: claims.jti,
    };

    request.extensions_mut().insert(current_user);
    Ok(next.run(request).await)
}
