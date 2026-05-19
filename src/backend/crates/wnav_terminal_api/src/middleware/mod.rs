// wnav_terminal_api ミドルウェアチェーン（MOD-BE-001 §2）
//
// ミドルウェアの適用順序:
// Tracing → Auth → RateLimit → Idempotency → Handler
//
// Idempotency は wnav_terminal_api 専用（wnav_master_api には適用しない）

pub mod auth;
pub mod idempotency;
pub mod rate_limit;
pub mod tracing_mw;

use axum::Router;
use std::sync::Arc;
use tower::ServiceBuilder;
use tower_http::{
    cors::{AllowHeaders, AllowMethods, AllowOrigin, CorsLayer},
    trace::TraceLayer,
};

use crate::state::AppState;
use wnav_config::TerminalApiConfig;

/// ミドルウェアチェーンを Router に適用して返す。
///
/// 適用順序（リクエストは上から下へ、レスポンスは逆順に通過する）:
/// 1. TraceLayer（X-Trace-Id・構造化ログ）
/// 2. CorsLayer（CORS ヘッダ付与）
/// 3. AuthMiddleware（JWT 検証・CurrentUser 注入）
/// 4. IdempotencyMiddleware（Idempotency-Key 検証・キャッシュ照合）
///
/// auth_middleware と idempotency_middleware は State<AppState> を使用するため、
/// from_fn_with_state でバインドする必要がある。
/// apply_middleware は main.rs で with_state() より前に呼ばれるため、
/// state を引数として受け取る。
pub fn apply_middleware(
    router: Router<AppState>,
    config: &TerminalApiConfig,
    state: AppState,
) -> Router<AppState> {
    let cors = build_cors_layer(config);

    router.layer(
        ServiceBuilder::new()
            // 1. TraceLayer: X-Trace-Id 付与・構造化ログ出力
            .layer(TraceLayer::new_for_http())
            // 2. CorsLayer: CORS ヘッダ付与
            .layer(cors)
            // 3. AuthMiddleware: JWT 検証・CurrentUser 注入
            // State<AppState> エクストラクタを使用するため from_fn_with_state が必要
            .layer(axum::middleware::from_fn_with_state(
                state.clone(),
                auth::auth_middleware,
            ))
            // 4. IdempotencyMiddleware: Idempotency-Key 検証・キャッシュ照合
            // State<AppState> エクストラクタを使用するため from_fn_with_state が必要
            .layer(axum::middleware::from_fn_with_state(
                state,
                idempotency::idempotency_middleware,
            )),
    )
}

/// CORS ポリシーを設定する（CFG の cors セクションに基づく）。
fn build_cors_layer(config: &TerminalApiConfig) -> CorsLayer {
    let origins: Vec<_> = config
        .shared
        .cors
        .allow_origins
        .iter()
        .filter_map(|s| s.parse().ok())
        .collect();

    let layer = CorsLayer::new()
        .allow_methods(AllowMethods::list([
            axum::http::Method::GET,
            axum::http::Method::POST,
            axum::http::Method::PUT,
            axum::http::Method::DELETE,
            axum::http::Method::OPTIONS,
        ]))
        .allow_headers(AllowHeaders::list([
            axum::http::header::AUTHORIZATION,
            axum::http::header::CONTENT_TYPE,
            axum::http::HeaderName::from_static("idempotency-key"),
            axum::http::HeaderName::from_static("x-trace-id"),
        ]))
        .max_age(std::time::Duration::from_secs(
            config.shared.cors.max_age_sec,
        ));

    if origins.is_empty() {
        // 開発環境向け: オリジン制限なし
        layer.allow_origin(AllowOrigin::any())
    } else {
        layer.allow_origin(AllowOrigin::list(origins))
    }
}

/// レート制限キー抽出（ユーザー ID または IP アドレス）
///
/// 認証済みリクエストはユーザー ID、未認証は IP アドレスをキーとする
pub fn extract_rate_limit_key(
    request: &axum::extract::Request,
    current_user: Option<&wnav_auth::CurrentUser>,
) -> String {
    if let Some(user) = current_user {
        return format!("user:{}", user.user_id);
    }

    // 未認証の場合は X-Forwarded-For または REMOTE_ADDR を使用する
    request
        .headers()
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.split(',').next())
        .map(|ip| format!("ip:{}", ip.trim()))
        .unwrap_or_else(|| "ip:unknown".to_string())
}

/// Bearer トークンを Authorization ヘッダから抽出する
pub fn extract_bearer_token(request: &axum::extract::Request) -> Option<String> {
    request
        .headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .map(String::from)
}

/// 認証不要パスかどうかを判定する
pub fn is_public_path(path: &str) -> bool {
    // /healthz と /api/v1/auth/login は認証不要（MOD-BE-001 §2-2）
    path == "/healthz"
        || path == "/api/v1/healthz"
        || path == "/api/v1/auth/login"
        || path == "/api/v1/auth/refresh"
        || path == "/api/v1/readyz"
        || path == "/api/v1/openapi.json"
}

/// インメモリ レート制限テーブル（トークンバケット方式）
pub struct RateLimiter {
    buckets: std::sync::Mutex<std::collections::HashMap<String, TokenBucket>>,
    /// 毎分の最大リクエスト数（rpm）
    rpm: u32,
}

struct TokenBucket {
    tokens: f64,
    last_refill: std::time::Instant,
}

impl RateLimiter {
    pub fn new(rpm: u32) -> Arc<Self> {
        Arc::new(Self {
            buckets: std::sync::Mutex::new(std::collections::HashMap::new()),
            rpm,
        })
    }

    /// トークンを消費する。成功時は true、レート超過時は false を返す。
    pub fn consume(&self, key: &str) -> bool {
        let mut buckets = self.buckets.lock().expect("RateLimiter mutex poisoned");

        let bucket = buckets
            .entry(key.to_string())
            .or_insert_with(|| TokenBucket {
                tokens: f64::from(self.rpm),
                last_refill: std::time::Instant::now(),
            });

        // 経過時間に応じてトークンを補充する（毎秒 rpm/60 個）
        let elapsed = bucket.last_refill.elapsed();
        let refill = elapsed.as_secs_f64() * (f64::from(self.rpm) / 60.0);
        bucket.tokens = (bucket.tokens + refill).min(f64::from(self.rpm));
        bucket.last_refill = std::time::Instant::now();

        if bucket.tokens >= 1.0 {
            bucket.tokens -= 1.0;
            true
        } else {
            false
        }
    }
}
