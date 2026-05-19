// IQC DTO（terminal-api 担当分: API-iqc-001・API-iqc-003）
//
// 現場端末からの入荷検査開始・測定値追加エンドポイントの Request/Response 型。
// 合否判定（API-iqc-004）・特採承認（API-iqc-005）は master-api の DTO が担当する。

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

/// 入荷検査開始リクエスト（POST /api/v1/iqc/incoming-inspections）
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct CreateInspectionRequest {
    /// 入荷ロット ID（TBL-024）
    pub lot_id: Uuid,
    /// 仕入先 ID
    pub supplier_id: Uuid,
    /// 資材 ID
    pub material_id: Uuid,
    /// ロット数量
    pub lot_quantity: i64,
}

/// 入荷検査開始レスポンス（HTTP 201）
#[derive(Debug, Serialize, ToSchema)]
pub struct InspectionCreatedResponse {
    pub inspection_id: Uuid,
    pub sampling_plan_id: Option<Uuid>,
    /// サンプリングサイズ n
    pub sample_size_n: Option<i32>,
    /// 合格判定数 Ac
    pub accept_number_ac: Option<i32>,
    /// 不合格判定数 Re
    pub reject_number_re: Option<i32>,
    /// 検査厳しさ状態（NORMAL / TIGHTENED / REDUCED）
    pub severity_state: String,
    /// 検査ステータス（PENDING）
    pub qc_status: String,
}

/// 測定値追加リクエスト（POST /api/v1/iqc/incoming-inspections/{id}/measurements）
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct AddMeasurementRequest {
    /// サンプル番号（1 以上）
    pub sample_no: i32,
    /// 測定値
    pub measured_value: f64,
    /// 不良フラグ（true: 不良）
    pub defect_flag: bool,
    /// エビデンスファイル ID（TBL-009、任意）
    pub evidence_file_id: Option<Uuid>,
}

/// 測定値追加レスポンス（HTTP 201）
#[derive(Debug, Serialize, ToSchema)]
pub struct MeasurementResponse {
    pub measurement_id: Uuid,
    pub inspection_id: Uuid,
    pub sample_no: i32,
    pub measured_value: f64,
    pub defect_flag: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}
