# 02 wnav_master_api 詳細設計（MOD-BE-010）

本章は `crates/wnav_master_api/` の axum ルータ・ミドルウェアチェーン・AppState・エラー変換レイヤの詳細設計を確定する。本クレートはマスタメンテナンス（React Web APP）および管理コンソール向けの Presentation 層実装であり、SOP・マスタ管理・承認フロー・ユーザー管理・監査・帳票・運用・トレーサビリティ・アラート・CAPA・改善提案・入庫検査判定・特採・手直し指示・廃棄・返品・認証に関する API エンドポイントのルーティング・ミドルウェア適用・レスポンス整形を担う。ポート 8081 でリッスンする。

---

## 1. AppState 構造体

AppState はすべてのハンドラと共有される依存注入コンテナである。Arc でラップして axum の Extension として全ハンドラに注入する。

wnav_master_api は **マスタ書き込みプール**（`write_pool`）と**読み取りプール**（`read_pool`）の 2 プールのみを保持する。イベント挿入専用プール（`event_insert_pool`）は持たない。これにより DB プール混入をコンパイル時に防止する。

```rust
// crates/wnav_master_api/src/state.rs

use std::sync::Arc;
use sqlx::PgPool;
use wnav_auth::AuthState;

/// マスタメンテ・管理コンソール向け API の依存コンテナ。
/// axum::Router に `.with_state(state)` で渡す。
/// event_insert_pool は持たない（コンパイル時にイベント挿入の混入を防止）。
#[derive(Clone)]
pub struct AppState {
    /// マスタ書き込みプール（SOP・マスタ・ユーザー等への INSERT / UPDATE / DELETE）
    pub write_pool: PgPool,
    /// 読み取り専用プール（SELECT のみ）
    pub read_pool: PgPool,
    /// JWT 検証・LDAP 認証状態
    pub auth_state: AuthState,
    /// アプリケーション設定
    pub config: Arc<AppConfig>,
}

/// wnav_master_api 専用の設定。
#[derive(Debug, serde::Deserialize, Clone)]
pub struct AppConfig {
    /// リッスンポート（デフォルト 8081）
    pub port: u16,
    /// CORS 許可オリジン（カンマ区切り）
    pub cors_allow_origins: String,
    /// JWT 公開鍵 PEM（CFG-006）
    pub jwt_public_key_pem: String,
    /// レート制限: 1 分あたり最大リクエスト数（CFG-002）
    pub rate_limit_rpm: u32,
    /// マスタ書き込み用 DB 接続文字列
    pub write_db_url: String,
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
Tracing → Auth → RateLimit → Handler
```

Idempotency ミドルウェアは適用しない。管理系リクエストはブラウザから操作されるため、ハンディ端末のような二重送信リスクがなく、誤ってキャッシュされた古いレスポンスが返ることを防ぐ。

```rust
// crates/wnav_master_api/src/middleware/mod.rs

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
            // 2. AuthMiddleware: aud = "master-api" 検証 → CurrentUser extension
            .layer(axum::middleware::from_fn_with_state(
                config.clone(),
                auth_middleware,
            ))
            // 3. RateLimitMiddleware: トークンバケット（CFG-002 rpm）
            // IdempotencyMiddleware は master-api には適用しない
            .layer(axum::middleware::from_fn_with_state(
                config.clone(),
                rate_limit_middleware,
            )),
    )
}
```

### 2-1. TracingMiddleware

> TracingMiddleware の実装（`make_trace_span`・`on_request`・`on_response`）は両バイナリで共通であり、`crates/wnav_common/src/middleware/tracing.rs` に置いて再利用する。

```rust
// crates/wnav_master_api/src/middleware/tracing.rs

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

### 2-2. AuthMiddleware（master-api 専用）

```rust
// crates/wnav_master_api/src/middleware/auth.rs

use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use wnav_auth::CurrentUser;

/// JWT を検証し、成功時は CurrentUser を Request Extension に追加する。
/// 失敗時は即座に 401 を返す。
/// クレーム `aud` が "master-api" であることを検証する。
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

    // aud = "master-api" を検証
    let claims = wnav_auth::verify_jwt_with_audience(
        &token,
        &config.jwt_public_key_pem,
        "master-api",
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
// crates/wnav_master_api/src/middleware/rate_limit.rs

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

---

## 3. ルータ定義（マスタメンテ・管理コンソール向けエンドポイント）

```rust
// crates/wnav_master_api/src/router.rs

use axum::{Router, routing::{get, post, put, delete}};

/// wnav_master_api ルータを生成して返す。
/// TLS 終端は IIS（リバースプロキシ）が担当するため、本クレートは HTTP のみを受け付ける。
/// 対象エンドポイント:
///   sop / master / approval / users / audit / reports / ops /
///   trace / alerts / capas / kaizen-proposals /
///   iqc/judge / iqc/concession / rework/disposition / scrap / return / auth
pub fn create_router(state: AppState) -> Router {
    let api = Router::new()
        // --- 認証 ---
        .route("/auth/login",   post(handlers::auth::login))
        .route("/auth/refresh", post(handlers::auth::refresh))
        .route("/auth/logout",  post(handlers::auth::logout))

        // --- SOP（Standard Operating Procedure）管理 ---
        .route("/sop",                                post(handlers::sop::create))
        .route("/sop",                                get(handlers::sop::list))
        .route("/sop/:id",                            get(handlers::sop::get_by_id))
        .route("/sop/:id",                            put(handlers::sop::update))
        .route("/sop/:id/publish",                    post(handlers::sop::publish))
        .route("/sop/:id/archive",                    post(handlers::sop::archive))

        // --- マスタ管理（Authoring Context）---
        .route("/master",                             get(handlers::master::list))
        .route("/master",                             post(handlers::master::create))
        .route("/master/:id",                         get(handlers::master::get_by_id))
        .route("/master/:id",                         put(handlers::master::update))
        .route("/master/:id/submit-for-review",       post(handlers::master::submit_for_review))
        .route("/master/:id/approve",                 post(handlers::master::approve))
        .route("/master/:id/publish",                 post(handlers::master::publish))
        .route("/master/:id/archive",                 post(handlers::master::archive))
        .route("/master/:id/diff",                    get(handlers::master::diff))

        // --- 承認フロー ---
        .route("/approval/requests",                  post(handlers::approval::create_request))
        .route("/approval/requests",                  get(handlers::approval::list_requests))
        .route("/approval/requests/:id",              get(handlers::approval::get_request))
        .route("/approval/requests/:id/approve",      post(handlers::approval::approve))
        .route("/approval/requests/:id/reject",       post(handlers::approval::reject))

        // --- ユーザー・ロール管理 ---
        .route("/users",                              get(handlers::user::list))
        .route("/users",                              post(handlers::user::create))
        .route("/users/:id",                          put(handlers::user::update))
        .route("/users/:id",                          delete(handlers::user::deactivate))

        // --- 監査ログ ---
        .route("/audit",                              get(handlers::audit::list))
        .route("/audit/:id",                          get(handlers::audit::get_by_id))
        .route("/audit/export/xes",                   post(handlers::audit::export_xes))

        // --- 帳票・レポート ---
        .route("/reports/work-summary",               post(handlers::report::work_summary))
        .route("/reports/audit-xes",                  post(handlers::report::audit_xes))
        .route("/reports/kpi-dashboard",              get(handlers::report::kpi_dashboard))

        // --- 運用・管理（ops）---
        .route("/ops/outbox/dlq",                     get(handlers::ops::list_dlq))
        .route("/ops/outbox/:id/requeue",             post(handlers::ops::requeue))
        .route("/ops/hash-chain/verify",              post(handlers::ops::verify_hash_chain))
        .route("/ops/master-sync",                    post(handlers::ops::trigger_master_sync))

        // --- トレーサビリティ ---
        .route("/trace/:lot_id",                      get(handlers::trace::get_by_lot))
        .route("/trace/:lot_id/timeline",             get(handlers::trace::get_timeline))

        // --- アラート ---
        .route("/alerts",                             get(handlers::alerts::list))
        .route("/alerts/:id",                         get(handlers::alerts::get_by_id))
        .route("/alerts/:id/acknowledge",             post(handlers::alerts::acknowledge))
        .route("/alerts/:id/resolve",                 post(handlers::alerts::resolve))

        // --- CAPA（是正処置・予防処置）---
        .route("/capas",                              post(handlers::capa::create))
        .route("/capas",                              get(handlers::capa::list))
        .route("/capas/:id",                          get(handlers::capa::get_by_id))
        .route("/capas/:id/close",                    post(handlers::capa::close))

        // --- 改善提案（Kaizen Proposals）---
        .route("/kaizen-proposals",                   post(handlers::kaizen::create))
        .route("/kaizen-proposals",                   get(handlers::kaizen::list))
        .route("/kaizen-proposals/:id",               get(handlers::kaizen::get_by_id))
        .route("/kaizen-proposals/:id/approve",       post(handlers::kaizen::approve))
        .route("/kaizen-proposals/:id/implement",     post(handlers::kaizen::implement))

        // --- 入庫検査（IQC Judge / Concession Context）---
        .route("/iqc/judge",                          post(handlers::iqc::create_judgment))
        .route("/iqc/judge/:id",                      get(handlers::iqc::get_judgment))
        .route("/iqc/judge/:id/approve",              post(handlers::iqc::approve_judgment))
        .route("/iqc/concession",                     post(handlers::iqc::create_concession))
        .route("/iqc/concession/:id",                 get(handlers::iqc::get_concession))
        .route("/iqc/concession/:id/approve",         post(handlers::iqc::approve_concession))

        // --- 手直し指示（Rework Disposition Context）---
        .route("/rework/disposition",                 post(handlers::rework::create_disposition))
        .route("/rework/disposition",                 get(handlers::rework::list_dispositions))
        .route("/rework/disposition/:id",             get(handlers::rework::get_disposition))
        .route("/rework/disposition/:id/approve",     post(handlers::rework::approve_disposition))

        // --- 廃棄（Scrap）---
        .route("/scrap",                              post(handlers::scrap::create))
        .route("/scrap",                              get(handlers::scrap::list))
        .route("/scrap/:id",                          get(handlers::scrap::get_by_id))
        .route("/scrap/:id/approve",                  post(handlers::scrap::approve))

        // --- 返品（Return）---
        .route("/return",                             post(handlers::ret::create))
        .route("/return",                             get(handlers::ret::list))
        .route("/return/:id",                         get(handlers::ret::get_by_id))
        .route("/return/:id/approve",                 post(handlers::ret::approve))

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
// crates/wnav_master_api/src/error.rs

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
// crates/wnav_master_api/src/cors.rs

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
            HeaderName::from_static("x-trace-id"),
        ])
        .max_age(std::time::Duration::from_secs(3600))
}
```

### 5-1. TLS 終端に関する注記

TLS 終端は IIS（Windows Server 2022）がリバースプロキシとして担当する。本クレートは HTTP（ポート 8081）でリッスンし、IIS から転送されるリクエストを受け付ける。

---

## 6. エントリポイント（main.rs）

wnav_master_api の main.rs は **write_pool** と **read_pool** のみを生成・注入する。event_insert_pool は生成しない。

```rust
// crates/wnav_master_api/src/main.rs

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 設定ロード（環境変数）
    let config: AppConfig = envy::from_env()?;

    // tracing 初期化
    init_tracing(&config.tracing_level);

    // マスタ書き込みプール（write_pool）生成
    let write_pool = wnav_db::connect_write(
        &config.write_db_url,
    )
    .await?;

    // 読み取り専用プール（read_pool）生成
    let read_pool = wnav_db::connect_read_only(
        &config.read_db_url,
    )
    .await?;

    // AppState 構築（event_insert_pool は持たない）
    let auth_state = wnav_auth::init_auth_state(&config.jwt_public_key_pem)?;
    let state = AppState {
        write_pool: write_pool.clone(),
        read_pool,
        auth_state,
        config: Arc::new(config.clone()),
    };

    // ルータ構築
    let app = create_router(state.clone());
    let app = apply_middleware(app, &config);
    let app = app.layer(cors_layer(&config.cors_allow_origins));

    // HashChainVerifier 週次検証スケジューラ起動（BAT-001）
    let hash_verify_handle = tokio::spawn(
        wnav_hash_chain::run_weekly_verifier(write_pool.clone())
    );

    // 集計系スケジュールジョブ起動（BAT-004 以降）
    // KPI 日次集計・SLA 監視・証拠ファイルアーカイブ等
    let aggregation_handle = tokio::spawn(
        schedule_aggregation_jobs(write_pool.clone())
    );

    // HTTP サーバー起動
    let listener = tokio::net::TcpListener::bind(
        format!("0.0.0.0:{}", config.port)
    ).await?;
    tracing::info!(port = config.port, "wnav_master_api started");

    tokio::select! {
        result = axum::serve(listener, app) => result?,
        _ = hash_verify_handle => tracing::error!("hash_chain_verifier が予期せず終了しました"),
        _ = aggregation_handle => tracing::error!("aggregation_jobs が予期せず終了しました"),
    }

    Ok(())
}

/// BAT-004 以降の集計系スケジュールジョブをまとめて起動する。
async fn schedule_aggregation_jobs(write_pool: PgPool) {
    tokio::join!(
        wnav_jobs::run_kpi_daily_aggregation(write_pool.clone()),   // BAT-004
        wnav_jobs::run_sla_monitor(write_pool.clone()),              // BAT-005
        wnav_jobs::run_evidence_archive(write_pool.clone()),         // BAT-006
    );
}
```

---

**本節で確定した方針**
- **AppState は write_pool + read_pool の 2 プールのみを保持し、event_insert_pool をコンパイル時に排除することでイベント挿入経路の混入を防止する。**
- **ミドルウェアチェーンは Tracing → Auth → RateLimit → Handler の順で適用し、Idempotency は master-api には適用しない。**
- **auth_middleware は aud = "master-api" クレームを検証する。**
- **main.rs は write_pool + read_pool のみを生成・注入し、HashChainVerifier（BAT-001）と集計系スケジュールジョブ（BAT-004 以降）を tokio::spawn で起動する。**
- **TLS 終端は IIS に委譲し、本クレートは HTTP（ポート 8081）リッスンのみとすることを確定した。**

---

## 参照業界分析

### 必須
- [`90_業界分析/09_セキュリティとアクセス制御.md`](../../90_業界分析/09_セキュリティとアクセス制御.md)

### 関連
- [`90_業界分析/06_品質管理とトレーサビリティ.md`](../../90_業界分析/06_品質管理とトレーサビリティ.md)
