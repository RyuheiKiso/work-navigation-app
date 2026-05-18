# 01 wnav_terminal_api 詳細設計（MOD-BE-001）

本章は `crates/wnav_terminal_api/` の axum ルータ・ミドルウェアチェーン・AppState・エラー変換レイヤの詳細設計を確定する。本クレートはハンディ端末（Android / iOS / Windows）専用の Presentation 層実装であり、作業実行・証拠記録・同期・入庫検査・手直し実行・認証に関する API エンドポイントのルーティング・ミドルウェア適用・レスポンス整形を担う。ポート 8080 でリッスンする。

---

## 1. AppState 構造体

AppState はすべてのハンドラと共有される依存注入コンテナである。Arc でラップして axum の Extension として全ハンドラに注入する。

wnav_terminal_api は **イベント挿入専用プール**（`event_insert_pool`）と**読み取りプール**（`read_pool`）の 2 プールのみを保持する。マスタ書き込みプール（`write_pool`）は持たない。これにより DB プール混入をコンパイル時に防止する。

```rust
// crates/wnav_terminal_api/src/state.rs

use std::sync::Arc;
use sqlx::PgPool;
use wnav_auth::AuthState;

/// ハンディ端末向け API の依存コンテナ。
/// axum::Router に `.with_state(state)` で渡す。
/// write_pool は持たない（コンパイル時にマスタ書き込みの混入を防止）。
#[derive(Clone)]
pub struct AppState {
    /// イベント挿入専用プール（app_events・app_event_idempotency_keys への INSERT のみ）
    pub event_insert_pool: PgPool,
    /// 読み取り専用プール（SELECT のみ）
    pub read_pool: PgPool,
    /// JWT 検証・LDAP 認証状態
    pub auth_state: AuthState,
    /// アプリケーション設定
    pub config: Arc<AppConfig>,
}

/// wnav_terminal_api 専用の設定。
#[derive(Debug, serde::Deserialize, Clone)]
pub struct AppConfig {
    /// リッスンポート（デフォルト 8080）
    pub port: u16,
    /// CORS 許可オリジン（カンマ区切り）
    pub cors_allow_origins: String,
    /// JWT 公開鍵 PEM（CFG-006）
    pub jwt_public_key_pem: String,
    /// レート制限: 1 分あたり最大リクエスト数（CFG-002）
    pub rate_limit_rpm: u32,
    /// Idempotency Key TTL 秒（デフォルト 86400）
    pub idempotency_ttl_secs: u64,
    /// イベント挿入用 DB 接続文字列
    pub event_insert_db_url: String,
    /// 読み取り用 DB 接続文字列
    pub read_db_url: String,
    /// トレーシングレベル（"info"・"debug" 等）
    pub tracing_level: String,
}
```

---

## 2. ミドルウェアチェーン

ミドルウェアは tower::ServiceBuilder で順番に積み上げる。リクエストは上から下の順に通過し、レスポンスは逆順に通過する。

```
Tracing → Auth → RateLimit → Idempotency → Handler
```

Idempotency ミドルウェアはハンディ端末向けの二重送信抑止が必要なため **terminal のみ** に適用する。

```rust
// crates/wnav_terminal_api/src/middleware/mod.rs

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
            // 2. AuthMiddleware: aud = "terminal-api" 検証 → CurrentUser extension
            .layer(axum::middleware::from_fn_with_state(
                config.clone(),
                auth_middleware,
            ))
            // 3. RateLimitMiddleware: トークンバケット（CFG-002 rpm）
            .layer(axum::middleware::from_fn_with_state(
                config.clone(),
                rate_limit_middleware,
            ))
            // 4. IdempotencyMiddleware: Idempotency-Key ヘッダ → TBL-035 照合（terminal のみ）
            .layer(axum::middleware::from_fn_with_state(
                config.clone(),
                idempotency_middleware,
            )),
    )
}
```

### 2-1. TracingMiddleware

> TracingMiddleware の実装（`make_trace_span`・`on_request`・`on_response`）は両バイナリで共通であり、`crates/wnav_common/src/middleware/tracing.rs` に置いて再利用する。

```rust
// crates/wnav_terminal_api/src/middleware/tracing.rs

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

### 2-2. AuthMiddleware（terminal-api 専用）

```rust
// crates/wnav_terminal_api/src/middleware/auth.rs

use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use wnav_auth::CurrentUser;

/// JWT を検証し、成功時は CurrentUser を Request Extension に追加する。
/// 失敗時は即座に 401 を返す。
/// `/healthz` と `POST /api/v1/auth/login` は検証をスキップする。
/// クレーム `aud` が "terminal-api" であることを検証する。
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

    // aud = "terminal-api" を検証
    let claims = wnav_auth::verify_jwt_with_audience(
        &token,
        &config.jwt_public_key_pem,
        "terminal-api",
    )
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
    matches!(path, "/healthz" | "/api/v1/auth/login")
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

> `RateLimiter` 構造体・`TokenBucket`・`consume` ロジックは両バイナリで共通であり、`crates/wnav_common/src/middleware/rate_limit.rs` に置いて再利用する。

```rust
// crates/wnav_terminal_api/src/middleware/rate_limit.rs

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

### 2-4. IdempotencyMiddleware（terminal 専用）

ハンディ端末からの書き込みリクエストは、ネットワーク不安定によるリトライが多発するため、idempotency_middleware で二重送信を抑止する。TBL-035（app_event_idempotency_keys）を **event_insert_pool** で照合する。

```rust
// crates/wnav_terminal_api/src/middleware/idempotency.rs

use axum::{extract::Request, middleware::Next, response::Response};

/// 書き込みメソッド（POST/PUT/PATCH/DELETE）に対して
/// Idempotency-Key ヘッダを要求し、TBL-035 で重複チェックを行う。
/// 照合には event_insert_pool を使用する。
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

    // TBL-035 照合（event_insert_pool 使用）: 既存キーならキャッシュを返す
    if let Some(cached) = idempotency_cache_get(
        &state.event_insert_pool,
        idempotency_key,
        state.config.idempotency_ttl_secs,
    )
    .await?
    {
        return Ok(cached.into_response());
    }

    // 新規: リクエスト処理後にレスポンスをキャッシュ
    let response = next.run(request).await;
    if response.status().is_success() {
        idempotency_cache_store(
            &state.event_insert_pool,
            idempotency_key,
            &response,
        )
        .await?;
    }
    Ok(response)
}
```

---

## 3. ルータ定義（ハンディ端末向けエンドポイント）

```rust
// crates/wnav_terminal_api/src/router.rs

use axum::{Router, routing::{get, post}};

/// (FNC-BE-016) wnav_terminal_api ルータを生成して返す。
/// TLS 終端は IIS（リバースプロキシ）が担当するため、本クレートは HTTP のみを受け付ける。
/// 対象エンドポイント: events / evidence / sync / iqc/inspect / rework/execute / auth
pub fn create_router(state: AppState) -> Router {
    let api = Router::new()
        // --- 認証 ---
        .route("/auth/login",   post(handlers::auth::login))

        // --- イベント記録（Event Sourcing INSERT 専用）---
        .route("/events",                             post(handlers::events::create))
        .route("/events/:id",                         get(handlers::events::get_by_id))

        // --- 証拠記録（Evidence Context）---
        .route("/evidence",                           post(handlers::evidence::upload))
        .route("/evidence/:id",                       get(handlers::evidence::get_by_id))

        // --- 同期（Sync Context）---
        .route("/sync/master",                        post(handlers::sync::pull_master))
        .route("/sync/events",                        post(handlers::sync::push_events))
        .route("/sync/status",                        get(handlers::sync::status))

        // --- 入庫検査（IQC Inspect Context）---
        .route("/iqc/inspect",                        post(handlers::iqc::create_inspection))
        .route("/iqc/inspect/:id",                    get(handlers::iqc::get_inspection))
        .route("/iqc/inspect/:id/result",             post(handlers::iqc::record_result))

        // --- 手直し実行（Rework Execute Context）---
        .route("/rework/execute",                     post(handlers::rework::create_execution))
        .route("/rework/execute/:id",                 get(handlers::rework::get_execution))
        .route("/rework/execute/:id/complete",        post(handlers::rework::complete))

        // --- システム ---
        .route("/healthz",                            get(handlers::health::healthz));

    Router::new()
        .nest("/api/v1", api)
        .with_state(state)
}
```

---

## 4. エラー変換レイヤ（AppError → Problem Details RFC 9457）

> `ProblemDetails` 構造体および `IntoResponse for AppError` の実装は `wnav_terminal_api` と `wnav_master_api` で共通である。実装は `crates/wnav_common/src/error.rs` に置き、両バイナリから参照する（`09_共通ライブラリ詳細設計.md` §4 参照）。以下は設計仕様の参考実装を示す。

```rust
// crates/wnav_terminal_api/src/error.rs

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
// crates/wnav_terminal_api/src/cors.rs

use tower_http::cors::{CorsLayer, AllowOrigin};
use axum::http::{Method, HeaderName};

pub fn cors_layer(allow_origins: &str) -> CorsLayer {
    let origins: Vec<_> = allow_origins
        .split(',')
        .map(|s| s.trim().parse().expect("不正なオリジン"))
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

TLS 終端は IIS（Windows Server 2022）がリバースプロキシとして担当する。本クレートは HTTP（ポート 8080）でリッスンし、IIS から転送されるリクエストを受け付ける。エンドユーザーは IIS を通じて HTTPS でアクセスする。

---

## 6. エントリポイント（main.rs）

wnav_terminal_api の main.rs は **event_insert_pool** と **read_pool** のみを生成・注入する。write_pool は生成しない。

```rust
// crates/wnav_terminal_api/src/main.rs

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 設定ロード（環境変数）
    let config: AppConfig = envy::from_env()?;

    // tracing 初期化
    init_tracing(&config.tracing_level);

    // イベント挿入専用プール（event_insert_pool）生成
    let event_insert_pool = wnav_db::connect_event_insert(
        &config.event_insert_db_url,
    )
    .await?;

    // 読み取り専用プール（read_pool）生成
    let read_pool = wnav_db::connect_read_only(
        &config.read_db_url,
    )
    .await?;

    // AppState 構築（write_pool は持たない）
    let auth_state = wnav_auth::init_auth_state(&config.jwt_public_key_pem)?;
    let state = AppState {
        event_insert_pool,
        read_pool,
        auth_state,
        config: Arc::new(config.clone()),
    };

    // ルータ構築
    let app = create_router(state.clone());
    let app = apply_middleware(app, &config);
    let app = app.layer(cors_layer(&config.cors_allow_origins));

    // Outbox Consumer 起動（BAT-002）
    // event_insert_pool からイベントを読み取り Webhook 等に配信する
    let outbox_handle = tokio::spawn(
        wnav_outbox::run_consumer(state.event_insert_pool.clone())
    );

    // HTTP サーバー起動
    let listener = tokio::net::TcpListener::bind(
        format!("0.0.0.0:{}", config.port)
    ).await?;
    tracing::info!(port = config.port, "wnav_terminal_api started");

    tokio::select! {
        result = axum::serve(listener, app) => result?,
        _ = outbox_handle => tracing::error!("outbox_consumer が予期せず終了しました"),
    }

    Ok(())
}
```

---

**本節で確定した方針**
- **AppState は event_insert_pool + read_pool の 2 プールのみを保持し、write_pool をコンパイル時に排除することでマスタ書き込みの混入を防止する。**
- **ミドルウェアチェーンは Tracing → Auth → RateLimit → Idempotency の順で適用し、Idempotency はハンディ端末特有の二重送信対策として terminal のみに適用する。**
- **auth_middleware は aud = "terminal-api" クレームを検証し、/healthz と /api/v1/auth/login のみ認証をスキップする。**
- **idempotency_middleware は TBL-035 を event_insert_pool で照合する。**
- **main.rs は event_insert_pool + read_pool のみを生成・注入し、OutboxWorker（BAT-002）を tokio::spawn で起動する。**
- **TLS 終端は IIS に委譲し、本クレートは HTTP（ポート 8080）リッスンのみとすることを確定した。**

---

## 参照業界分析

### 必須
- [`90_業界分析/09_セキュリティとアクセス制御.md`](../../90_業界分析/09_セキュリティとアクセス制御.md)

### 関連
- [`90_業界分析/06_品質管理とトレーサビリティ.md`](../../90_業界分析/06_品質管理とトレーサビリティ.md)
