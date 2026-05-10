//! work-navigation-app ドメイン層
//!
//! 対応 §: ロードマップ §3 §9.1 §9.4 §10.6.1 §28
//!
//! 本 crate は最内層（クリーンアーキテクチャの中心）であり、
//! 標準ライブラリのみに依存する純粋関数中心の構成（§9.4）。
//!
//! ユビキタス言語（§28）と完全一致する命名で
//! Aggregate／Entity／Value Object／Repository trait を提供する。

// 公開する子モジュールの宣言（責務単位で分割し 1 ファイル ≤ 500 行を維持）
// 値オブジェクト（TaskId／DeviceId／LamportTimestamp／CompletionCriteria 等）
pub mod value_object;
// 「作業（Task）」Aggregate（§3.1.1 11 構成要素）
pub mod task;
// Repository trait（依存逆転の I/F 定義）
pub mod repository;
// ドメインエラー型
pub mod error;
// 認証（§10.5 §27 F-006 §29 R-006）
pub mod auth;
// 順序情報（§10.3.2 §27 F-005）
pub mod production_order;
// プロセス（製造工程、§3.4 §10.2.1）
pub mod process;
// 同期ドメイン（§10.6 §27 F-002）
pub mod sync;

// 上位から import しやすいよう代表型を再エクスポートする
pub use error::DomainError;
pub use repository::TaskRepository;
pub use task::{Task, TaskState};
pub use value_object::{CompletionCriteria, DeviceId, Evidence, LamportTimestamp, TaskId};
pub use auth::{
    Credential, CredentialError, PasswordHash, PasswordHasher, Session, SessionToken,
    User, UserId,
};
pub use production_order::{
    IdempotencyKey, ItemCode, OrderId, OrderState, ProductionOrder, ProductionOrderError, Quantity,
};
pub use process::{
    Process, ProcessEdge, ProcessError, ProcessId, Step, StepId,
};
pub use sync::{lww_strictly_after, SyncEvent, SyncEventKind};
