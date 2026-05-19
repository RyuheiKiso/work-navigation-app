// 入荷検査（IQC）のドメインモデル
// AQL 規格に基づく入荷検査を管理する（FR-IQ-001〜006）。
// ハッシュチェーンで検査記録の改ざんを検出する。

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 入荷検査エンティティ（IQC: Incoming Quality Control）。
/// AQL 規格に基づく入荷検査記録を管理する（FR-IQ-001〜006）。
/// ハッシュチェーン（prev_hash/content_hash/chain_hash）で改ざん検出を保証する。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncomingInspection {
    /// 検査ケース ID（UUID v7）
    pub qc_case_id: Uuid,
    /// 検査対象ロット ID
    pub lot_id: Uuid,
    /// 使用する検査 SOP ID
    pub sop_id: Uuid,
    /// 検査ステータス
    pub status: IqcStatus,
    /// 検査員 ID
    pub inspector_id: Uuid,
    /// 検査開始日時
    pub started_at: Option<DateTime<Utc>>,
    /// 検査完了日時
    pub completed_at: Option<DateTime<Utc>>,
    /// 検査結果（完了後に設定）
    pub result: Option<IqcResult>,
    /// 前ブロックのチェーンハッシュ（SHA-256 hex 64 桁）
    pub prev_hash: String,
    /// 本レコードのコンテンツハッシュ（SHA-256 hex 64 桁）
    pub content_hash: String,
    /// チェーンハッシュ（SHA-256(prev_hash || content_hash)）
    pub chain_hash: String,
}

/// IQC ステータス。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum IqcStatus {
    /// 検査待ち
    Pending,
    /// 検査中
    InProgress,
    /// 検査完了（結果判定待ち）
    Completed,
    /// 承認済み
    Approved,
}

/// IQC 検査結果（AQL 判定区分）。
/// FR-IQ-005: Accept / Concession / Screening / Reject の 4 区分。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum IqcResult {
    /// 合格（全数使用可）
    Accept,
    /// 特採（条件付き使用。偏差承認が必要）
    Concession,
    /// 選別（良品のみ使用）
    Screening,
    /// 不合格（使用不可・返品/廃棄）
    Reject,
}

/// 入荷検査の個別測定値（IQC Measurement）。
/// 複数のステップで測定した値を保持し、AQL 判定に使用する。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncomingInspectionMeasurement {
    /// 測定 ID（UUID v7）
    pub measurement_id: Uuid,
    /// 検査ケース ID
    pub qc_case_id: Uuid,
    /// ステップ ID
    pub step_id: Uuid,
    /// 測定値
    pub value: f64,
    /// 規格下限値
    pub lower_limit: Option<f64>,
    /// 規格上限値
    pub upper_limit: Option<f64>,
    /// 合否（true=合格・false=不合格）
    pub is_pass: bool,
}
