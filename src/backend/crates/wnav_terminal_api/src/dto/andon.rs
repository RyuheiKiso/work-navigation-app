// アンドン API（API-andon-001）の DTO 定義（06_アンドン・CAPA・KaizenAPI仕様.md §1）

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

/// アンドン発報リクエスト（API-andon-001）
#[derive(Debug, Deserialize, ToSchema)]
pub struct AndonRequest {
    /// アラート種別（quality / safety / equipment / process）
    pub alert_type: String,
    /// 重大度（low / medium / high / critical）
    pub severity: String,
    /// 関連する作業実行 ID（任意）
    pub work_execution_id: Option<Uuid>,
    /// 関連するステップ ID（任意）
    pub step_id: Option<Uuid>,
    /// 起票者 ID（TBL-016 に存在すること）
    pub raised_by: Uuid,
    /// アラートタイトル（1〜200 文字）
    pub title: String,
    /// 詳細説明（1〜2000 文字）
    pub description: String,
    /// クライアント側の発生時刻
    pub timestamp_client: DateTime<Utc>,
}

/// アンドン発報レスポンスの data フィールド（API-andon-001）
#[derive(Debug, Serialize, ToSchema)]
pub struct AndonData {
    /// アラート ID（TBL-012）
    pub alert_id: Uuid,
    pub alert_type: String,
    pub severity: String,
    /// ステータス（作成直後は常に "open"）
    pub status: String,
    pub work_execution_id: Option<Uuid>,
    pub raised_by: Uuid,
    pub title: String,
    /// 発報時刻（サーバー側権威タイムスタンプ）
    pub raised_at: DateTime<Utc>,
    /// supervisor への通知送信結果
    pub notification_sent: bool,
}
