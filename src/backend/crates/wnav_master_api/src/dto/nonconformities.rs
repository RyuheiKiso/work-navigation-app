// 非適合品 DTO（補足仕様）
//
// 非適合品登録エンドポイントの Request/Response 型。
// CAPA との連携に必要。quality_admin / system_admin のみ登録可。

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

/// 非適合品登録リクエスト（POST /api/v1/nonconformities）
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct RegisterNonconformityRequest {
    /// 関連アラート ID（TBL-012、任意）
    pub alert_id: Option<Uuid>,
    /// 関連作業実行 ID（TBL-005、任意）
    pub work_execution_id: Option<Uuid>,
    /// 関連ロット ID（TBL-024、任意）
    pub lot_id: Option<Uuid>,
    /// 非適合種別（process_deviation / material_defect / measurement_out_of_spec / document_error）
    pub nc_type: String,
    /// 非適合内容説明（1〜2000 文字）
    pub description: String,
    /// 発見者 ID（quality_admin / supervisor）
    pub discovered_by: Uuid,
    /// 発見ステップ ID（任意）
    pub discovery_step_id: Option<Uuid>,
    /// 関連エビデンス ID 一覧（各要素が UUID v7・TBL-009 に存在）
    pub evidence_ids: Option<Vec<Uuid>>,
    /// クライアント側の登録時刻
    pub timestamp_client: DateTime<Utc>,
}

/// 非適合品登録レスポンス
#[derive(Debug, Serialize, ToSchema)]
pub struct NonconformityResponse {
    pub nonconformity_id: Uuid,
    pub nc_type: String,
    pub status: String,
    pub alert_id: Option<Uuid>,
    pub lot_id: Option<Uuid>,
    pub discovered_by: Uuid,
    pub created_at: DateTime<Utc>,
}
