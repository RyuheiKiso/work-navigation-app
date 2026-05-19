// 帳票生成 DTO（API-reports-001〜002）
//
// 帳票生成リクエスト（非同期ジョブ）と帳票取得の型。

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

/// 帳票種別（RP-001〜010）
#[derive(Debug, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ReportType {
    /// RP-001: SOP 実行記録
    SopExecutionRecord,
    /// RP-003: 不適合記録
    NonConformanceRecord,
    /// RP-004: 改善記録
    ImprovementRecord,
    /// RP-005: 検査記録
    InspectionRecord,
    /// RP-006: 集計レポート
    AggregationReport,
    /// RP-007: KPI ダッシュボード
    KpiDashboard,
    /// RP-008: 監査証跡
    AuditTrail,
    /// RP-009: コスト分析
    CostAnalysis,
    /// RP-010: トレサビ記録
    TraceabilityRecord,
}

/// 帳票生成リクエスト（API-reports-001）
///
/// 非同期処理。ジョブ ID を返し、完了後に GET /api/v1/reports/{id} で取得する。
#[derive(Debug, Deserialize, ToSchema)]
pub struct ReportGenerateRequest {
    /// 帳票種別
    pub report_type: ReportType,
    /// 対象期間 開始
    pub from: DateTime<Utc>,
    /// 対象期間 終了
    pub to: DateTime<Utc>,
    /// 追加フィルタパラメータ（任意）
    pub filters: Option<serde_json::Value>,
    /// 出力フォーマット（"pdf" または "xlsx"）
    pub format: Option<String>,
}

/// 帳票ジョブレスポンス（API-reports-001）
///
/// ジョブ ID を返す。完了確認は GET /api/v1/reports/{id} で行う。
#[derive(Debug, Serialize, ToSchema)]
pub struct ReportJobResponse {
    /// ジョブ ID（帳票取得に使用する）
    pub job_id: Uuid,
    /// 帳票種別
    pub report_type: ReportType,
    /// ジョブステータス（"queued", "running", "completed", "failed"）
    pub status: String,
    /// 生成開始日時
    pub created_at: DateTime<Utc>,
    /// 予想完了時刻（参考値）
    pub estimated_completion_at: Option<DateTime<Utc>>,
}

/// 帳票取得レスポンス（API-reports-002）
#[derive(Debug, Serialize, ToSchema)]
pub struct ReportResponse {
    /// ジョブ ID
    pub job_id: Uuid,
    /// 帳票種別
    pub report_type: ReportType,
    /// ジョブステータス
    pub status: String,
    /// 帳票ダウンロード URL（完了時のみ）
    pub download_url: Option<String>,
    /// 帳票の有効期限（ダウンロード URL の有効期限）
    pub expires_at: Option<DateTime<Utc>>,
    /// エラーメッセージ（失敗時のみ）
    pub error: Option<String>,
    /// 生成完了日時
    pub completed_at: Option<DateTime<Utc>>,
}
