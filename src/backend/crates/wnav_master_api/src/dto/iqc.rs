// 入荷検査 (IQC) DTO（API-iqc-001〜005）
//
// 入荷検査登録・測定値追加・検査完了・特採承認・ディスポジション登録の型。

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

/// IQC 検査状態
#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum IqcStatus {
    /// 検査開始
    InProgress,
    /// AQL 合格（自動判定）
    Passed,
    /// AQL 不合格（特採待ち）
    Failed,
    /// 特採承認済み
    ConcessionallyApproved,
    /// 返品・廃棄決定
    Rejected,
}

/// IQC AQL 判定結果
#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum AqlJudgment {
    /// 合格
    Accept,
    /// 不合格
    Reject,
    /// 保留（追加検査が必要）
    Hold,
}

// CreateIqcInspectionRequest と AddIqcMeasurementRequest は terminal-api に移管済み。
// これらの型は terminal_api/src/dto/iqc.rs で定義されている。
// master-api は合否判定（judge）・特採承認（concession）・ディスポジションのみを担当する。

/// IQC 特採承認リクエスト（API-iqc-004）
///
/// ApproverRole 必須。申請者と異なるユーザーが承認する。
#[derive(Debug, Deserialize, ToSchema)]
pub struct ApproveInspectionRequest {
    /// 特採理由（必須）
    pub concession_reason: String,
    /// 使用制限条件
    pub use_restrictions: Option<String>,
    /// 承認者コメント
    pub approver_comment: Option<String>,
}

/// ディスポジション登録リクエスト（API-iqc-005）
///
/// Two-Person Integrity 必須（FR-AU-007）。
/// 登録者（current_user）と承認者（approver_id）が異なるユーザーである必要がある。
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateDispositionRequest {
    /// 対象 IQC 検査 ID
    pub inspection_id: Uuid,
    /// ディスポジション種別（"use_as_is", "rework", "scrap", "return"）
    pub disposition_type: String,
    /// ディスポジション理由
    pub reason: String,
    /// 第二承認者 ID（Two-Person Integrity: 登録者と異なるユーザー）
    pub approver_id: Uuid,
    /// 承認コメント
    pub approver_comment: Option<String>,
}

/// IQC 検査レスポンス（API-iqc-001〜004）
#[derive(Debug, Serialize, ToSchema)]
pub struct IqcInspectionResponse {
    /// 検査 ID
    pub id: Uuid,
    /// 受入ロット ID
    pub lot_id: String,
    /// サプライヤー ID
    pub supplier_id: String,
    /// 品目コード
    pub part_number: String,
    /// 受入数量
    pub received_quantity: i64,
    /// 抜取数量
    pub sample_size: i64,
    /// 検査状態
    pub status: IqcStatus,
    /// AQL 判定結果（完了後に設定）
    pub aql_judgment: Option<AqlJudgment>,
    /// 不合格数計
    pub total_defects: Option<i64>,
    /// 登録者 ID
    pub created_by: Uuid,
    /// 登録日時
    pub created_at: DateTime<Utc>,
    /// 完了日時
    pub completed_at: Option<DateTime<Utc>>,
    /// IQC ハッシュチェーンの直近ハッシュ（16進数）
    pub current_hash: Option<String>,
}

/// ディスポジションレスポンス（API-iqc-005）
#[derive(Debug, Serialize, ToSchema)]
pub struct DispositionResponse {
    /// ディスポジション ID
    pub id: Uuid,
    /// 対象 IQC 検査 ID
    pub inspection_id: Uuid,
    /// ディスポジション種別
    pub disposition_type: String,
    /// ディスポジション理由
    pub reason: String,
    /// 登録者 ID（第一承認者）
    pub created_by: Uuid,
    /// 第二承認者 ID
    pub approved_by: Uuid,
    /// 登録日時
    pub created_at: DateTime<Utc>,
}
