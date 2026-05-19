// ロット（資材ロット）のドメインモデル
// 資材・製品のロット管理。親ロット追跡によるトレーサビリティを実現する。
// qc_status で入荷検査（IQC）の結果を反映する。

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// ロットエンティティ。
/// 資材・部品の受入・使用・廃棄を追跡する。
/// parent_lot_id で親ロットへの参照を保持し、トレーサビリティチェーンを構成する。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lot {
    /// ロット ID（UUID v7）
    pub lot_id: Uuid,
    /// 資材 ID
    pub material_id: Uuid,
    /// 仕入先 ID
    pub supplier_id: Option<Uuid>,
    /// 親ロット ID（分割・合流時のトレーサビリティ用）
    pub parent_lot_id: Option<Uuid>,
    /// ロット番号（外部システム識別子）
    pub lot_number: String,
    /// 数量
    pub quantity: f64,
    /// 単位（例: "個", "kg", "L"）
    pub unit: String,
    /// 品質管理ステータス
    pub qc_status: QcStatus,
    /// 受入日時
    pub received_at: DateTime<Utc>,
    /// 使用期限
    pub expiry_at: Option<DateTime<Utc>>,
}

/// ロットの品質管理ステータス。
/// IQC（入荷検査）の結果を反映する（FR-IQ-001〜006）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum QcStatus {
    /// 検査待ち
    Pending,
    /// 合格
    Accepted,
    /// 条件付き合格（特採）
    Concession,
    /// 選別使用
    Screened,
    /// 不合格
    Rejected,
}
