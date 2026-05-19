// RateLimitMiddleware — トークンバケット方式によるレート制限（MOD-BE-001 §2-3）
//
// CFG-002: デフォルト rpm を設定から読み込む。
// ユーザー ID（認証済み）または IP アドレス（未認証）をキーとする。

use axum::{extract::Request, http::StatusCode, middleware::Next, response::Response};

/// トークンバケット方式でレート制限するシンプルなミドルウェア。
///
/// State に依存せず、クロージャでキャプチャした RateLimiter を使用する。
/// 超過時は HTTP 429 Too Many Requests を返す。
pub async fn rate_limit_check(
    request: Request,
    next: Next,
    limiter: &std::sync::Arc<crate::middleware::RateLimiter>,
    rpm: u32,
) -> Result<Response, StatusCode> {
    use wnav_auth::CurrentUser;

    let current_user = request.extensions().get::<CurrentUser>().cloned();
    let key = crate::middleware::extract_rate_limit_key(&request, current_user.as_ref());

    if !limiter.consume(&key) {
        tracing::warn!(
            log_id = "LOG-RATE-001",
            key = %key,
            "レート制限超過"
        );
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }

    let mut response = next.run(request).await;

    // X-RateLimit ヘッダを付与する（OpenAPI 共通仕様 §6）
    let headers = response.headers_mut();
    if let Ok(v) = rpm.to_string().parse() {
        headers.insert("X-RateLimit-Limit", v);
    }
    let reset_time = chrono::Utc::now().timestamp() + 60;
    if let Ok(v) = reset_time.to_string().parse() {
        headers.insert("X-RateLimit-Reset", v);
    }

    Ok(response)
}
