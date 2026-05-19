// wnav_terminal_api ルータ定義（MOD-BE-001 §3 / FNC-BE-016）
//
// 全エンドポイントを登録する Router を生成する。
// utoipa で OpenAPI 3.1 スキーマを自動生成する。

use axum::{
    Router,
    routing::{get, post, put},
};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::{
    api::{
        andon, auth, electronic_signatures, evidences, health, iqc, kaizen, nonconformities,
        reworks, sse, step_events, sync, work_assignments, work_executions, work_orders,
    },
    state::AppState,
};

/// OpenAPI ドキュメント定義（utoipa 自動生成）
///
/// GET /api/v1/openapi.json で配信する。
#[derive(OpenApi)]
#[openapi(
    info(
        title = "wnav_terminal_api",
        version = "0.1.0",
        description = "作業ナビゲーションシステム 端末 API（ハンディ端末向け）",
    ),
    paths(
        auth::login,
        auth::refresh,
        auth::logout,
        work_orders::list_work_orders,
        work_orders::create_work_order,
        work_orders::get_work_order,
        work_executions::start_work_execution,
        work_executions::get_work_execution,
        work_executions::suspend_work_execution,
        work_executions::resume_work_execution,
        work_executions::complete_work_execution,
        work_executions::heartbeat_work_execution,
        step_events::post_step_event,
        evidences::upload_evidence,
        electronic_signatures::create_electronic_signature,
        electronic_signatures::list_electronic_signatures,
        electronic_signatures::get_electronic_signature,
        sync::sync_master,
        sync::sync_outbox_inbound,
        work_assignments::list_work_assignments,
        work_assignments::ack_work_assignment,
        sse::sse_assignments,
        andon::create_andon_alert,
        kaizen::create_kaizen_proposal,
        iqc::create_inspection,
        iqc::add_measurement,
        reworks::create_rework,
        reworks::create_rework_verification,
        nonconformities::register_nonconformity,
        health::healthz,
        health::readyz,
    ),
    tags(
        (name = "auth", description = "認証・認可"),
        (name = "work-orders", description = "作業指示"),
        (name = "work-executions", description = "作業実行"),
        (name = "step-events", description = "ステップイベント"),
        (name = "evidences", description = "エビデンス"),
        (name = "electronic-signatures", description = "電子サイン"),
        (name = "sync", description = "同期"),
        (name = "work-assignments", description = "作業割当"),
        (name = "sse", description = "SSE"),
        (name = "andon", description = "アンドン"),
        (name = "kaizen", description = "改善提案"),
        (name = "iqc", description = "入荷検査"),
        (name = "reworks", description = "リワーク"),
        (name = "nonconformities", description = "非適合品"),
        (name = "health", description = "ヘルスチェック"),
    ),
    security(
        ("bearer_auth" = [])
    ),
    modifiers(&BearerSecurityAddon),
)]
struct ApiDoc;

/// Bearer JWT 認証スキームを OpenAPI components に登録するモディファイア
struct BearerSecurityAddon;

impl utoipa::Modify for BearerSecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        // components が存在しない場合は新規作成する
        let components = openapi.components.get_or_insert_with(Default::default);
        // "bearer_auth" という名前で Bearer JWT スキームを登録する
        components.add_security_scheme(
            "bearer_auth",
            utoipa::openapi::security::SecurityScheme::Http(
                utoipa::openapi::security::HttpBuilder::new()
                    .scheme(utoipa::openapi::security::HttpAuthScheme::Bearer)
                    .bearer_format("JWT")
                    .build(),
            ),
        );
    }
}

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
        // ─── ハートビート（API-work-execs-006）────────────────────────
        .route(
            "/work-executions/{id}/heartbeat",
            put(work_executions::heartbeat_work_execution),
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
        // ─── OpenAPI JSON 配信 ────────────────────────────────────────
        .route(
            "/openapi.json",
            get(|| async { axum::Json(ApiDoc::openapi()) }),
        )
        // ─── システム ─────────────────────────────────────────────────
        .route("/readyz", get(health::readyz));

    Router::new()
        // Swagger UI（/swagger-ui で OpenAPI ドキュメントを閲覧する）
        .merge(SwaggerUi::new("/swagger-ui").url("/api/v1/openapi.json", ApiDoc::openapi()))
        // /healthz は /api/v1 プレフィックスなし（liveness probe 用）
        .route("/healthz", get(health::healthz))
        .route("/api/v1/healthz", get(health::healthz))
        // /api/v1 プレフィックス配下をネストする
        .nest("/api/v1", api_v1)
}
