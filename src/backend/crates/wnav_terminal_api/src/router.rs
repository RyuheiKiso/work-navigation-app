// wnav_terminal_api ルータ定義（MOD-BE-001 §3 / FNC-BE-016）
//
// 全エンドポイントを登録する Router を生成する。
// utoipa で OpenAPI 3.1 スキーマを自動生成する。

use axum::{
    Router,
    routing::{get, post},
};

use crate::{
    api::{
        andon, auth, electronic_signatures, evidences, health, iqc, kaizen, nonconformities,
        reworks, sse, step_events, sync, work_assignments, work_executions, work_orders,
    },
    state::AppState,
};

/// wnav_terminal_api の全エンドポイントを登録した Router<AppState> を返す。
///
/// TLS 終端は IIS（Windows Server 2022）に委譲する。
/// 本バイナリは HTTP のみを受け付ける（ポート 8080）。
/// State はここでは解決せず、`main.rs` で `.with_state()` を呼び出す。
pub fn create_router() -> Router<AppState> {
    // /api/v1 プレフィックス配下のエンドポイント
    let api_v1 = Router::new()
        // ─── 認証 ─────────────────────────────────────────────────────
        .route("/auth/login", post(auth::login))
        .route("/auth/refresh", post(auth::refresh))
        .route("/auth/logout", post(auth::logout))
        // ─── 作業指示 ─────────────────────────────────────────────────
        .route("/work-orders", get(work_orders::list_work_orders))
        .route("/work-orders", post(work_orders::create_work_order))
        .route("/work-orders/{id}", get(work_orders::get_work_order))
        // ─── 作業実行 ─────────────────────────────────────────────────
        .route(
            "/work-executions",
            post(work_executions::start_work_execution),
        )
        .route(
            "/work-executions/{id}",
            get(work_executions::get_work_execution),
        )
        .route(
            "/work-executions/{id}/suspend",
            post(work_executions::suspend_work_execution),
        )
        .route(
            "/work-executions/{id}/resume",
            post(work_executions::resume_work_execution),
        )
        .route(
            "/work-executions/{id}/complete",
            post(work_executions::complete_work_execution),
        )
        // ─── ステップイベント ──────────────────────────────────────────
        .route(
            "/work-executions/{id}/events",
            post(step_events::post_step_event),
        )
        // ─── エビデンス ───────────────────────────────────────────────
        .route("/evidences", post(evidences::upload_evidence))
        // ─── 電子サイン ───────────────────────────────────────────────
        .route(
            "/electronic-signs",
            post(electronic_signatures::create_electronic_signature),
        )
        .route(
            "/electronic-signs",
            get(electronic_signatures::list_electronic_signatures),
        )
        .route(
            "/electronic-signs/{id}",
            get(electronic_signatures::get_electronic_signature),
        )
        // ─── 同期 ─────────────────────────────────────────────────────
        .route("/sync/master", get(sync::sync_master))
        .route("/sync/outbox/inbound", post(sync::sync_outbox_inbound))
        // ─── 作業割当（API-sync-005）──────────────────────────────────
        .route(
            "/work-assignments",
            get(work_assignments::list_work_assignments),
        )
        .route(
            "/work-assignments/{id}/ack",
            post(work_assignments::ack_work_assignment),
        )
        // ─── SSE（API-sync-004）──────────────────────────────────────
        .route("/sse/assignments", get(sse::sse_assignments))
        // ─── アンドン（terminal-api 担当分） ──────────────────────────
        .route("/alerts", post(andon::create_andon_alert))
        // ─── Kaizen 改善提案 ──────────────────────────────────────────
        .route("/kaizen-proposals", post(kaizen::create_kaizen_proposal))
        // ─── IQC（terminal-api 担当分: 入荷検査開始・測定値追加） ──────
        .route("/iqc/incoming-inspections", post(iqc::create_inspection))
        .route(
            "/iqc/incoming-inspections/{id}/measurements",
            post(iqc::add_measurement),
        )
        // ─── リワーク（terminal-api 担当分） ──────────────────────────
        .route("/reworks", post(reworks::create_rework))
        .route(
            "/rework-verifications",
            post(reworks::create_rework_verification),
        )
        // ─── 非適合品登録（terminal-api 担当分）──────────────────────
        .route(
            "/nonconformities",
            post(nonconformities::register_nonconformity),
        )
        // ─── システム ─────────────────────────────────────────────────
        .route("/readyz", get(health::readyz));

    Router::new()
        // /healthz は /api/v1 プレフィックスなし（liveness probe 用）
        .route("/healthz", get(health::healthz))
        .route("/api/v1/healthz", get(health::healthz))
        // /api/v1 プレフィックス配下をネストする
        .nest("/api/v1", api_v1)
}
