//! work-navigation-app adapter 層
//!
//! 対応 §: ロードマップ §9.1 §10.3.1 §10.6.1 §11.4.1
//!
//! 永続化（PostgreSQL／sqlx）と DTO 変換を担う境界 crate。
//! 副作用はすべて本層に閉じ込め、内側の domain／usecase は純粋に保つ。

// 子モジュール
pub mod dto;
pub mod postgres_repository;
pub mod argon2_hasher;
pub mod memory_idempotency_store;
pub mod postgres_credential_repository;
pub mod postgres_order_repository;
pub mod postgres_lww_repository;
pub mod hs256_session_factory;
pub mod postgres_audit_repository;
pub mod postgres_master_repository;

// 代表型を再エクスポート
pub use dto::{TaskDto, AppendRecordRequestDto};
pub use postgres_repository::{PostgresRepository, PostgresRepositoryError};
pub use argon2_hasher::Argon2idHasher;
pub use memory_idempotency_store::MemoryOrderRepository;
pub use postgres_credential_repository::PostgresCredentialRepository;
pub use postgres_order_repository::PostgresOrderRepository;
pub use postgres_lww_repository::{LwwEntry, PostgresLwwRepository};
pub use hs256_session_factory::Hs256SessionFactory;
pub use postgres_audit_repository::{AuditEntry, AuditRow, PostgresAuditRepository};
pub use postgres_master_repository::{
    FlowSummary, MasterRow, PostgresMasterRepository, TaskListItem, TaskStepRow,
};
