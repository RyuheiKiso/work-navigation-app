// TracingMiddleware — X-Trace-Id 付与・構造化ログ出力（MOD-BE-001 §2-1）
//
// X-Trace-Id が存在する場合はそれを trace_id として使用し、
// 存在しない場合はサーバー側で UUID v7 を採番する。

use axum::{extract::Request, middleware::Next, response::Response};
use uuid::Uuid;

/// X-Trace-Id を付与してリクエスト/レスポンスを構造化ログに記録するミドルウェア。
pub async fn tracing_middleware(mut request: Request, next: Next) -> Response {
    // X-Trace-Id ヘッダが存在する場合はそれを使用し、なければ UUID v7 を採番する
    let trace_id = request
        .headers()
        .get("x-trace-id")
        .and_then(|v| v.to_str().ok())
        .map(String::from)
        .unwrap_or_else(|| Uuid::now_v7().to_string());

    let method = request.method().to_string();
    let path = request.uri().path().to_string();

    // リクエスト受信ログを出力する
    tracing::info!(
        log_id = "LOG-001",
        trace_id = %trace_id,
        method = %method,
        path = %path,
        "api.request.received"
    );

    // trace_id をリクエスト Extension に追加して下流ハンドラで参照できるようにする
    request.extensions_mut().insert(TraceId(trace_id.clone()));

    let start = std::time::Instant::now();
    let response = next.run(request).await;
    let latency_ms = start.elapsed().as_millis();

    // レスポンス送信ログを出力する
    tracing::info!(
        log_id = "LOG-002",
        trace_id = %trace_id,
        status = response.status().as_u16(),
        latency_ms = %latency_ms,
        "api.response.sent"
    );

    response
}

/// リクエスト Extension に保存する Trace ID ラッパー型
#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct TraceId(pub String);
