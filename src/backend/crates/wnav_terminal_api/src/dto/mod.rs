// wnav_terminal_api DTO（Data Transfer Objects）モジュール
//
// 全エンドポイントのリクエスト・レスポンス型を定義する。
// ウトピア OpenAPI スキーマ自動生成のため #[derive(ToSchema)] を付与する。

pub mod andon;
pub mod auth;
pub mod electronic_signatures;
pub mod evidences;
pub mod iqc;
pub mod kaizen;
pub mod response_envelope;
pub mod reworks;
pub mod step_events;
pub mod sync;
pub mod system;
pub mod work_assignments;
pub mod work_executions;
pub mod work_orders;

// よく使う型を再エクスポートする（一部は API ハンドラから直接参照するため pub use する）
#[allow(unused_imports)]
pub use response_envelope::{ApiResponse, PaginatedMeta, PaginatedResponse};
