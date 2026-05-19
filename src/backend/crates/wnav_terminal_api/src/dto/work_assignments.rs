// 作業指示 Pull 補完 API（API-sync-005）の DTO 定義（07_作業指示Pull補完API仕様.md）

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

/// 作業割当一覧取得クエリパラメータ（API-sync-005）
#[derive(Debug, Deserialize, ToSchema)]
pub struct WorkAssignmentQuery {
    /// ステータスフィルタ（カンマ区切り、デフォルト "pending,dispatched"）
    pub status: Option<String>,
    /// 最大取得件数（1〜200、デフォルト 50）
    pub limit: Option<i64>,
    /// カーソルページング用 UUID v7（この ID より新しいレコードを返す）
    pub after: Option<Uuid>,
}

/// 作業割当情報（TBL-052 の必要列のみ）
///
/// 機密性の高い内部管理フィールド（external_order_id 等）は含めない
#[derive(Debug, Serialize, ToSchema)]
pub struct WorkAssignmentDto {
    /// 作業割当 ID（TBL-052）
    pub id: Uuid,
    /// SOP ID（TBL-007）
    pub sop_id: Uuid,
    /// SOP 名称（JOIN して取得）
    pub sop_name: String,
    /// ロット ID（TBL-024）
    pub lot_id: Option<Uuid>,
    /// ロット番号（人間可読）
    pub lot_number: Option<String>,
    /// 推奨作業者 ID（TBL-016）
    pub suggested_worker_id: Option<Uuid>,
    /// 推奨設備 ID（TBL-018）
    pub suggested_equipment_id: Option<Uuid>,
    /// 期限日時
    pub due_at: Option<DateTime<Utc>>,
    /// 優先度（1〜5）
    pub priority: i32,
    /// ステータス（pending / dispatched / acknowledged / cancelled）
    pub status: String,
    /// サーバー受信時刻（ソート・カーソル基準）
    pub received_at: DateTime<Utc>,
}

/// 作業割当 ACK リクエスト（ACK エンドポイント用）
///
/// ボディは空オブジェクト {}。割当 ID はパスパラメータから取得する
#[derive(Debug, Deserialize, ToSchema, Default)]
pub struct WorkAssignmentAckRequest {}

/// 作業割当 ACK レスポンスの data フィールド
#[derive(Debug, Serialize, ToSchema)]
pub struct WorkAssignmentAckData {
    pub assignment_id: Uuid,
    pub status: String,
    pub acknowledged_at: DateTime<Utc>,
}
