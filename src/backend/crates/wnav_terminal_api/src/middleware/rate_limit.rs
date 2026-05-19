// RateLimitMiddleware — トークンバケット方式によるレート制限（MOD-BE-001 §2-3）
//
// CFG-002: デフォルト rpm を設定から読み込む。
// ユーザー ID（認証済み）または IP アドレス（未認証）をキーとする。

use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};

use crate::{error::AppError, state::AppState};

/// トークンバケット方式でレート制限するミドルウェア（MOD-BE-001 §2-3）。
///
/// State<AppState> から RateLimiter を取得して認証済みユーザー ID または IP をキーとする。
/// 超過時は HTTP 429 Too Many Requests（ERR-SYS-002）を返す。
pub async fn rate_limit_middleware(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    use wnav_auth::CurrentUser;

    let current_user = request.extensions().get::<CurrentUser>().cloned();
    let key = crate::middleware::extract_rate_limit_key(&request, current_user.as_ref());

    if !state.rate_limiter.consume(&key) {
        tracing::warn!(
            log_id = "LOG-RATE-001",
            key = %key,
            "レート制限超過"
        );
        return Err(AppError::RateLimited);
    }

    let mut response = next.run(request).await;

    // X-RateLimit ヘッダを付与する（OpenAPI 共通仕様 §6）
    let headers = response.headers_mut();
    if let Ok(v) = state.rate_limiter.rpm().to_string().parse() {
        headers.insert("x-ratelimit-limit", v);
    }
    let reset_time = chrono::Utc::now().timestamp() + 60;
    if let Ok(v) = reset_time.to_string().parse() {
        headers.insert("x-ratelimit-reset", v);
    }

    Ok(response)
}
