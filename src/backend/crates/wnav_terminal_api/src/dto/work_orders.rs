// 作業指示 API（API-work-orders-001〜002）の DTO 定義（03_作業実行API仕様.md §1〜2）

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

/// 作業指示一覧・単件取得のレスポンス data 要素
#[derive(Debug, Serialize, ToSchema)]
pub struct WorkOrderDto {
    /// 作業指示 ID（TBL-006）
    pub id: Uuid,
    /// 作業指示番号（人間可読）
    pub work_order_number: String,
    /// ステータス（open / in_progress / completed / cancelled）
    pub status: String,
    /// 工程 ID（TBL-021）
    pub process_id: Uuid,
    /// 工程名
    pub process_name: String,
    /// SOP ID（TBL-007）
    pub sop_id: Uuid,
    /// SOP バージョン（semver 形式）
    pub sop_version: String,
    /// ロット ID（TBL-024）
    pub lot_id: Option<Uuid>,
    /// ロット番号（人間可読）
    pub lot_number: Option<String>,
    /// 製品 ID（TBL-023）
    pub product_id: Option<Uuid>,
    /// 製品名
    pub product_name: Option<String>,
    /// 予定開始時刻
    pub scheduled_start: Option<DateTime<Utc>>,
    /// 予定終了時刻
    pub scheduled_end: Option<DateTime<Utc>>,
    /// 担当オペレータ ID
    pub assigned_to: Option<Uuid>,
    /// 作成時刻
    pub created_at: DateTime<Utc>,
    /// 最終更新時刻
    pub updated_at: DateTime<Utc>,
}

/// 作業指示一覧取得のクエリパラメータ（API-work-orders-001）
#[derive(Debug, Deserialize, ToSchema)]
pub struct WorkOrderQuery {
    /// ステータスフィルタ（open / in_progress / completed / cancelled）
    pub status: Option<String>,
    /// 工程 ID フィルタ
    pub process_id: Option<Uuid>,
    /// ロット ID フィルタ
    pub lot_id: Option<Uuid>,
    /// 担当オペレータ ID フィルタ
    pub assigned_to: Option<Uuid>,
    /// 予定日フィルタ（ISO 8601 date 形式）
    pub scheduled_date: Option<NaiveDate>,
    /// ページ番号（1 始まり、デフォルト 1）
    pub page: Option<i64>,
    /// 1 ページあたりの件数（デフォルト 50 / 最大 200）
    pub per_page: Option<i64>,
}

/// 作業指示作成リクエスト（API-work-orders-002）
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateWorkOrderRequest {
    /// 作業指示番号（1〜32 文字、英数字・ハイフン）
    pub work_order_number: String,
    /// 工程 ID（TBL-021 に存在すること）
    pub process_id: Uuid,
    /// SOP ID（TBL-007 に Published バージョンが存在すること）
    pub sop_id: Uuid,
    /// ロット ID（TBL-024 に存在すること）
    pub lot_id: Uuid,
    /// 製品 ID（TBL-023 に存在すること）
    pub product_id: Uuid,
    /// 予定開始時刻（未来時刻）
    pub scheduled_start: DateTime<Utc>,
    /// 予定終了時刻（scheduled_start より後）
    pub scheduled_end: DateTime<Utc>,
    /// 担当オペレータ ID（任意）
    pub assigned_to: Option<Uuid>,
}
