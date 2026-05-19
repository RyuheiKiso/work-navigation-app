// 作業実行サービス Trait（FNC-BE-001〜005）
// Application 層インターフェース。
// ロックステップ強制・証拠必須検証・スキルゲート等のドメインルールを集約する。

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::DomainError;
use crate::model::work_event::WorkEvent;
use crate::model::work_execution::WorkExecution;

/// 作業実行ユースケースのサービス Trait（Application 層インターフェース）。
/// start_work / complete_step / suspend / resume / complete_work の 5 操作を定義する。
#[async_trait]
pub trait WorkExecutionService: Send + Sync + 'static {
    /// (FNC-BE-001) 作業を開始し、WorkExecution を作成して WorkEvent(work.started) を記録する。
    /// スキルゲート（BR-BUS-002/041）を検証する。
    async fn start_work(&self, cmd: StartWorkCmd) -> Result<WorkExecution, DomainError>;

    /// (FNC-BE-002) Step を完了し、WorkEvent(step.completed) を記録する。
    /// ロックステップ強制（BR-BUS-001）・証拠必須検証（BR-BUS-003）を行う。
    async fn complete_step(&self, cmd: CompleteStepCmd) -> Result<WorkEvent, DomainError>;

    /// (FNC-BE-003) 作業を中断し、WorkEvent(work.suspended) を記録する。
    async fn suspend(&self, cmd: SuspendCmd) -> Result<Suspension, DomainError>;

    /// (FNC-BE-004) 中断された作業を再開し、WorkEvent(work.resumed) を記録する。
    async fn resume(&self, cmd: ResumeCmd) -> Result<WorkExecution, DomainError>;

    /// 作業を完了し、WorkEvent(work.completed) を記録する。
    /// 全 Step が完了していることを検証する。
    async fn complete_work(&self, cmd: CompleteWorkCmd) -> Result<WorkExecution, DomainError>;
}

/// 作業開始コマンド（FNC-BE-001）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartWorkCmd {
    /// 作業実行 ID（UUID v7。クライアントが事前生成）
    pub work_execution_id: Uuid,
    /// SOP バージョン ID
    pub sop_version_id: Uuid,
    /// 主担当作業員 ID
    pub primary_worker_id: Uuid,
    /// 補助担当作業員 ID（任意）
    pub secondary_worker_id: Option<Uuid>,
    /// 端末 ID
    pub terminal_id: Uuid,
    /// 生産対象 ID（ロット・シリアル等）
    pub production_target_id: Option<String>,
    /// 冪等性キー（UUID v7）
    pub idempotency_key: Uuid,
    /// クライアント記録日時（申告値）
    pub client_timestamp: DateTime<Utc>,
}

/// ステップ完了コマンド（FNC-BE-002）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompleteStepCmd {
    /// 作業実行 ID
    pub work_execution_id: Uuid,
    /// 完了するステップ ID
    pub step_id: Uuid,
    /// 作業員 ID
    pub worker_id: Uuid,
    /// 添付証拠 ID 一覧
    pub evidence_ids: Vec<Uuid>,
    /// 電子サイン ID（Signature ステップの場合）
    pub sign_id: Option<Uuid>,
    /// 測定値（Measurement ステップの場合）
    pub measurement_value: Option<serde_json::Value>,
    /// 冪等性キー（UUID v7）
    pub idempotency_key: Uuid,
    /// クライアント記録日時（申告値）
    pub client_timestamp: DateTime<Utc>,
}

/// 作業中断コマンド（FNC-BE-003）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuspendCmd {
    /// 作業実行 ID
    pub work_execution_id: Uuid,
    /// 中断者 ID
    pub worker_id: Uuid,
    /// 中断理由コード（列挙型。自由文字列禁止）
    pub reason_code: String,
    /// 中断理由テキスト（任意）
    pub reason_text: Option<String>,
    /// 楽観ロック用最終更新日時
    pub expected_updated_at: DateTime<Utc>,
    /// 冪等性キー（UUID v7）
    pub idempotency_key: Uuid,
    /// クライアント記録日時（申告値）
    pub client_timestamp: DateTime<Utc>,
}

/// 作業再開コマンド（FNC-BE-004）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResumeCmd {
    /// 作業実行 ID
    pub work_execution_id: Uuid,
    /// 再開者 ID
    pub worker_id: Uuid,
    /// 楽観ロック用最終更新日時
    pub expected_updated_at: DateTime<Utc>,
    /// 冪等性キー（UUID v7）
    pub idempotency_key: Uuid,
    /// クライアント記録日時（申告値）
    pub client_timestamp: DateTime<Utc>,
}

/// 作業完了コマンド。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompleteWorkCmd {
    /// 作業実行 ID
    pub work_execution_id: Uuid,
    /// 完了者 ID
    pub worker_id: Uuid,
    /// 楽観ロック用最終更新日時
    pub expected_updated_at: DateTime<Utc>,
    /// 冪等性キー（UUID v7）
    pub idempotency_key: Uuid,
    /// クライアント記録日時（申告値）
    pub client_timestamp: DateTime<Utc>,
}

/// 中断結果。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Suspension {
    /// 中断 ID（UUID v7）
    pub suspension_id: Uuid,
    /// 作業実行 ID
    pub work_execution_id: Uuid,
    /// 中断日時
    pub suspended_at: DateTime<Utc>,
    /// 中断理由コード
    pub reason_code: String,
}
