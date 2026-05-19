// 作業指示のドメインモデル
// 外部システムから受信する作業指示を管理する。
// 冪等性キーで同一指示の重複受信を防止する。

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 作業指示エンティティ。
/// 外部システム（生産管理・MES 等）から Push/Pull で受信する作業指示を管理する。
/// idempotency_key で同一指示の重複受信を防止する（Idempotent API 原則）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkAssignment {
    /// 作業指示 ID（UUID v7）
    pub assignment_id: Uuid,
    /// 対象 SOP ID
    pub sop_id: Uuid,
    /// ケース ID（作業実行の識別子）
    pub case_id: Uuid,
    /// 対象ロット ID（任意）
    pub lot_id: Option<Uuid>,
    /// 優先度（数値が小さいほど優先度が高い）
    pub priority: i32,
    /// 指示ステータス
    pub status: AssignmentStatus,
    /// 配信先端末 ID（特定端末への指示の場合）
    pub target_terminal_id: Option<Uuid>,
    /// 送信元外部システム名
    pub external_system: Option<String>,
    /// 冪等性キー（外部システムが付与する一意キー）
    pub idempotency_key: Uuid,
    /// 受信日時
    pub received_at: DateTime<Utc>,
}

/// 作業指示のステータス。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AssignmentStatus {
    /// 受信済み・未配信
    Pending,
    /// 端末に配信済み
    Dispatched,
    /// 端末が受理
    Accepted,
    /// 作業中
    Inprogress,
    /// 完了
    Completed,
    /// キャンセル
    Cancelled,
}
