// wnav_master_api — マスタメンテ・管理コンソール向け axum API バイナリ
//
// 起動フロー:
// 1. 設定ロード（WNAV_PROFILE 環境変数 / YAML + secret_ref 解決 / fail-fast exit 78）
// 2. tracing 初期化（JSON 構造化ログ）
// 3. DB 接続（write_pool + read_pool のみ / event_insert_pool は持たない）
// 4. JWT キーストア初期化（aud = "master-api" / RS256 秘密鍵 + 公開鍵）
// 5. BAT 起動（tokio::spawn）:
//    - BAT-001 Hash Chain Verify（週次 cron）
//    - BAT-004 PII Anonymizer（月次）
//    - BAT-005 PG Backup 通知（日次 02:00）
//    - BAT-006〜010 Reports 生成（イベント駆動）
//    - BAT-011 リワーク・コスト集計（日次）
// 6. axum Router 起動（port 8081）

// unsafe コードを禁止する（src/CLAUDE.md および src/backend/CLAUDE.md の必須要件）
#![forbid(unsafe_code)]
// Clippy の全 lint を有効化する（ワークスペース設定で deny 済みだが明示する）
#![deny(clippy::all, clippy::pedantic)]
// 例外: doc コメントのリンク省略は許容
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
// 例外: モジュール名重複は許容（例: api::auth::AuthError）
#![allow(clippy::module_name_repetitions)]
// 例外: must_use 警告は許容
#![allow(clippy::must_use_candidate)]
// 例外: wildcard_imports は utoipa の path マクロ内で発生するため許容
#![allow(clippy::wildcard_imports)]

mod api;
mod batch;
mod dto;
mod error;
mod middleware;
mod router;
mod state;

use std::sync::Arc;

use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};
use wnav_auth::JwtKeyStore;
use wnav_db::DbConfig;

use crate::state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // ── 1. 設定ロード（fail-fast: WNAV_PROFILE 未設定で exit 78）──────────
    let config = wnav_config::load_master_api().unwrap_or_else(|e| {
        eprintln!("FATAL: configuration load failed: {e}");
        // POSIX exit code 78 (EX_CONFIG) を使用して設定エラーを示す
        std::process::exit(78);
    });
    let config = Arc::new(config);

    // ── 2. tracing 初期化（JSON 構造化ログ）─────────────────────────────
    init_tracing(&config.shared.observability.log_level);

    tracing::info!(
        service = "wnav_master_api",
        version = env!("CARGO_PKG_VERSION"),
        "サービスを起動しています",
    );

    // ── 3. DB 接続（write_pool + read_pool のみ）─────────────────────────
    // event_insert_pool は保有しない（コンパイル時に型で保証済み）
    let db_cfg = DbConfig {
        max_connections: config.database.max_connections,
        min_connections: config.database.min_connections,
        acquire_timeout_secs: config.database.acquire_timeout_sec,
        idle_timeout_secs: config.database.idle_timeout_sec,
        max_lifetime_secs: config.database.max_lifetime_sec,
    };

    let (write_pool, read_pool) = wnav_db::init_master_pools(
        &config.database.host,
        config.database.port,
        &config.database.name,
        &config.database.write.user,
        config.database.write.password.expose(),
        &config.database.read.user,
        config.database.read.password.expose(),
        &db_cfg,
    )
    .await
    .unwrap_or_else(|e| {
        tracing::error!(error = %e, "DB 接続プール初期化に失敗しました");
        std::process::exit(1);
    });

    tracing::info!("DB 接続プール初期化完了（write_pool + read_pool）");

    // ── 4. JWT キーストア初期化（aud = "master-api"）──────────────────────
    // master-api は JWT 発行も行うため with_signing_key を使用する
    // kid は hash_chain_verify.cron フィールドを流用せず、固定文字列で初期化する
    // 本番では kid も設定ファイルに切り出す
    let key_store = Arc::new(JwtKeyStore::with_signing_key(
        config.jwt_private.private_key.expose(),
        config.shared.jwt_public.public_key.expose(),
        "2026-Q2",       // 鍵ローテーション識別子（90 日ごとに更新）
        "master-api",    // aud クレームの値（terminal-api トークンを拒否する）
    ));

    tracing::info!("JWT キーストア初期化完了（aud = 'master-api'）");

    // ── 5. AppState 構築（event_insert_pool は含まない）──────────────────
    let state = AppState {
        write_pool: write_pool.clone(),
        read_pool: read_pool.clone(),
        key_store: key_store.clone(),
        config: config.clone(),
    };

    // ── 5a. BAT-001: ハッシュチェーン検証（週次）─────────────────────────
    let bat001_write_pool = write_pool.clone();
    let bat001_read_pool = read_pool.clone();
    let bat001_cron = config.hash_chain_verify.cron.clone();
    tokio::spawn(async move {
        batch::hash_chain_verify::run(bat001_write_pool, bat001_read_pool, bat001_cron).await;
    });

    // ── 5b. BAT-004: PII 匿名化（月次）──────────────────────────────────
    let bat004_write_pool = write_pool.clone();
    tokio::spawn(async move {
        batch::pii_anonymizer::run(bat004_write_pool).await;
    });

    // ── 5c. BAT-005: PG バックアップ通知（日次）──────────────────────────
    let bat005_write_pool = write_pool.clone();
    // backup_notification_url は設定から取得する（空文字の場合はスキップ）
    // MasterApiConfig には external セクションがないため固定空文字列を使用する
    let backup_url = String::new();
    tokio::spawn(async move {
        batch::pg_backup::run(bat005_write_pool, backup_url).await;
    });

    // ── 5d. BAT-006〜010: レポート生成（イベント駆動）────────────────────
    let bat006_write_pool = write_pool.clone();
    let bat006_read_pool = read_pool.clone();
    tokio::spawn(async move {
        batch::reports::run(bat006_write_pool, bat006_read_pool).await;
    });

    // ── 5e. BAT-011: リワーク・コスト集計（日次）─────────────────────────
    let bat011_write_pool = write_pool.clone();
    let bat011_read_pool = read_pool.clone();
    tokio::spawn(async move {
        batch::rework_cost::run(bat011_write_pool, bat011_read_pool).await;
    });

    // ── 6. axum Router 起動（ポート 8081）───────────────────────────────
    let app = router::create_router(state.clone());

    // ミドルウェアチェーン: TracingMiddleware → AuthMiddleware → Handler
    // IdempotencyMiddleware は master-api には適用しない
    use axum::middleware::from_fn_with_state;
    use tower::ServiceBuilder;
    use tower_http::trace::TraceLayer;

    let cors = build_cors_layer(&config.shared.cors.allow_origins);

    let app = app.layer(
        ServiceBuilder::new()
            // トレーシング（X-Request-Id 付与・構造化ログ）
            .layer(TraceLayer::new_for_http())
            .layer(axum::middleware::from_fn(
                middleware::tracing::request_id_middleware,
            ))
            // JWT 認証（aud = "master-api" 検証）
            .layer(from_fn_with_state(
                key_store,
                middleware::auth::auth_middleware,
            ))
            // CORS
            .layer(cors),
    );

    let bind_addr = format!(
        "{}:{}",
        config.server.master_api.bind_addr, config.server.master_api.port
    );

    let listener = tokio::net::TcpListener::bind(&bind_addr)
        .await
        .unwrap_or_else(|e| {
            tracing::error!(addr = %bind_addr, error = %e, "TCP リスナーのバインドに失敗しました");
            std::process::exit(1);
        });

    tracing::info!(
        service = "wnav_master_api",
        addr = %bind_addr,
        "wnav_master_api を起動しました（管理 LAN 専用・ポート 8081）",
    );

    // HTTP サーバーを起動する（axum::serve はシグナルハンドリングなしで無限待機）
    axum::serve(listener, app)
        .await
        .unwrap_or_else(|e| {
            tracing::error!(error = %e, "axum サーバーが異常終了しました");
            std::process::exit(1);
        });

    Ok(())
}

/// JSON 構造化ログ tracing サブスクライバを初期化する。
fn init_tracing(log_level: &str) {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        EnvFilter::new(format!("{log_level},tower_http=info,axum=info"))
    });

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt::layer().json())
        .init();
}

/// CORS レイヤーを構築する。
///
/// 設定の `cors.allow_origins` リストから `CorsLayer` を生成する。
fn build_cors_layer(allow_origins: &[String]) -> tower_http::cors::CorsLayer {
    use axum::http::{HeaderName, Method};
    use tower_http::cors::AllowOrigin;

    // オリジンを HeaderValue に変換する
    let origins: Vec<_> = allow_origins
        .iter()
        .filter_map(|o| o.parse().ok())
        .collect();

    tower_http::cors::CorsLayer::new()
        .allow_origin(if origins.is_empty() {
            AllowOrigin::any()
        } else {
            AllowOrigin::list(origins)
        })
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([
            HeaderName::from_static("authorization"),
            HeaderName::from_static("content-type"),
            HeaderName::from_static("x-request-id"),
            HeaderName::from_static("x-signature-256"),
        ])
        .max_age(std::time::Duration::from_secs(3600))
}
