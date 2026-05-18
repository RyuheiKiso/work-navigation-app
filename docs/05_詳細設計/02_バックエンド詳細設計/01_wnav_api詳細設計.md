# 01 wnav_api 詳細設計（MOD-BE-001）

本章は `crates/wnav_api/` の axum ルータ・ミドルウェアチェーン・AppState・エラー変換レイヤの詳細設計を確定する。本クレートは Presentation 層の唯一の実装であり、全 39 API エンドポイントのルーティング・ミドルウェア適用・レスポンス整形を担う。

---

## 1. AppState 構造体

AppState はすべてのハンドラと共有される依存注入コンテナである。Arc でラップして axum の Extension として全ハンドラに注入する。

```rust
// crates/wnav_api/src/state.rs

use std::sync::Arc;
use wnav_auth::AuthService;
use wnav_domain::service::{
    WorkExecutionService, MasterService, EvidenceService,
    AndonService, ReportService,
};
use wnav_outbox::OutboxConsumer;
use wnav_hash_chain::HashChainService;

/// アプリケーション全体の依存コンテナ。
/// axum::Router に `.with_state(state)` で渡す。
#[derive(Clone)]
pub struct AppState {
    /// 作業実行ユースケース（FNC-BE-001〜004 を含む）
    pub work_execution_svc: Arc<dyn WorkExecutionService>,
    /// マスタ管理ユースケース
    pub master_svc: Arc<dyn MasterService>,
    /// 証拠記録ユースケース
    pub evidence_svc: Arc<dyn EvidenceService>,
    /// アンドン・不適合ユースケース
    pub andon_svc: Arc<dyn AndonService>,
    /// 帳票生成ユースケース
    pub report_svc: Arc<dyn ReportService>,
    /// JWT 検証・LDAP 認証
    pub auth_svc: Arc<dyn AuthService>,
    /// ハッシュチェーン検証
    pub hash_chain_svc: Arc<HashChainService>,
    /// Outbox Consumer（管理コンソール向け DLQ 再投入）
    pub outbox_consumer: Arc<OutboxConsumer>,
    /// アプリケーション設定
    pub config: Arc<AppConfig>,
}

// AppConfig は wnav_config クレートに移管（ADR-IMPL-001）
// TerminalApiConfig / MasterApiConfig として各バイナリが必要なサブツリーのみを保持する。
// 詳細: docs/05_詳細設計/02_バックエンド詳細設計/10_wnav_config詳細設計.md

// wnav_terminal_api が使用する設定型
pub use wnav_config::TerminalApiConfig;
// wnav_master_api が使用する設定型
pub use wnav_config::MasterApiConfig;
```

---

## 2. ミドルウェアチェーン

ミドルウェアは tower::ServiceBuilder で順番に積み上げる。リクエストは上から下の順に通過し、レスポンスは逆順に通過する。

```rust
// crates/wnav_api/src/middleware/mod.rs

use axum::Router;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;

pub fn apply_middleware(router: Router<AppState>, config: &AppConfig) -> Router {
    router.layer(
        ServiceBuilder::new()
            // 1. TracingMiddleware: X-Trace-Id 付与・構造化ログ出力
            .layer(TraceLayer::new_for_http()
                .make_span_with(make_trace_span)
                .on_request(on_request)
                .on_response(on_response))
            // 2. AuthMiddleware: Authorization ヘッダの JWT 検証 → CurrentUser extension
            .layer(axum::middleware::from_fn_with_state(
                config.clone(),
                auth_middleware,
            ))
            // 3. RateLimitMiddleware: トークンバケット（CFG-002 rpm）
            .layer(axum::middleware::from_fn_with_state(
                config.clone(),
                rate_limit_middleware,
            ))
            // 4. IdempotencyMiddleware: Idempotency-Key ヘッダ → TBL-035 照合
            .layer(axum::middleware::from_fn_with_state(
                config.clone(),
                idempotency_middleware,
            )),
    )
}
```

### 2-1. TracingMiddleware

```rust
// crates/wnav_api/src/middleware/tracing.rs

use axum::{extract::Request, response::Response};
use tracing::Span;
use uuid::Uuid;

fn make_trace_span(request: &Request) -> Span {
    let trace_id = request
        .headers()
        .get("X-Trace-Id")
        .and_then(|v| v.to_str().ok())
        .map(String::from)
        .unwrap_or_else(|| Uuid::now_v7().to_string());

    tracing::info_span!(
        "http_request",
        log_id = "LOG-001",
        trace_id = %trace_id,
        method = %request.method(),
        path = %request.uri().path(),
    )
}

fn on_request(request: &Request, _span: &Span) {
    tracing::info!(
        log_id = "LOG-001",
        event_name = "api.request.received",
        content_length = ?request.headers().get("content-length"),
    );
}

fn on_response(response: &Response, latency: std::time::Duration, _span: &Span) {
    tracing::info!(
        log_id = "LOG-002",
        event_name = "api.response.sent",
        status = response.status().as_u16(),
        latency_ms = latency.as_millis(),
    );
}
```

### 2-2. AuthMiddleware

```rust
// crates/wnav_api/src/middleware/auth.rs

use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use wnav_auth::CurrentUser;

/// JWT を検証し、成功時は CurrentUser を Request Extension に追加する。
/// 失敗時は即座に 401 を返す。
/// `/healthz`・`POST /auth/login`・`POST /auth/refresh` は検証をスキップする。
pub async fn auth_middleware(
    State(config): State<AppConfig>,
    mut request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let path = request.uri().path();

    // 認証スキップパス
    if is_public_path(path) {
        return Ok(next.run(request).await);
    }

    let token = extract_bearer_token(&request)
        .ok_or(AppError::Unauthorized)?;

    let claims = wnav_auth::verify_jwt(&token, &config.jwt_public_key_pem)
        .map_err(|_| AppError::JwtExpired)?;

    let current_user = CurrentUser {
        user_id: claims.sub,
        roles: claims.roles,
        factory_id: claims.factory_id,
        device_id: claims.device_id,
    };

    request.extensions_mut().insert(current_user);
    Ok(next.run(request).await)
}

fn is_public_path(path: &str) -> bool {
    matches!(path,
        "/healthz" | "/api/v1/auth/login" | "/api/v1/auth/refresh"
    )
}

fn extract_bearer_token(request: &Request) -> Option<String> {
    request
        .headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .map(String::from)
}
```

### 2-3. RateLimitMiddleware

```rust
// crates/wnav_api/src/middleware/rate_limit.rs

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use axum::{extract::Request, middleware::Next, response::Response};

/// トークンバケットアルゴリズムによるレート制限。
/// CFG-002: デフォルト 300 rpm（1 分あたり 300 リクエスト）
/// ユーザー ID（認証済み）または IP アドレス（未認証）をキーとする。
pub struct RateLimiter {
    buckets: Mutex<HashMap<String, TokenBucket>>,
    rpm: u32,
}

struct TokenBucket {
    tokens: f64,
    last_refill: Instant,
}

impl RateLimiter {
    pub fn new(rpm: u32) -> Arc<Self> {
        Arc::new(Self {
            buckets: Mutex::new(HashMap::new()),
            rpm,
        })
    }

    /// トークンを消費する。成功時は true、レート超過時は false を返す。
    pub fn consume(&self, key: &str) -> bool {
        let mut buckets = self.buckets.lock().unwrap();
        let bucket = buckets.entry(key.to_string()).or_insert_with(|| TokenBucket {
            tokens: self.rpm as f64,
            last_refill: Instant::now(),
        });

        let elapsed = bucket.last_refill.elapsed();
        let refill = elapsed.as_secs_f64() * (self.rpm as f64 / 60.0);
        bucket.tokens = (bucket.tokens + refill).min(self.rpm as f64);
        bucket.last_refill = Instant::now();

        if bucket.tokens >= 1.0 {
            bucket.tokens -= 1.0;
            true
        } else {
            false
        }
    }
}

pub async fn rate_limit_middleware(
    State(limiter): State<Arc<RateLimiter>>,
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let key = extract_rate_limit_key(&request);
    if !limiter.consume(&key) {
        return Err(AppError::RateLimited);
    }
    Ok(next.run(request).await)
}
```

### 2-4. IdempotencyMiddleware

```rust
// crates/wnav_api/src/middleware/idempotency.rs

use axum::{extract::Request, middleware::Next, response::Response};

/// 書き込みメソッド（POST/PUT/PATCH/DELETE）に対して
/// Idempotency-Key ヘッダを要求し、TBL-035 で重複チェックを行う。
/// 既存の同一キーが存在する場合はキャッシュされたレスポンスを返す。
pub async fn idempotency_middleware(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let method = request.method();

    // 読み取り専用メソッドはスキップ
    if method.is_safe() {
        return Ok(next.run(request).await);
    }

    let idempotency_key = request
        .headers()
        .get("Idempotency-Key")
        .and_then(|v| v.to_str().ok())
        .ok_or(AppError::MissingIdempotencyKey)?;

    // TBL-035 照合: 既存キーならキャッシュを返す
    if let Some(cached) = state.idempotency_cache
        .get(idempotency_key)
        .await? {
        return Ok(cached.into_response());
    }

    // 新規: リクエスト処理後にレスポンスをキャッシュ
    let response = next.run(request).await;
    if response.status().is_success() {
        state.idempotency_cache
            .store(idempotency_key, &response)
            .await?;
    }
    Ok(response)
}
```

---

## 3. ルータ定義（全 39 エンドポイント）

```rust
// crates/wnav_api/src/router.rs

use axum::{Router, routing::{get, post, put, delete}};

/// (FNC-BE-016) アプリケーションルータを生成して返す。
/// TLS 終端は IIS（リバースプロキシ）が担当するため、本クレートでは HTTP のみを受け付ける。
pub fn create_router(state: AppState) -> Router {
    let api = Router::new()
        // --- 認証 ---
        .route("/auth/login",   post(handlers::auth::login))
        .route("/auth/refresh", post(handlers::auth::refresh))
        .route("/auth/logout",  post(handlers::auth::logout))

        // --- 作業実行（Execution Context）---
        .route("/work-executions",                            post(handlers::work_execution::create))
        .route("/work-executions",                            get(handlers::work_execution::list))
        .route("/work-executions/:id",                        get(handlers::work_execution::get_by_id))
        .route("/work-executions/:id/events",                 post(handlers::work_execution::record_event))
        .route("/work-executions/:id/suspend",                post(handlers::work_execution::suspend))
        .route("/work-executions/:id/resume",                 post(handlers::work_execution::resume))
        .route("/work-executions/:id/complete",               post(handlers::work_execution::complete))

        // --- マスタ管理（Authoring Context）---
        .route("/master-versions",                            get(handlers::master::list))
        .route("/master-versions",                            post(handlers::master::create))
        .route("/master-versions/:id",                        get(handlers::master::get_by_id))
        .route("/master-versions/:id",                        put(handlers::master::update))
        .route("/master-versions/:id/submit-for-review",      post(handlers::master::submit_for_review))
        .route("/master-versions/:id/approve",                post(handlers::master::approve))
        .route("/master-versions/:id/publish",                post(handlers::master::publish))
        .route("/master-versions/:id/archive",                post(handlers::master::archive))
        .route("/master-versions/:id/diff",                   get(handlers::master::diff))

        // --- 証拠記録（Evidence Context）---
        .route("/evidence-files",                             post(handlers::evidence::upload))
        .route("/electronic-signs",                           post(handlers::evidence::sign))

        // --- アンドン・不適合（Quality Context）---
        .route("/andon-alerts",                               post(handlers::andon::create))
        .route("/andon-alerts/:id/acknowledge",               post(handlers::andon::acknowledge))
        .route("/andon-alerts/:id/resolve",                   post(handlers::andon::resolve))
        .route("/nonconformances",                            post(handlers::nonconformance::create))
        .route("/nonconformances/:id",                        put(handlers::nonconformance::update))
        .route("/capas",                                      post(handlers::capa::create))
        .route("/capas/:id/close",                            post(handlers::capa::close))

        // --- ユーザー・ロール管理 ---
        .route("/users",                                      get(handlers::user::list))
        .route("/users",                                      post(handlers::user::create))
        .route("/users/:id",                                  put(handlers::user::update))
        .route("/users/:id",                                  delete(handlers::user::deactivate))

        // --- 帳票・レポート ---
        .route("/reports/work-summary",                       post(handlers::report::work_summary))
        .route("/reports/audit-xes",                          post(handlers::report::audit_xes))

        // --- 運用・管理 ---
        .route("/ops/outbox/dlq",                             get(handlers::ops::list_dlq))
        .route("/ops/outbox/:id/requeue",                     post(handlers::ops::requeue))
        .route("/ops/hash-chain/verify",                      post(handlers::ops::verify_hash_chain))
        .route("/ops/master-sync",                            post(handlers::ops::trigger_master_sync))

        // --- システム ---
        .route("/healthz",                                    get(handlers::health::healthz));

    Router::new()
        .nest("/api/v1", api)
        .with_state(state)
}
```

---

## 4. エラー変換レイヤ（AppError → Problem Details RFC 9457）

```rust
// crates/wnav_api/src/error.rs

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use uuid::Uuid;

/// RFC 9457 Problem Details レスポンスボディ
#[derive(Debug, Serialize)]
pub struct ProblemDetails {
    /// エラー種別 URI（例: "https://errors.wnav.example.com/ERR-AUTH-001"）
    #[serde(rename = "type")]
    pub type_: String,
    /// エラー名（英語・機械可読）
    pub title: String,
    /// HTTP ステータスコード
    pub status: u16,
    /// ユーザー向けメッセージ（多言語対応済み文字列）
    pub detail: String,
    /// リクエスト識別子（"/requests/{uuid}"）
    pub instance: String,
    /// ERR-NNN 識別子（例: "ERR-AUTH-001"）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_id: Option<String>,
}

impl ProblemDetails {
    pub fn new(err: &AppError, request_id: &Uuid) -> Self {
        Self {
            type_: format!(
                "https://errors.wnav.example.com/{}",
                err.error_code()
            ),
            title: err.title(),
            status: err.status_code().as_u16(),
            detail: err.user_message(),
            instance: format!("/requests/{}", request_id),
            error_id: Some(err.error_code().to_string()),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let request_id = Uuid::now_v7();
        let status = self.status_code();
        let body = ProblemDetails::new(&self, &request_id);

        // エラーレベルに応じたログ出力
        match self.log_level() {
            LogLevel::Error => tracing::error!(
                log_id = %self.log_id(),
                error_code = %self.error_code(),
                detail = %self.user_message(),
            ),
            LogLevel::Warn => tracing::warn!(
                log_id = %self.log_id(),
                error_code = %self.error_code(),
            ),
            LogLevel::Info => tracing::info!(
                log_id = %self.log_id(),
                error_code = %self.error_code(),
            ),
        }

        (
            status,
            [("Content-Type", "application/problem+json")],
            Json(body),
        )
            .into_response()
    }
}
```

---

## 5. CORS 設定

```rust
// crates/wnav_api/src/cors.rs

use tower_http::cors::{CorsLayer, AllowOrigin};
use axum::http::{Method, HeaderName};

pub fn cors_layer(allow_origins: &str) -> CorsLayer {
    let origins: Vec<_> = allow_origins
        .split(',')
        .map(|s| s.trim().parse().expect("Invalid origin"))
        .collect();

    CorsLayer::new()
        .allow_origin(AllowOrigin::list(origins))
        .allow_methods([
            Method::GET, Method::POST, Method::PUT,
            Method::DELETE, Method::OPTIONS,
        ])
        .allow_headers([
            HeaderName::from_static("authorization"),
            HeaderName::from_static("content-type"),
            HeaderName::from_static("idempotency-key"),
            HeaderName::from_static("x-trace-id"),
        ])
        .max_age(std::time::Duration::from_secs(3600))
}
```

### 5-1. TLS 終端に関する注記

TLS 終端は IIS（Windows Server 2022）がリバースプロキシとして担当する。本クレートは HTTP（ポート 8080）でリッスンし、IIS から転送されるリクエストを受け付ける。エンドユーザーは IIS を通じて HTTPS でアクセスする。Docker Compose 環境では IIS の前段に nginx または本クレート直接で HTTPS を終端するか、環境変数 `TLS_CERT_PATH`・`TLS_KEY_PATH` を設定することでオプションの `rustls` TLS ハンドラを有効化できる。

---

## 6. エントリポイント（main.rs）

```rust
// crates/wnav_api/src/main.rs

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 設定ロード（YAML + 環境変数オーバーレイ・ADR-IMPL-001）
    // WNAV_PROFILE が未設定または YAML 欠損の場合は exit code 78 で即時終了
    let config: TerminalApiConfig = wnav_config::load_terminal_api()?;

    // tracing 初期化
    init_tracing(&config.tracing_level);

    // DB コネクションプール（CFG-001）
    let db_pool = wnav_db::connect(&config.database_url, &config.db_config).await?;

    // 依存オブジェクト構築
    let state = build_app_state(db_pool, &config).await?;

    // ルータ構築
    let app = create_router(state.clone());
    let app = apply_middleware(app, &config);
    let app = app.layer(cors_layer(&config.cors_allow_origins));

    // Outbox Consumer 起動（BAT-002）
    let outbox_handle = tokio::spawn(
        wnav_outbox::run_consumer(state.outbox_consumer.clone())
    );

    // ハッシュチェーン週次検証スケジューラ起動（BAT-001）
    let hash_verify_handle = tokio::spawn(
        wnav_hash_chain::run_weekly_verifier(state.hash_chain_svc.clone())
    );

    // HTTP サーバー起動
    let listener = tokio::net::TcpListener::bind(
        format!("0.0.0.0:{}", config.port)
    ).await?;
    tracing::info!(port = config.port, "wnav_api started");

    tokio::select! {
        result = axum::serve(listener, app) => result?,
        _ = outbox_handle => tracing::error!("outbox_consumer exited unexpectedly"),
        _ = hash_verify_handle => tracing::error!("hash_chain_verifier exited unexpectedly"),
    }

    Ok(())
}
```

---

**本節で確定した方針**
- **ミドルウェアチェーンは Tracing → Auth → RateLimit → Idempotency の順で適用し、認証失敗は RateLimit に到達する前に 401 を返す設計を確定した。**
- **全 39 エンドポイントを `/api/v1` プレフィックス下に集約し、リソース単位でハンドラモジュールを分割する構造を確定した。**
- **TLS 終端は IIS に委譲し、本クレートは HTTP リッスンのみとすることを確定した。**

---

## 参照業界分析

### 必須
- [`90_業界分析/09_セキュリティとアクセス制御.md`](../../90_業界分析/09_セキュリティとアクセス制御.md)

### 関連
- [`90_業界分析/06_品質管理とトレーサビリティ.md`](../../90_業界分析/06_品質管理とトレーサビリティ.md)
