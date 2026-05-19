// リワーク・廃棄・返品 DTO（API-reworks-001 / API-rework-verifications-001）
//
// リワーク登録・リワーク検証の Request/Response 型。
// Two-Person Integrity（FR-AU-007）を検証登録で実施する。

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

/// リワーク登録リクエスト（API-reworks-001）
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateReworkRequest {
    /// 対象 Case ID
    pub case_id: Uuid,
    /// リワーク指示内容
    pub instruction: String,
    /// リワーク種別（例: "repair", "reprocess", "reinspect"）
    pub rework_type: String,
    /// リワーク理由コード
    pub reason_code: String,
    /// 詳細説明
    pub description: Option<String>,
    /// 計画工数（分）
    pub planned_hours: Option<f64>,
}

/// リワークレスポンス（API-reworks-001）
#[derive(Debug, Serialize, ToSchema)]
pub struct ReworkResponse {
    /// リワーク ID
    pub id: Uuid,
    /// 対象 Case ID
    pub case_id: Uuid,
    /// リワーク指示内容
    pub instruction: String,
    /// リワーク種別
    pub rework_type: String,
    /// 理由コード
    pub reason_code: String,
    /// ステータス（"pending", "in_progress", "completed", "verified"）
    pub status: String,
    /// 登録者 ID
    pub created_by: Uuid,
    /// 登録日時
    pub created_at: DateTime<Utc>,
}

/// リワーク検証リクエスト（API-rework-verifications-001）
///
/// Two-Person Integrity 必須（FR-AU-007）。
/// 検証者（current_user）とリワーク実施者が異なるユーザーである必要がある。
#[derive(Debug, Deserialize, ToSchema)]
pub struct ReworkVerificationRequest {
    /// 対象リワーク ID
    pub rework_id: Uuid,
    /// 検証結果（true: 合格、false: 不合格）
    pub is_passed: bool,
    /// 検証コメント
    pub comment: Option<String>,
    /// 第二検証者 ID（Two-Person Integrity: 登録者と異なるユーザー）
    pub second_verifier_id: Uuid,
}

/// リワーク検証レスポンス（API-rework-verifications-001）
#[derive(Debug, Serialize, ToSchema)]
pub struct ReworkVerificationResponse {
    /// 検証 ID
    pub id: Uuid,
    /// 対象リワーク ID
    pub rework_id: Uuid,
    /// 検証結果
    pub is_passed: bool,
    /// 第一検証者 ID
    pub verified_by: Uuid,
    /// 第二検証者 ID
    pub second_verifier_id: Uuid,
    /// 検証日時
    pub verified_at: DateTime<Utc>,
}
