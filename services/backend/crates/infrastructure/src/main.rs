//! work-navigation-app バックエンドエントリポイント
//!
//! 対応 §: ロードマップ §7.3 §14.2 §16

use std::sync::Arc;
use anyhow::Result;
use sqlx::postgres::PgPoolOptions;
use std::env;
use tracing::info;
use tracing_subscriber::EnvFilter;

use wna_adapter::{
    Argon2idHasher, Hs256SessionFactory, PostgresAuditRepository,
    PostgresCredentialRepository, PostgresMasterRepository, PostgresRepository,
};
use wna_infrastructure::AppConfig;
use wna_presentation::{build_router, AppState};
use wna_usecase::{LoginUseCase, StartTaskUseCase};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_target(false)
        .init();

    let cfg = AppConfig::from_env()?;
    info!(listen_addr = %cfg.listen_addr, "work-navigation-app バックエンドを起動します");

    let pool = PgPoolOptions::new()
        .min_connections(2)
        .max_connections(10)
        .connect(&cfg.database_url)
        .await?;

    sqlx::migrate!("../../migrations").run(&pool).await?;

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
