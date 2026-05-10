//! axum ルータ構築
//!
//! 対応 §: ロードマップ §7.3 §10.3.1 §10.5 §11.4.1

use axum::{
    middleware::{from_fn, from_fn_with_state},
    routing::{delete, get, post, put},
    Router,
};
use std::sync::Arc;
use wna_adapter::Hs256SessionFactory;
use wna_domain::PasswordHasher;

use crate::{
    app_state::AppState,
    handler_audit, handler_auth, handler_dashboard, handler_flows, handler_health,
    handler_master, handler_records, handler_tasks,
    middleware_auth::require_session,
    middleware_request_id::request_id,
};

#[must_use]
pub fn build_router<H>(state: AppState<H>) -> Router
where
    H: PasswordHasher + Send + Sync + Clone + 'static,
{
    let session_factory: Arc<Hs256SessionFactory> = state.session_factory.clone();

    let public = Router::new()
        .route("/healthz", get(handler_health::healthz))
        .route("/readyz", get(handler_health::readyz::<H>))
        .route("/auth/login", post(handler_auth::post_login::<H>))
        .with_state(state.clone());

    let protected = Router::new()
        .route("/tasks", get(handler_tasks::list_tasks::<H>))
        .route("/tasks/:id", get(handler_tasks::get_task::<H>))
        .route("/tasks/:id/start", post(handler_tasks::start_task::<H>))
        .route("/tasks/:id/complete", post(handler_tasks::complete_task::<H>))
        .route("/tasks/:id/suspend", post(handler_tasks::suspend_task::<H>))
        .route("/tasks/:id/resume", post(handler_tasks::resume_task::<H>))
        .route("/tasks/:id/abort", post(handler_tasks::abort_task::<H>))
        .route("/tasks/:id/steps", get(handler_tasks::list_steps::<H>))
        .route("/tasks/:id/steps/:step_id/done", post(handler_tasks::mark_step_done::<H>))
        .route("/tasks/:id/records", post(handler_records::append_record::<H>))
        .route("/flows", get(handler_flows::list_flows::<H>))
        .route("/flows/:id/trials", post(handler_flows::publish_trial::<H>))
        .route("/audit", get(handler_audit::list_audit::<H>))
        .route("/master/products", get(handler_master::list_products::<H>))
        .route("/master/products", put(handler_master::upsert_product::<H>))
        .route("/master/products/:code", delete(handler_master::delete_product::<H>))
        .route("/master/equipments", get(handler_master::list_equipments::<H>))
        .route("/master/equipments", put(handler_master::upsert_equipment::<H>))
        .route("/master/equipments/:code", delete(handler_master::delete_equipment::<H>))
        .route("/master/parts", get(handler_master::list_parts::<H>))
        .route("/master/parts", put(handler_master::upsert_part::<H>))
        .route("/master/parts/:code", delete(handler_master::delete_part::<H>))
        .route("/dashboard/tasks", get(handler_dashboard::list_dashboard_tasks::<H>))
        .route("/auth/me", get(handler_auth::me::<H>))
        .layer(from_fn_with_state(session_factory, require_session))
        .with_state(state);

    // 全ルートに request_id ミドルウェアを挟む。
    // 認証より外側で発行することで、401/403 のレスポンスにも ID が乗る。
    public.merge(protected).layer(from_fn(request_id))
}
