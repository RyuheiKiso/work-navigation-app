// 作業実行 API（API-work-execs-001〜005）の DTO 定義（03_作業実行API仕様.md §3〜7）

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

/// SOP バージョンスナップショット情報
#[derive(Debug, Serialize, ToSchema)]
pub struct SopVersionSnapshot {
    pub sop_id: Uuid,
    pub version: String,
    pub snapshot_hash: String,
}

/// 作業実行イベントサマリ（GET レスポンス内の events 配列要素）
#[derive(Debug, Serialize, ToSchema)]
pub struct WorkEventSummary {
    pub event_id: Uuid,
    pub activity: String,
    pub step_id: Option<Uuid>,
    pub step_number: Option<i32>,
    pub timestamp_server: DateTime<Utc>,
}

// ─────────────────────────────────────────────────────────────────────────────
// API-work-execs-001: POST /api/v1/work-executions
// ─────────────────────────────────────────────────────────────────────────────

/// 作業開始リクエスト（API-work-execs-001）
#[derive(Debug, Deserialize, ToSchema)]
#[allow(dead_code)]
pub struct StartWorkRequest {
    /// 対象作業指示 ID（TBL-006 に open ステータスで存在すること）
    pub work_order_id: Uuid,
    /// 実行オペレータ ID（TBL-016 に存在し、必要スキルを保有すること）
    pub operator_id: Uuid,
    /// 実行端末 ID（TBL-033 に存在すること）
    pub device_id: Uuid,
    /// クライアント側の開始時刻（監査用）
    pub start_timestamp_client: DateTime<Utc>,
}

/// 作業開始レスポンスの data フィールド（API-work-execs-001）
#[derive(Debug, Serialize, ToSchema)]
pub struct StartWorkData {
    pub id: Uuid,
    pub work_order_id: Uuid,
    pub operator_id: Uuid,
    pub device_id: Uuid,
    pub status: String,
    pub current_step_id: Option<Uuid>,
    pub sop_version_snapshot: SopVersionSnapshot,
    pub started_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

// ─────────────────────────────────────────────────────────────────────────────
// API-work-execs-002: GET /api/v1/work-executions/{id}
// ─────────────────────────────────────────────────────────────────────────────

/// 作業実行詳細レスポンスの data フィールド（API-work-execs-002）
#[derive(Debug, Serialize, ToSchema)]
pub struct WorkExecutionDetailData {
    pub id: Uuid,
    pub work_order_id: Uuid,
    pub operator_id: Uuid,
    pub device_id: Uuid,
    pub status: String,
    pub current_step_id: Option<Uuid>,
    pub completed_step_count: i32,
    pub total_step_count: i32,
    pub sop_version_snapshot: SopVersionSnapshot,
    pub started_at: DateTime<Utc>,
    pub last_event_at: Option<DateTime<Utc>>,
    pub events: Vec<WorkEventSummary>,
}

// ─────────────────────────────────────────────────────────────────────────────
// API-work-execs-003: POST /api/v1/work-executions/{id}/suspend
// ─────────────────────────────────────────────────────────────────────────────

/// 作業中断リクエスト（API-work-execs-003）
#[derive(Debug, Deserialize, ToSchema)]
#[allow(dead_code)]
pub struct SuspendRequest {
    /// 中断理由コード（equipment_breakdown / material_shortage / quality_issue / emergency / other）
    pub reason_code: String,
    /// 補足説明（任意、最大 500 文字）
    pub reason_detail: Option<String>,
    /// クライアント側の中断時刻
    pub timestamp_client: DateTime<Utc>,
}

/// 作業中断レスポンスの data フィールド（API-work-execs-003）
#[derive(Debug, Serialize, ToSchema)]
pub struct SuspendData {
    pub id: Uuid,
    pub status: String,
    pub suspension_id: Uuid,
    pub suspended_at: DateTime<Utc>,
}

// ─────────────────────────────────────────────────────────────────────────────
// API-work-execs-004: POST /api/v1/work-executions/{id}/resume
// ─────────────────────────────────────────────────────────────────────────────

/// 作業再開リクエスト（API-work-execs-004）
#[derive(Debug, Deserialize, ToSchema)]
#[allow(dead_code)]
pub struct ResumeRequest {
    /// 再開実行者 ID（TBL-016 に存在すること）
    pub resumed_by: Uuid,
    /// クライアント側の再開時刻
    pub timestamp_client: DateTime<Utc>,
}

/// 作業再開レスポンスの data フィールド（API-work-execs-004）
#[derive(Debug, Serialize, ToSchema)]
pub struct ResumeData {
    pub id: Uuid,
    pub status: String,
    pub resumed_at: DateTime<Utc>,
    pub current_step_id: Option<Uuid>,
}

// ─────────────────────────────────────────────────────────────────────────────
// API-work-execs-005: POST /api/v1/work-executions/{id}/complete
// ─────────────────────────────────────────────────────────────────────────────

/// 作業完了リクエスト（API-work-execs-005）
#[derive(Debug, Deserialize, ToSchema)]
#[allow(dead_code)]
pub struct CompleteWorkRequest {
    /// 完了実行者 ID（TBL-016 に存在すること）
    pub completed_by: Uuid,
    /// クライアント側の完了時刻
    pub timestamp_client: DateTime<Utc>,
    /// 最終備考（任意、最大 1000 文字）
    pub final_remarks: Option<String>,
}

/// 作業完了レスポンスの data フィールド（API-work-execs-005）
#[derive(Debug, Serialize, ToSchema)]
pub struct CompleteWorkData {
    pub id: Uuid,
    pub status: String,
    pub completed_at: DateTime<Utc>,
    pub hash_chain_block_id: Option<Uuid>,
    pub hash_chain_value: Option<String>,
}
