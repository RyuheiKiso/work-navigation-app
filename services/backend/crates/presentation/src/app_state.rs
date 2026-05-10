//! アプリケーション State（DI）
//!
//! 対応 §: ロードマップ §9.1（DI コンテナ）

use std::sync::Arc;
use wna_adapter::{
    Hs256SessionFactory, PostgresAuditRepository, PostgresCredentialRepository,
    PostgresMasterRepository, PostgresRepository,
};
use wna_usecase::{LoginUseCase, StartTaskUseCase};
use wna_domain::PasswordHasher;

/// 本番 / dev で共有する DI 状態
#[derive(Clone)]
pub struct AppState<H: PasswordHasher + Send + Sync + 'static> {
    pub task_repo: PostgresRepository,
    pub credential_repo: PostgresCredentialRepository,
    pub audit_repo: PostgresAuditRepository,
    pub master_repo: PostgresMasterRepository,
    pub session_factory: Arc<Hs256SessionFactory>,
    pub start_task_uc: Arc<
        StartTaskUseCase<PostgresRepository>,
    >,
    pub login_uc: Arc<
        LoginUseCase<PostgresCredentialRepository, Hs256SessionFactory, H>,
    >,
}
