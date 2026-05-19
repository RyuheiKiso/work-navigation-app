// wnav_master_api ルータ定義
//
// 全エンドポイントを `/api/v1` プレフィックス配下に登録する。
// OpenAPI スキーマ自動生成（utoipa）と Swagger UI を提供する。
// TLS 終端は IIS（リバースプロキシ）に委譲するため、本クレートは HTTP のみを受け付ける。

use axum::{
    Router,
    routing::{get, patch, post, put},
};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::{
    api::{
        alerts, auth, capas, health, iqc, master, nonconformities, ops, public_config, reports,
        scraps, trace, work_assignments,
    },
    state::AppState,
};

/// OpenAPI ドキュメント定義
///
/// utoipa でハンドラから自動生成する。
/// GET /api/v1/openapi.json で最新スキーマを配信する（マスタ SPA のクライアント生成元）。
#[derive(OpenApi)]
#[openapi(
    info(
        title = "wnav_master_api",
        version = "0.1.0",
        description = "作業ナビゲーションシステム 管理 API（マスタメンテナンス・管理コンソール向け）",
    ),
    paths(
        // 認証
        auth::login,
        auth::refresh,
        auth::logout,
        auth::rotate_keys,
        // ヘルスチェック
        health::healthz,
        health::ops_health,
        // マスタバージョン管理
        master::list_versions,
        master::create_draft,
        master::update_draft,
        master::submit_version,
        master::approve_version,
        master::rollback_version,
        master::dry_run,
        // 補足マスタエンドポイント
        master::list_processes,
        master::list_sops,
        master::list_users,
        master::create_user,
        master::assign_roles,
        // 作業指示 Push 受信
        work_assignments::receive_work_assignment,
        // 運用・監視
        ops::list_dlq,
        ops::requeue,
        ops::verify_hash_chain,
        ops::trigger_master_sync,
        ops::metrics,
        // 帳票生成
        reports::generate_report,
        reports::audit_xes,
        // トレサビ
        trace::forward_trace,
        trace::backward_trace,
        // IQC（master-api 担当分: 合否判定・特採承認）
        iqc::submit_inspection,
        iqc::approve_inspection,
        iqc::create_disposition,
        // アンドン対応（master-api）
        alerts::acknowledge_alert,
        // CAPA 管理
        capas::create_capa,
        capas::update_capa,
        // 非適合品登録
        nonconformities::register_nonconformity,
        // 廃却・返品
        scraps::create_scrap_record,
        scraps::create_return_record,
        // 公開設定
        public_config::get_public_config,
    ),
    components(
        schemas(
            // 認証 DTO
            crate::dto::auth::LoginRequest,
            crate::dto::auth::LoginResponse,
            crate::dto::auth::RefreshRequest,
            crate::dto::auth::RefreshResponse,
            crate::dto::auth::LogoutRequest,
            crate::dto::auth::KeyRotateRequest,
            crate::dto::auth::KeyRotateResponse,
            // ヘルスチェック DTO
            crate::dto::health::HealthResponse,
            crate::dto::health::HealthComponents,
            crate::dto::health::ComponentStatus,
            // マスタ DTO
            crate::dto::master::MasterVersionStatus,
            crate::dto::master::MasterVersionSummary,
            crate::dto::master::MasterVersionResponse,
            crate::dto::master::MasterVersionListResponse,
            crate::dto::master::CreateMasterVersionRequest,
            crate::dto::master::UpdateMasterVersionRequest,
            crate::dto::master::SubmitVersionRequest,
            crate::dto::master::ApproveVersionRequest,
            crate::dto::master::RollbackVersionRequest,
            crate::dto::master::DryRunResult,
            crate::dto::master::DryRunError,
            // 作業指示 DTO
            crate::dto::work_assignments::WorkAssignmentPushRequest,
            crate::dto::work_assignments::WorkAssignmentPushResponse,
            // Ops DTO
            crate::dto::ops::DlqEntry,
            crate::dto::ops::DlqListResponse,
            crate::dto::ops::RequeueRequest,
            crate::dto::ops::RequeueResponse,
            crate::dto::ops::HashChainVerifyRequest,
            crate::dto::ops::HashChainVerifyResponse,
            crate::dto::ops::MasterSyncResponse,
            // 帳票 DTO
            crate::dto::reports::ReportType,
            crate::dto::reports::ReportGenerateRequest,
            crate::dto::reports::ReportJobResponse,
            crate::dto::reports::ReportResponse,
            // トレサビ DTO
            crate::dto::trace::TraceEvent,
            crate::dto::trace::CaseTraceResponse,
            crate::dto::trace::LotTraceNode,
            crate::dto::trace::LotTraceResponse,
            // IQC DTO
            crate::dto::iqc::IqcStatus,
            crate::dto::iqc::AqlJudgment,
            crate::dto::iqc::CreateIqcInspectionRequest,
            crate::dto::iqc::AddIqcMeasurementRequest,
            crate::dto::iqc::ApproveInspectionRequest,
            crate::dto::iqc::CreateDispositionRequest,
            crate::dto::iqc::IqcInspectionResponse,
            crate::dto::iqc::DispositionResponse,
            // アンドン対応 DTO
            crate::dto::alerts::AcknowledgeAlertRequest,
            crate::dto::alerts::AlertAcknowledgedResponse,
            // CAPA DTO
            crate::dto::capas::CreateCapaRequest,
            crate::dto::capas::UpdateCapaRequest,
            crate::dto::capas::CapaResponse,
            // 非適合品 DTO
            crate::dto::nonconformities::RegisterNonconformityRequest,
            crate::dto::nonconformities::NonconformityResponse,
            // 廃却・返品 DTO
            crate::dto::scraps::ScrapRecordRequest,
            crate::dto::scraps::ScrapRecordResponse,
            crate::dto::scraps::ReturnRecordRequest,
            crate::dto::scraps::ReturnRecordResponse,
            // 公開設定 DTO
            crate::dto::public_config::PublicConfigResponse,
        )
    ),
    tags(
        (name = "auth", description = "認証・認可 API"),
        (name = "health", description = "ヘルスチェック"),
        (name = "master", description = "マスタバージョン管理 API"),
        (name = "work_assignments", description = "作業指示 Push 受信 API"),
        (name = "ops", description = "運用・監視 API"),
        (name = "reports", description = "帳票生成 API"),
        (name = "trace", description = "トレサビ API"),
        (name = "iqc", description = "入荷検査 (IQC) API（判定・承認）"),
        (name = "alerts", description = "アンドン対応 API"),
        (name = "capas", description = "CAPA 管理 API"),
        (name = "nonconformities", description = "非適合品登録 API"),
        (name = "scraps", description = "廃却・返品記録 API"),
        (name = "public", description = "公開設定 API（認証不要）"),
    ),
    security(
        ("Bearer" = [])
    ),
    modifiers(&SecurityAddon),
)]
struct ApiDoc;

/// Bearer 認証スキームを OpenAPI に登録するモディファイア
struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.get_or_insert_with(Default::default);
        components.add_security_scheme(
            "Bearer",
            utoipa::openapi::security::SecurityScheme::Http(
                utoipa::openapi::security::HttpBuilder::new()
                    .scheme(utoipa::openapi::security::HttpAuthScheme::Bearer)
                    .bearer_format("JWT")
                    .build(),
            ),
        );
    }
}

/// wnav_master_api のルータを生成して返す。
///
/// 全エンドポイントを `/api/v1` プレフィックス配下に配置する。
/// Swagger UI は `/swagger-ui/` で提供する（開発用）。
pub fn create_router(state: AppState) -> Router {
    // API v1 ルータ
    let api_v1 = Router::new()
        // ── 認証 ──────────────────────────────────────────────────────────
        .route("/auth/login", post(auth::login))
        .route("/auth/refresh", post(auth::refresh))
        .route("/auth/logout", post(auth::logout))
        .route("/auth/keys/rotate", post(auth::rotate_keys))

        // ── マスタバージョン管理 ──────────────────────────────────────────
        .route("/master-versions", get(master::list_versions))
        .route("/master-versions/draft", post(master::create_draft))
        .route("/master-versions/:id", patch(master::update_draft))
        .route("/master-versions/:id/submit", post(master::submit_version))
        .route("/master-versions/:id/approve", post(master::approve_version))
        .route("/master-versions/:id/rollback", post(master::rollback_version))
        .route("/master-versions/:id/dry-run", post(master::dry_run))

        // ── 補足マスタエンドポイント ──────────────────────────────────────
        .route("/master/processes", get(master::list_processes))
        .route("/master/sops", get(master::list_sops))
        .route("/master/users", get(master::list_users))
        .route("/master/users", post(master::create_user))
        .route("/master/users/:id/roles", put(master::assign_roles))

        // ── 作業指示 Push 受信 ─────────────────────────────────────────────
        .route("/work-assignments", post(work_assignments::receive_work_assignment))

        // ── 運用・監視 ────────────────────────────────────────────────────
        .route("/ops/health", get(health::ops_health))
        .route("/ops/metrics", get(ops::metrics))
        .route("/ops/outbox/dlq", get(ops::list_dlq))
        .route("/ops/outbox/:id/requeue", post(ops::requeue))
        .route("/ops/hash-chain/verify", post(ops::verify_hash_chain))
        .route("/ops/master-sync", post(ops::trigger_master_sync))

        // ── 帳票生成 ──────────────────────────────────────────────────────
        // SOP 実行記録帳票（非同期ジョブ登録）
        .route("/reports/sop-execution", post(reports::generate_report))
        // XES 形式監査帳票（条件指定が複雑なため POST を使用する）
        .route("/reports/audit-xes", post(reports::audit_xes))

        // ── トレサビ ──────────────────────────────────────────────────────
        // 順方向トレース: case_id でイベントを全件取得
        .route("/trace/forward", get(trace::forward_trace))
        // 逆方向トレース: lot_id でイベントを全件取得
        .route("/trace/backward", get(trace::backward_trace))

        // ── 入荷検査（IQC）master-api 担当分: 合否判定・特採承認 ──────────
        .route("/iqc/incoming-inspections/:id/judge", post(iqc::submit_inspection))
        .route("/iqc/incoming-inspections/:id/concession", post(iqc::approve_inspection))
        .route("/dispositions", post(iqc::create_disposition))

        // ── アンドン対応（master-api）─────────────────────────────────────
        // 管理コンソールからのアラート対応・解決（AuditorRole 以上必須）
        .route("/alerts/:id/acknowledge", patch(alerts::acknowledge_alert))

        // ── CAPA 管理（master-api）───────────────────────────────────────
        .route("/capas", post(capas::create_capa))
        .route("/capas/:id", patch(capas::update_capa))

        // ── 非適合品登録（master-api）────────────────────────────────────
        .route("/nonconformities", post(nonconformities::register_nonconformity))

        // ── 廃却・返品 ────────────────────────────────────────────────────
        .route("/scrap-records", post(scraps::create_scrap_record))
        .route("/return-records", post(scraps::create_return_record))

        // ── 公開設定（認証不要）──────────────────────────────────────────
        .route("/public/config", get(public_config::get_public_config))

        // ── OpenAPI スキーマ配信 ──────────────────────────────────────────
        .route(
            "/openapi.json",
            get(|| async { axum::Json(ApiDoc::openapi()) }),
        );

    // Swagger UI（開発環境用）
    let swagger = SwaggerUi::new("/swagger-ui")
        .url("/api/v1/openapi.json", ApiDoc::openapi());

    Router::new()
        // システムレベルのヘルスチェック（認証不要）
        .route("/healthz", get(health::healthz))
        // API v1 プレフィックス
        .nest("/api/v1", api_v1)
        // Swagger UI
        .merge(swagger)
        // AppState を全ハンドラに共有する
        .with_state(state)
}
