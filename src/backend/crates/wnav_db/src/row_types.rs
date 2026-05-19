// DB 行の中間型定義
// sqlx の FromRow を実装する中間行型（DB カラム → Rust 型のマッピング用）。
// Domain モデルとは別に定義し、各リポジトリが From<RowType> で変換する。

use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::FromRow;
use uuid::Uuid;

/// work_executions テーブルの行型（TBL-005）
#[derive(Debug, FromRow)]
pub struct WorkExecutionRow {
    pub work_execution_id: Uuid,
    pub sop_version_id: Uuid,
    pub primary_worker_id: Uuid,
    pub secondary_worker_id: Option<Uuid>,
    pub terminal_id: Uuid,
    pub production_target_id: Option<String>,
    pub status: String,
    pub current_step_index: i32,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub updated_at: DateTime<Utc>,
}

/// work_events テーブルの行型（TBL-001）
#[derive(Debug, FromRow)]
pub struct WorkEventRow {
    pub event_id: Uuid,
    pub case_id: Uuid,
    pub activity: String,
    pub step_id: Option<Uuid>,
    pub timestamp_client: DateTime<Utc>,
    pub timestamp_server: DateTime<Utc>,
    pub resource: Uuid,
    pub sop_version_id: Uuid,
    pub terminal_id: Uuid,
    pub payload: Value,
    pub prev_hash: String,
    pub content_hash: String,
}

/// master_versions テーブルの行型（TBL-037）
#[derive(Debug, FromRow)]
pub struct MasterVersionRow {
    pub master_version_id: Uuid,
    pub sop_id: Uuid,
    pub version_number: String,
    pub status: String,
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<DateTime<Utc>>,
    pub published_at: Option<DateTime<Utc>>,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// users テーブルの行型（TBL-016）
#[derive(Debug, FromRow)]
pub struct UserRow {
    pub user_id: Uuid,
    pub login_id: String,
    pub password_hash: String,
    pub display_name: String,
    pub email: Option<String>,
    pub factory_id: Uuid,
    /// JSONB 形式の roles 配列を文字列配列として受け取る
    pub roles: Value,
    pub skill_level: i16,
    pub is_active: bool,
}

/// sops テーブルの行型（TBL-007）
#[derive(Debug, FromRow)]
pub struct SopRow {
    pub sop_id: Uuid,
    pub operation_id: Uuid,
    pub name_json: Value,
    pub version: String,
    pub status: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// steps テーブルの行型（TBL-008）
#[derive(Debug, FromRow)]
pub struct StepRow {
    pub step_id: Uuid,
    pub sop_id: Uuid,
    pub step_number: i32,
    pub title: String,
    pub instruction: String,
    pub condition_dsl: Option<Value>,
    pub evidence_required: bool,
    pub sign_required: bool,
    pub skippable: bool,
    pub estimated_duration_secs: Option<i32>,
    pub step_type: String,
}

/// idempotency_keys テーブルの行型（TBL-035）
#[derive(Debug, FromRow)]
pub struct IdempotencyRow {
    pub idempotency_key: Uuid,
    pub response_body: Value,
    pub expires_at: DateTime<Utc>,
}

/// case_locks テーブルの行型（TBL-051）
#[derive(Debug, FromRow)]
pub struct CaseLockRow {
    pub case_id: Uuid,
    pub terminal_id: Uuid,
    pub locked_by: Uuid,
    pub locked_at: DateTime<Utc>,
    pub heartbeat_at: DateTime<Utc>,
    pub status: String,
}

/// outbox_events テーブルの行型（TBL-003）
#[derive(Debug, FromRow)]
pub struct OutboxEventRow {
    pub outbox_id: Uuid,
    pub event_id: Uuid,
    pub idempotency_key: Uuid,
    pub event_type: String,
    pub payload: Value,
    pub status: String,
    pub retry_count: i32,
    pub last_attempted_at: Option<DateTime<Utc>>,
}

/// evidence_files テーブルの行型（TBL-009）
#[derive(Debug, FromRow)]
pub struct EvidenceFileRow {
    pub evidence_id: Uuid,
    pub work_execution_id: Uuid,
    pub step_id: Uuid,
    /// BYTEA として取得した SHA-256 ハッシュ（32 バイト）
    pub file_hash: Vec<u8>,
    pub file_path: String,
    pub evidence_type: String,
    pub recorded_by: Uuid,
    pub client_recorded_at: DateTime<Utc>,
    pub server_received_at: DateTime<Utc>,
}

/// electronic_signatures テーブルの行型（TBL-011）
#[derive(Debug, FromRow)]
pub struct ElectronicSignatureRow {
    pub sign_id: Uuid,
    pub work_execution_id: Uuid,
    pub step_id: Uuid,
    pub signer_id: Uuid,
    pub signature_data: String,
    pub signed_at: DateTime<Utc>,
    pub verified: bool,
}

/// measurements テーブルの行型（TBL-010）
#[derive(Debug, FromRow)]
pub struct MeasurementRow {
    pub measurement_id: Uuid,
    pub work_execution_id: Uuid,
    pub step_id: Uuid,
    pub value: f64,
    pub unit: String,
    pub nominal: Option<f64>,
    pub upper_limit: Option<f64>,
    pub lower_limit: Option<f64>,
    pub cp: Option<f64>,
    pub cpk: Option<f64>,
}

/// andon_events テーブルの行型（TBL-015）
#[derive(Debug, FromRow)]
pub struct AndonEventRow {
    pub andon_id: Uuid,
    pub work_execution_id: Uuid,
    pub triggered_by: Uuid,
    pub reason_code: String,
    pub reason_text: Option<String>,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

/// capas テーブルの行型（TBL-013）
#[derive(Debug, FromRow)]
pub struct CapaRow {
    pub capa_id: Uuid,
    pub andon_id: Option<Uuid>,
    pub deviation_id: Option<Uuid>,
    pub phase: String,
    pub assignee: Uuid,
    pub due_date: Option<DateTime<Utc>>,
    pub description: String,
    pub root_cause_json: Option<Value>,
    pub corrective_action: Option<String>,
    pub closed_at: Option<DateTime<Utc>>,
}

/// kaizen_reports テーブルの行型（TBL-014）
#[derive(Debug, FromRow)]
pub struct KaizenReportRow {
    pub report_id: Uuid,
    pub reporter_id: Uuid,
    pub category: String,
    pub title: String,
    pub description: String,
    pub status: String,
    pub impact_level: String,
}

/// incoming_inspections テーブルの行型（TBL-038）
#[derive(Debug, FromRow)]
pub struct IncomingInspectionRow {
    pub qc_case_id: Uuid,
    pub lot_id: Uuid,
    pub sop_id: Uuid,
    pub status: String,
    pub inspector_id: Uuid,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub result: Option<String>,
    pub prev_hash: String,
    pub content_hash: String,
    pub chain_hash: String,
}

/// reworks テーブルの行型（TBL-043）
#[derive(Debug, FromRow)]
pub struct ReworkRow {
    pub rework_id: Uuid,
    pub parent_nonconformity_id: Uuid,
    pub lot_id: Uuid,
    pub sop_id: Uuid,
    pub status: String,
    pub assignee: Uuid,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub prev_hash: String,
    pub content_hash: String,
    pub chain_hash: String,
}

/// dispositions テーブルの行型（TBL-047）
#[derive(Debug, FromRow)]
pub struct DispositionRow {
    pub disposition_id: Uuid,
    pub lot_id: Uuid,
    pub rework_id: Option<Uuid>,
    pub disposition_type: String,
    pub approved_by_1: Option<Uuid>,
    pub approved_by_2: Option<Uuid>,
    pub approved_at: Option<DateTime<Utc>>,
    pub prev_hash: String,
    pub content_hash: String,
    pub chain_hash: String,
}

/// work_assignments テーブルの行型（TBL-052）
#[derive(Debug, FromRow)]
pub struct WorkAssignmentRow {
    pub assignment_id: Uuid,
    pub sop_id: Uuid,
    pub case_id: Uuid,
    pub lot_id: Option<Uuid>,
    pub priority: i32,
    pub status: String,
    pub target_terminal_id: Option<Uuid>,
    pub external_system: Option<String>,
    pub idempotency_key: Uuid,
    pub received_at: DateTime<Utc>,
}

/// hash_chain_blocks テーブルの行型（TBL-031）
#[derive(Debug, FromRow)]
pub struct HashChainBlockRow {
    pub block_id: Uuid,
    pub case_id: Uuid,
    pub sequence_number: i64,
    /// BYTEA 32 バイト: prev_block_hash
    pub prev_block_hash: Vec<u8>,
    /// BYTEA 32 バイト: content_hash
    pub content_hash: Vec<u8>,
    /// BYTEA 32 バイト: block_hash
    pub block_hash: Vec<u8>,
    pub created_at: DateTime<Utc>,
}

/// auth_logs テーブルの行型（TBL-032）
#[derive(Debug, FromRow)]
pub struct AuthLogRow {
    pub log_id: Uuid,
    pub user_id: Option<Uuid>,
    pub login_id: Option<String>,
    pub event_type: String,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub created_at: DateTime<Utc>,
}
