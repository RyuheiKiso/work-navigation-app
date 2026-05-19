// リワーク DTO（terminal-api 担当分: API-reworks-001・API-rework-verifications-001）
//
// 現場端末からのリワーク作業開始・再検査記録エンドポイントの Request/Response 型。
// Two-Person Integrity: 作業者 ≠ 検証者（BR-BUS-042）。

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

/// リワーク作業開始リクエスト（POST /api/v1/reworks）
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct CreateReworkRequest {
    /// ディスポジション ID（REWORK 判定を受けた nonconformity の disposition）
    pub disposition_id: Uuid,
    /// リワーク作業者 ID
    pub operator_id: Uuid,
    /// リワーク指示内容（最大 2000 文字）
    pub instruction: Option<String>,
    /// クライアント側の開始時刻
    pub timestamp_client: chrono::DateTime<chrono::Utc>,
}

/// リワークレスポンス（HTTP 201）
#[derive(Debug, Serialize, ToSchema)]
pub struct ReworkResponse {
    pub rework_id: Uuid,
    pub disposition_id: Uuid,
    pub operator_id: Uuid,
    pub status: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// リワーク再検査リクエスト（POST /api/v1/rework-verifications）
///
/// Two-Person Integrity 必須（BR-BUS-042）: 再検査者はリワーク実施者と異なること。
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct CreateReworkVerificationRequest {
    /// 対象リワーク ID
    pub rework_id: Uuid,
    /// 再検査者 ID（リワーク実施者と異なること）
    pub verifier_id: Uuid,
    /// 検査合否（true: 合格・false: 不合格）
    pub passed: bool,
    /// 検査コメント（最大 1000 文字）
    pub comment: Option<String>,
    /// クライアント側の検査時刻
    pub timestamp_client: chrono::DateTime<chrono::Utc>,
}

/// リワーク再検査レスポンス（HTTP 201）
#[derive(Debug, Serialize, ToSchema)]
pub struct VerificationResponse {
    pub verification_id: Uuid,
    pub rework_id: Uuid,
    pub verifier_id: Uuid,
    pub passed: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// 非適合品登録リクエスト（POST /api/v1/nonconformities）
/// terminal-api からの起票版（ドキュメントの補足仕様に準拠）
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct RegisterNonconformityRequest {
    /// 関連アラート ID（任意）
    pub alert_id: Option<Uuid>,
    /// 関連作業実行 ID（任意）
    pub work_execution_id: Option<Uuid>,
    /// 関連ロット ID（任意）
    pub lot_id: Option<Uuid>,
    /// 非適合種別（process_deviation / material_defect / measurement_out_of_spec / document_error）
    pub nc_type: String,
    /// 非適合内容説明（1〜2000 文字）
    pub description: String,
    /// 発見者 ID
    pub discovered_by: Uuid,
    /// 関連エビデンス ID 一覧
    pub evidence_ids: Option<Vec<Uuid>>,
    /// クライアント側の登録時刻
    pub timestamp_client: chrono::DateTime<chrono::Utc>,
}

/// 非適合品登録レスポンス
#[derive(Debug, Serialize, ToSchema)]
pub struct NonconformityResponse {
    pub nonconformity_id: Uuid,
    pub nc_type: String,
    pub status: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}
