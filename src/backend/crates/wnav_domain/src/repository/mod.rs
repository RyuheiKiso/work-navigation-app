// リポジトリ Trait モジュール
// Domain 層が依存するリポジトリ Trait の集約モジュール。
// 実装は `crates/wnav_db/` が担う（依存性逆転の原則）。

pub mod andon_repo;
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

// 主要な Trait を再エクスポートして使いやすくする
pub use andon_repo::AndonRepository;
pub use capa_repo::CapaRepository;
pub use case_lock_repo::CaseLockRepository;
pub use disposition_repo::DispositionRepository;
pub use electronic_signature_repo::ElectronicSignatureRepository;
pub use evidence_repo::EvidenceRepository;
pub use hash_chain_block_repo::{HashChainBlock, HashChainBlockRepository};
pub use idempotency_repo::{IdempotencyRecord, IdempotencyRepository};
pub use incoming_inspection_repo::IncomingInspectionRepository;
pub use kaizen_repo::KaizenRepository;
pub use master_version_repo::{CreateMasterVersionCmd, MasterVersionRepository};
pub use measurement_repo::MeasurementRepository;
pub use outbox_repo::OutboxRepository;
pub use rework_repo::ReworkRepository;
pub use sop_repo::{CreateSopCmd, SopRepository, UpdateSopCmd};
pub use step_repo::StepRepository;
pub use user_repo::{CreateUserCmd, UserRepository};
pub use work_assignment_repo::WorkAssignmentRepository;
pub use work_event_repo::WorkEventRepository;
pub use work_execution_repo::{
    CreateWorkExecutionCmd, WorkExecutionFilter, WorkExecutionRepository,
};
