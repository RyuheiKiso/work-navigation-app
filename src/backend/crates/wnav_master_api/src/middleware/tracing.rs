// 構造化ログ・X-Request-Id 付与ミドルウェア（TracingMiddleware）
//
// 各リクエストに UUID v7 の X-Request-Id を付与し、
// 受信・送信をそれぞれ JSON 構造化ログとして出力する（LOG-001 / LOG-002）。

use axum::{body::Body, extract::Request, http::HeaderValue, middleware::Next, response::Response};
use uuid::Uuid;

/// X-Request-Id 付与・構造化ログ出力ミドルウェア。
///
/// 既存の X-Request-Id ヘッダがある場合はそれを使用し、
/// ない場合は UUID v7 を新規採番してリクエスト・レスポンス両方に付与する。
pub async fn request_id_middleware(mut req: Request<Body>, next: Next) -> Response {
    // 既存の X-Request-Id を取得するか新規に採番する
    let request_id = req
        .headers()
        .get("x-request-id")
        .and_then(|v| v.to_str().ok())
        .map(ToString::to_string)
        .unwrap_or_else(|| Uuid::now_v7().to_string());

    // リクエストのメタデータを構造化ログに記録する
    tracing::info!(
        log_id = "LOG-001",
        event_name = "api.request.received",
        request_id = %request_id,
        method = %req.method(),
        path = %req.uri().path(),
        "API リクエストを受信しました",
    );

    // X-Request-Id ヘッダをリクエストに付与する（downstream への伝播）
    if let Ok(value) = HeaderValue::from_str(&request_id) {
        req.headers_mut().insert("x-request-id", value);
    }

    let start = std::time::Instant::now();
    let mut response = next.run(req).await;
    let latency_ms = start.elapsed().as_millis();

    // レスポンスのメタデータを構造化ログに記録する
    tracing::info!(
        log_id = "LOG-002",
        event_name = "api.response.sent",
        request_id = %request_id,
        status = response.status().as_u16(),
        latency_ms = latency_ms,
        "API レスポンスを送信しました",
    );

    // X-Request-Id をレスポンスヘッダにも付与する（クライアントが相関 ID を取得できるようにする）
    if let Ok(value) = HeaderValue::from_str(&request_id) {
        response.headers_mut().insert("x-request-id", value);
    }

    response
}
