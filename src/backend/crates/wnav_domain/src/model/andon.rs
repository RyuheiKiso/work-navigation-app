// アンドン（異常通知）のドメインモデル
// 作業中の異常・問題発生を即時通知するアンドンイベントを管理する。
// WorkSuspended ドメインイベントとの連動（src/CLAUDE.md 参照）。

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// アンドンイベントエンティティ。
/// 作業中断・異常発生時に記録する。
/// Open → Resolved / Escalated の状態遷移で管理する。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AndonEvent {
    /// アンドン ID（UUID v7）
    pub andon_id: Uuid,
    /// 関連する作業実行 ID
    pub work_execution_id: Uuid,
    /// 発報者 ID
    pub triggered_by: Uuid,
    /// 異常コード（列挙型で管理。自由文字列禁止）
    pub reason_code: String,
    /// 異常詳細テキスト（任意）
    pub reason_text: Option<String>,
    /// アンドンステータス
    pub status: AndonStatus,
    /// 発報日時
    pub created_at: DateTime<Utc>,
}

/// アンドンステータス。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AndonStatus {
    /// 未解決（対応待ち）
    Open,
    /// 解決済み
    Resolved,
    /// エスカレーション済み（CAPA に連携）
    Escalated,
}
