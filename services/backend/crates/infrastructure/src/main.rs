//! work-navigation-app バックエンドエントリポイント
//!
//! 対応 §: ロードマップ §7.3 §14.2 §16
//!
//! 単一バイナリ `wna-backend` で REST API サーバ起動と
//! デモシード投入の双方を担う。サブコマンドは `clap` で分岐する。
//! 引数なしは `serve` と等価で、既存の起動方法（compose／systemd）を壊さない。

use std::sync::Arc;
use anyhow::Result;
use clap::{Parser, Subcommand};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::env;
use tracing::info;
use tracing_subscriber::EnvFilter;

use wna_adapter::{
    Argon2idHasher, Hs256SessionFactory, PostgresAuditRepository,
    PostgresCredentialRepository, PostgresMasterRepository, PostgresRepository,
};
use wna_infrastructure::{run_seed, AppConfig, SeedPreset};
use wna_presentation::{build_router, AppState};
use wna_usecase::{LoginUseCase, StartTaskUseCase};

/// `wna-backend` CLI 定義
#[derive(Parser)]
#[command(name = "wna-backend", about = "work-navigation-app backend")]
struct Cli {
    /// サブコマンド（未指定なら serve）
    #[command(subcommand)]
    command: Option<Command>,
}

/// サブコマンド一覧
#[derive(Subcommand)]
enum Command {
    /// REST API サーバを起動する（既定）
    Serve,
    /// デモデータを DB に投入する（§14.2 顧客／社内デモ用）
    Seed {
        /// 投入プリセット（minimal: ユーザのみ／showcase: マスタ＋タスクも）
        #[arg(long, value_enum, default_value_t = SeedPreset::Showcase)]
        preset: SeedPreset,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    // 構造化ログを最初に初期化する（後段の info! を捕捉するため）
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_target(false)
        .init();

    // CLI 解析
    let cli = Cli::parse();

    // 共通: 構成読込と PgPool 構築（serve／seed 双方に必要）
    let cfg = AppConfig::from_env()?;
    let pool = build_pool(&cfg.database_url).await?;
    sqlx::migrate!("../../migrations").run(&pool).await?;

    // サブコマンド分岐（未指定は Serve）
    match cli.command.unwrap_or(Command::Serve) {
        Command::Serve => serve(cfg, pool).await,
        Command::Seed { preset } => run_seed(preset, &pool).await,
    }
}

/// `PgPool` を構築する（serve／seed 共通）
async fn build_pool(database_url: &str) -> Result<PgPool> {
    let pool = PgPoolOptions::new()
        .min_connections(2)
        .max_connections(10)
        .connect(database_url)
        .await?;
    Ok(pool)
}

/// REST API サーバを起動する
async fn serve(cfg: AppConfig, pool: PgPool) -> Result<()> {
    info!(listen_addr = %cfg.listen_addr, "work-navigation-app バックエンドを起動します");

    let task_repo = PostgresRepository::new(pool.clone());
    let credential_repo = PostgresCredentialRepository::new(pool.clone());
    let audit_repo = PostgresAuditRepository::new(pool.clone());
    let master_repo = PostgresMasterRepository::new(pool.clone());

    let session_secret = env::var("WNA_SESSION_SECRET").map_err(|_| {
        anyhow::anyhow!("WNA_SESSION_SECRET が未設定です")
    })?;
    let session_factory = Arc::new(Hs256SessionFactory::new(session_secret.into_bytes()));
    let hasher = Argon2idHasher::new();

    let start_task_uc = Arc::new(StartTaskUseCase::new(task_repo.clone()));
    let login_uc = Arc::new(LoginUseCase::new(
        credential_repo.clone(),
        (*session_factory).clone(),
        hasher,
    ));

    let state = AppState::<Argon2idHasher> {
        task_repo,
        credential_repo,
        audit_repo,
        master_repo,
        session_factory,
        start_task_uc,
        login_uc,
    };

    let app = build_router(state);

    let listener = tokio::net::TcpListener::bind(cfg.listen_addr).await?;
    info!("REST API リスナを開始します: {}", cfg.listen_addr);
    axum::serve(listener, app).await?;

    Ok(())
}
