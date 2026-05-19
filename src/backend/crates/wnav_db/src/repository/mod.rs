// リポジトリ実装モジュール集約
// wnav_domain の Repository Trait に対応する PostgreSQL 実装を提供する。
// 各モジュールは sqlx::query_as を用いた型安全な DB アクセスを実装する。

pub mod andon_repo;
pub mod auth_log_repo;
pub mod capa_repo;
pub mod case_lock_repo;
pub mod disposition_repo;
pub mod electronic_signature_repo;
pub mod evidence_repo;
pub mod hash_chain_block_repo;
pub mod idempotency_repo;
pub mod incoming_inspection_repo;
pub mod kaizen_repo;
pub mod master_version_repo;
pub mod measurement_repo;
pub mod outbox_repo;
pub mod rework_repo;
pub mod sop_repo;
pub mod step_repo;
pub mod user_repo;
pub mod work_assignment_repo;
pub mod work_event_repo;
pub mod work_execution_repo;

// 主要なリポジトリ型を再エクスポートして利用しやすくする
pub use andon_repo::PgAndonRepository;
pub use auth_log_repo::{AuthLogEventType, InsertAuthLogCmd, PgAuthLogRepository};
pub use capa_repo::PgCapaRepository;
pub use case_lock_repo::PgCaseLockRepository;
pub use disposition_repo::PgDispositionRepository;
pub use electronic_signature_repo::PgElectronicSignatureRepository;
pub use evidence_repo::PgEvidenceRepository;
pub use hash_chain_block_repo::PgHashChainBlockRepository;
pub use idempotency_repo::PgIdempotencyRepository;
pub use incoming_inspection_repo::PgIncomingInspectionRepository;
pub use kaizen_repo::PgKaizenRepository;
pub use master_version_repo::PgMasterVersionRepository;
pub use measurement_repo::PgMeasurementRepository;
pub use outbox_repo::PgOutboxRepository;
pub use rework_repo::PgReworkRepository;
pub use sop_repo::PgSopRepository;
pub use step_repo::PgStepRepository;
pub use user_repo::PgUserRepository;
pub use work_assignment_repo::PgWorkAssignmentRepository;
pub use work_event_repo::PgWorkEventRepository;
pub use work_execution_repo::PgWorkExecutionRepository;
