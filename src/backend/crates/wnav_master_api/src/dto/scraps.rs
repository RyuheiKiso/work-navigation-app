// 廃却・返品記録 DTO
//
// POST /api/v1/scrap-records と POST /api/v1/return-records の Request/Response 型。

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

/// 廃却記録リクエスト
#[derive(Debug, Deserialize, ToSchema)]
pub struct ScrapRecordRequest {
    /// 廃棄対象ロット ID
    pub lot_id: String,
    /// 廃棄数量
    pub quantity: i64,
    /// 廃棄理由コード
    pub reason_code: String,
    /// 廃棄理由詳細
    pub description: Option<String>,
    /// 廃棄日時
    pub scrapped_at: DateTime<Utc>,
    /// 廃棄コスト（円）
    pub cost: Option<f64>,
}

/// 廃却記録レスポンス
#[derive(Debug, Serialize, ToSchema)]
pub struct ScrapRecordResponse {
    /// 廃却記録 ID
    pub id: Uuid,
    /// 廃棄対象ロット ID
    pub lot_id: String,
    /// 廃棄数量
    pub quantity: i64,
    /// 廃棄理由コード
    pub reason_code: String,
    /// 登録者 ID
    pub created_by: Uuid,
    /// 登録日時
    pub created_at: DateTime<Utc>,
}

/// 返品記録リクエスト
#[derive(Debug, Deserialize, ToSchema)]
pub struct ReturnRecordRequest {
    /// 返品対象ロット ID
    pub lot_id: String,
    /// サプライヤー ID
    pub supplier_id: String,
    /// 返品数量
    pub quantity: i64,
    /// 返品理由コード
    pub reason_code: String,
    /// 返品理由詳細
    pub description: Option<String>,
    /// 返品日時
    pub returned_at: DateTime<Utc>,
}

/// 返品記録レスポンス
#[derive(Debug, Serialize, ToSchema)]
pub struct ReturnRecordResponse {
    /// 返品記録 ID
    pub id: Uuid,
    /// 返品対象ロット ID
    pub lot_id: String,
    /// サプライヤー ID
    pub supplier_id: String,
    /// 返品数量
    pub quantity: i64,
    /// 返品理由コード
    pub reason_code: String,
    /// 登録者 ID
    pub created_by: Uuid,
    /// 登録日時
    pub created_at: DateTime<Utc>,
}
