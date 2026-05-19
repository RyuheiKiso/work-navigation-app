// wnav_terminal_api エントリポイント（MOD-BE-001 §6）
//
// 起動順序:
// 1. 設定ロード（WNAV_PROFILE 未設定なら exit 78 で fail-fast）
// 2. tracing 初期化（JSON 構造化ログ）
// 3. DB 接続（event_insert_pool + read_pool のみ。write_pool は生成しない）
// 4. バッチタスク起動（BAT-002 Outbox / BAT-013 CaseLock Reaper / BAT-014 SSE Retry）
// 5. axum Router 組み立て + ミドルウェア適用 + HTTP リスナー起動
//
// DB ロール物理保証:
// - event_insert_pool: app_event_insert ロール（INSERT 専用）
// - read_pool: app_read ロール（SELECT 専用）
// - write_pool は存在しない（コンパイル時に型で保証）

// unsafe コードを完全に禁止する（src/backend/CLAUDE.md 必須要件）
#![forbid(unsafe_code)]
// Clippy の全 lint をエラーとして扱う（コード品質の維持）
#![deny(clippy::all, clippy::pedantic)]
// 例外: doc コメントのリンク省略は許容
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
// 例外: モジュール名重複は許容
#![allow(clippy::module_name_repetitions)]
// 例外: must_use 警告は許容
#![allow(clippy::must_use_candidate)]
// 例外: DTO フィールド・ミドルウェアヘルパー関数は API 実装として宣言されている
// 全量接続後に削除すること
#![allow(dead_code)]

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
use wnav_db::pool::{DbConfig, connect};

use crate::{
    middleware::apply_middleware,
    router::create_router,
    state::AppState,
};

/// wnav_terminal_api バイナリのエントリポイント。
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // ─────────────────────────────────────────────────────────────────────────
    // 1. 設定ロード（fail-fast: WNAV_PROFILE 未設定なら exit 78）
    // ─────────────────────────────────────────────────────────────────────────
    let config = wnav_config::load_terminal_api().unwrap_or_else(|e| {
        eprintln!("FATAL: 設定ロードに失敗した: {e}");
        std::process::exit(78);
    });

    // ─────────────────────────────────────────────────────────────────────────
    // 2. tracing 初期化（JSON 構造化ログ）
    // ─────────────────────────────────────────────────────────────────────────
    init_tracing(&config.shared.observability.log_level);

    tracing::info!(
        log_id = "LOG-START-001",
        event = "wnav_terminal_api.starting",
        port = config.server.terminal_api.port,
        "wnav_terminal_api を起動中"
    );

    // ─────────────────────────────────────────────────────────────────────────
    // 3. DB 接続（event_insert_pool + read_pool のみ。write_pool は生成しない）
    // ─────────────────────────────────────────────────────────────────────────
    let db_cfg = DbConfig {
        max_connections: config.database.max_connections,
        min_connections: config.database.min_connections,
        acquire_timeout_secs: config.database.acquire_timeout_sec,
        idle_timeout_secs: config.database.idle_timeout_sec,
        max_lifetime_secs: config.database.max_lifetime_sec,
    };

    // app_event_insert プール: INSERT 専用（Append-only テーブルへの書き込み）
    let event_insert_url = format!(
        "postgres://{}:{}@{}:{}/{}",
        config.database.event_insert.user,
        config.database.event_insert.password.expose(),
        config.database.host,
        config.database.port,
        config.database.name,
    );
    let event_insert_pool = connect(&event_insert_url, &db_cfg)
        .await
        .unwrap_or_else(|e| {
            tracing::error!(error = %e, "event_insert_pool の接続に失敗した");
            std::process::exit(1);
        });

    // app_read プール: SELECT 専用（全テーブルの読み取り）
    let read_url = format!(
        "postgres://{}:{}@{}:{}/{}",
        config.database.read.user,
        config.database.read.password.expose(),
        config.database.host,
        config.database.port,
        config.database.name,
    );
    let read_pool = connect(&read_url, &db_cfg)
        .await
        .unwrap_or_else(|e| {
            tracing::error!(error = %e, "read_pool の接続に失敗した");
            std::process::exit(1);
        });

    tracing::info!(
        log_id = "LOG-START-002",
        event = "db_pools.initialized",
        "DB プールを初期化した（event_insert + read）"
    );

    // ─────────────────────────────────────────────────────────────────────────
    // JWT キーストア初期化（検証専用。terminal-api は JWT を発行しない）
    // ─────────────────────────────────────────────────────────────────────────
    let jwt_key_store = Arc::new(JwtKeyStore::new(
        config.shared.jwt_public.public_key.expose(),
        // kid は将来的に YAML から動的に取得する。現時点では固定値を使用する
        "default",
        "terminal-api",
    ));

    // ─────────────────────────────────────────────────────────────────────────
    // AppState 構築（write_pool は含まない）
    // ─────────────────────────────────────────────────────────────────────────
    let config_arc = Arc::new(config.clone());
    let state = AppState {
        event_insert_pool: event_insert_pool.clone(),
        read_pool,
        jwt_key_store,
        config: config_arc,
    };

    // ─────────────────────────────────────────────────────────────────────────
    // 4. バッチタスク起動
    // ─────────────────────────────────────────────────────────────────────────

    // シャットダウンシグナルチャネルを生成する
    let (shutdown_tx, _) = tokio::sync::broadcast::channel::<()>(1);

    // BAT-002: Outbox Consumer（wnav_outbox クレートに委譲する）
    let outbox_config = wnav_outbox::OutboxConfig {
        poll_interval_ms: config.outbox.interval_ms,
        batch_size: 10,
        max_retry_attempts: config.outbox.retry_max,
        initial_backoff_ms: 1_000,
        max_backoff_ms: config.outbox.backoff_max_sec.saturating_mul(1_000),
        parent_api_url: String::new(),
        hmac_secret: config.webhook.hmac_key.expose().to_string(),
        http_timeout_secs: 30,
    };
    let outbox_consumer = wnav_outbox::OutboxConsumer::from_config(
        event_insert_pool.clone(),
        outbox_config,
    )
    .unwrap_or_else(|e| {
        tracing::error!(error = %e, "Outbox Consumer の初期化に失敗した");
        std::process::exit(1);
    });
    let outbox_arc = Arc::new(outbox_consumer);
    let outbox_shutdown = shutdown_tx.subscribe();
    tokio::spawn(wnav_outbox::run_consumer(outbox_arc, outbox_shutdown));

    // BAT-013: CaseLock Reaper（60 秒ごとにハートビートタイムアウトを処理する）
    let reaper_pool = event_insert_pool.clone();
    let reaper_shutdown = shutdown_tx.subscribe();
    tokio::spawn(batch::case_lock_reaper::run_case_lock_reaper(
        reaper_pool,
        reaper_shutdown,
    ));

    // BAT-014: SSE Retry（1 分ごとに failed sse_dispatch_log を再送試行する）
    let sse_retry_pool = event_insert_pool.clone();
    let sse_retry_shutdown = shutdown_tx.subscribe();
    tokio::spawn(batch::sse_retry::run_sse_retry(sse_retry_pool, sse_retry_shutdown));

    tracing::info!(
        log_id = "LOG-START-003",
        event = "batch_tasks.started",
        "バッチタスクを起動した（BAT-002 / BAT-013 / BAT-014）"
    );

    // ─────────────────────────────────────────────────────────────────────────
    // 5. axum Router 組み立て + ミドルウェア適用 + HTTP リスナー起動
    // ─────────────────────────────────────────────────────────────────────────

    // Router<AppState> を生成し、ミドルウェアを適用してから State を解決する
    let app = create_router();
    let app = apply_middleware(app, &config);
    // State を解決して Router<()> に変換する（axum::serve に渡すために必要）
    let app = app.with_state(state);

    let bind_addr = format!(
        "{}:{}",
        config.server.terminal_api.bind_addr, config.server.terminal_api.port
    );

    let listener = tokio::net::TcpListener::bind(&bind_addr)
        .await
        .unwrap_or_else(|e| {
            tracing::error!(error = %e, bind_addr = %bind_addr, "TCP リスナーのバインドに失敗した");
            std::process::exit(1);
        });

    tracing::info!(
        log_id = "LOG-START-004",
        event = "wnav_terminal_api.started",
        bind_addr = %bind_addr,
        "wnav_terminal_api が起動した"
    );

    // Ctrl+C シグナルハンドラを設定する
    let shutdown_signal = async move {
        tokio::signal::ctrl_c()
            .await
            .expect("Ctrl+C シグナルハンドラの設定に失敗した");
        tracing::info!(
            log_id = "LOG-SHUTDOWN-001",
            "シャットダウンシグナルを受信した"
        );
        // バッチタスクにシャットダウンシグナルを送信する
        let _ = shutdown_tx.send(());
    };

    // axum サーバーを起動する（TLS 終端は IIS に委譲するため HTTP のみ）
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal)
        .await?;

    Ok(())
}

/// tracing サブスクライバを JSON 構造化ログ形式で初期化する。
fn init_tracing(log_level: &str) {
    // RUST_LOG 環境変数が設定されている場合はそれを優先する
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(log_level));

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt::layer().json())
        .init();
}
