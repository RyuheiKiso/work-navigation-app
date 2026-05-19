// ドメインモデルモジュール
// wnav_domain のコアドメインエンティティ・値オブジェクトを集約する。
// 外部クレート（axum・sqlx 等）への依存を持たない純粋なドメインモデル層。

pub mod andon;
pub mod capa;
pub mod case_lock;
pub mod disposition;
pub mod electronic_signature;
pub mod evidence;
pub mod incoming_inspection;
pub mod kaizen;
pub mod lot;
pub mod master_version;
pub mod measurement;
pub mod outbox;
pub mod pagination;
pub mod rework;
pub mod sop;
pub mod step;
pub mod user;
pub mod work_assignment;
pub mod work_event;
pub mod work_execution;
