//! work-navigation-app presentation 層
//!
//! 対応 §: ロードマップ §7.3 §10.3.1 §11.4.1

pub mod router;
pub mod handler_tasks;
pub mod handler_auth;
pub mod handler_records;
pub mod handler_audit;
pub mod handler_master;
pub mod handler_flows;
pub mod handler_dashboard;
pub mod middleware_auth;
pub mod middleware_request_id;
pub mod app_state;
pub mod api_error;

pub use router::build_router;
pub use handler_auth::{LoginRequest, LoginResponse};
pub use middleware_auth::{require_session, AuthContext, SESSION_MAX_AGE_SECONDS};
pub use middleware_request_id::{request_id, RequestId};
pub use api_error::{ApiError, ErrorKind, REQUEST_ID_HEADER};
pub use app_state::AppState;
