// 帳票生成ハンドラ（API-reports-001〜002）
//
// SOP 実行記録帳票（POST /reports/sop-execution）と XES 監査帳票（POST /reports/audit-xes）を担当する。
// audit-xes は条件指定が複雑なため GET ではなく POST を使用する。
// SQLX_OFFLINE=true 環境のため sqlx::query() 動的クエリを使用する。

use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    dto::reports::{ReportGenerateRequest, ReportJobResponse, ReportType},
    error::AppError,
    state::AppState,
};
// NOTE: parse_report_type は audit-xes 移行により不要となった
use wnav_auth::{AuditorRole, AuthenticatedUser};

/// XES 形式監査帳票の条件指定リクエスト（POST /api/v1/reports/audit-xes）
#[derive(Debug, Deserialize, Serialize, utoipa::ToSchema)]
pub struct AuditXesRequest {
    /// 対象期間（開始）
    pub from: chrono::DateTime<Utc>,
    /// 対象期間（終了）
    pub to: chrono::DateTime<Utc>,
    /// 対象工程 ID（省略時は全工程）
    pub process_id: Option<Uuid>,
    /// 対象作業者 ID（省略時は全作業者）
    pub worker_id: Option<Uuid>,
    /// 出力フォーマット（デフォルト: xes）
    pub format: Option<String>,
}

/// XES 帳票ジョブレスポンス
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct AuditXesJobResponse {
    pub job_id: Uuid,
    pub status: String,
    pub requested_at: chrono::DateTime<Utc>,
    pub estimated_completion_at: Option<chrono::DateTime<Utc>>,
}

/// SOP 実行記録帳票生成リクエスト（POST /api/v1/reports/sop-execution）。
///
/// AuditorRole 以上が必要。非同期ジョブを起動してジョブ ID を返す。
#[utoipa::path(
    post,
    path = "/api/v1/reports/sop-execution",
    tag = "reports",
    security(("Bearer" = [])),
    request_body = ReportGenerateRequest,
    responses(
        (status = 202, description = "ジョブ登録成功", body = ReportJobResponse),
        (status = 401, description = "未認証"),
        (status = 403, description = "権限不足"),
    )
)]
pub async fn generate_report(
    user: AuthenticatedUser<AuditorRole>,
    State(state): State<AppState>,
    Json(req): Json<ReportGenerateRequest>,
) -> Result<impl IntoResponse, AppError> {
    let job_id = Uuid::now_v7();
    let now = Utc::now();
    let report_type_str = format_report_type(&req.report_type);

    sqlx::query(
        r#"
        INSERT INTO report_jobs
            (id, report_type, status, requested_by, from_date, to_date,
             filters, format, created_at)
        VALUES ($1, $2, 'queued', $3, $4, $5, $6, $7, $8)
        "#,
    )
    .bind(job_id)
    .bind(&report_type_str)
    .bind(user.user_id)
    .bind(req.from)
    .bind(req.to)
    .bind(&req.filters)
    .bind(req.format.as_deref().unwrap_or("pdf"))
    .bind(now)
    .execute(&state.write_pool)
    .await?;

    tracing::info!(
        event = "report.job.queued",
        job_id = %job_id,
        report_type = %report_type_str,
        requested_by = %user.user_id,
        "帳票生成ジョブをキューに登録しました",
    );

    Ok((
        StatusCode::ACCEPTED,
        Json(ReportJobResponse {
            job_id,
            report_type: req.report_type,
            status: "queued".to_string(),
            created_at: now,
            estimated_completion_at: None,
        }),
    ))
}

/// XES 形式監査帳票生成（POST /api/v1/reports/audit-xes）。
///
/// 条件指定が複雑なため GET ではなく POST を使用する。AuditorRole 以上が必要。
#[utoipa::path(
    post,
    path = "/api/v1/reports/audit-xes",
    tag = "reports",
    security(("Bearer" = [])),
    request_body = AuditXesRequest,
    responses(
        (status = 202, description = "XES 帳票ジョブ登録成功", body = AuditXesJobResponse),
        (status = 401, description = "未認証"),
        (status = 403, description = "権限不足"),
    )
)]
pub async fn audit_xes(
    user: AuthenticatedUser<AuditorRole>,
    State(state): State<AppState>,
    Json(req): Json<AuditXesRequest>,
) -> Result<impl IntoResponse, AppError> {
    let job_id = Uuid::now_v7();
    let now = Utc::now();
    let format = req.format.as_deref().unwrap_or("xes");

    // XES 監査帳票ジョブを非同期キューに登録する
    sqlx::query(
        r#"
        INSERT INTO report_jobs
            (id, report_type, status, requested_by, from_date, to_date,
             filters, format, created_at)
        VALUES ($1, 'audit_trail', 'queued', $2, $3, $4, $5, $6, $7)
        "#,
    )
    .bind(job_id)
    .bind(user.user_id)
    .bind(req.from)
    .bind(req.to)
    .bind(serde_json::json!({
        "process_id": req.process_id,
        "worker_id": req.worker_id,
    }))
    .bind(format)
    .bind(now)
    .execute(&state.write_pool)
    .await?;

    tracing::info!(
        event = "report.audit_xes.queued",
        job_id = %job_id,
        requested_by = %user.user_id,
        "XES 監査帳票ジョブをキューに登録しました",
    );

    Ok((
        StatusCode::ACCEPTED,
        Json(AuditXesJobResponse {
            job_id,
            status: "queued".to_string(),
            requested_at: now,
            estimated_completion_at: None,
        }),
    ))
}

/// ReportType を DB 格納文字列に変換するヘルパー
fn format_report_type(rt: &ReportType) -> String {
    match rt {
        ReportType::SopExecutionRecord => "sop_execution_record",
        ReportType::NonConformanceRecord => "non_conformance_record",
        ReportType::ImprovementRecord => "improvement_record",
        ReportType::InspectionRecord => "inspection_record",
        ReportType::AggregationReport => "aggregation_report",
        ReportType::KpiDashboard => "kpi_dashboard",
        ReportType::AuditTrail => "audit_trail",
        ReportType::CostAnalysis => "cost_analysis",
        ReportType::TraceabilityRecord => "traceability_record",
    }
    .to_string()
}

/// DB 格納文字列を ReportType に変換するヘルパー（将来の参照用に残す）
#[allow(dead_code)]
fn parse_report_type(s: &str) -> ReportType {
    match s {
        "sop_execution_record" => ReportType::SopExecutionRecord,
        "non_conformance_record" => ReportType::NonConformanceRecord,
        "improvement_record" => ReportType::ImprovementRecord,
        "inspection_record" => ReportType::InspectionRecord,
        "aggregation_report" => ReportType::AggregationReport,
        "kpi_dashboard" => ReportType::KpiDashboard,
        "audit_trail" => ReportType::AuditTrail,
        "cost_analysis" => ReportType::CostAnalysis,
        _ => ReportType::TraceabilityRecord,
    }
}
