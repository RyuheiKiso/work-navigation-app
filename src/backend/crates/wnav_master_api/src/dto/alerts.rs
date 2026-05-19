// アンドン対応 DTO（API-andon-002）
//
// master-api 担当のアラート対応（acknowledge）エンドポイントの Request/Response 型。
// アンドン発報（API-andon-001）は terminal-api の DTO が担当する。

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

/// アンドン対応リクエスト（PATCH /api/v1/alerts/{id}/acknowledge）
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct AcknowledgeAlertRequest {
    /// 確認者 ID（supervisor 以上必須）
    pub acknowledged_by: Uuid,
    /// 確認コメント（最大 500 文字）
    pub acknowledgement_note: Option<String>,
    /// クライアント側の確認時刻
    pub timestamp_client: DateTime<Utc>,
    /// アラートを解決済みにするか否か（true で status を resolved に変更する）
    pub resolved: bool,
}

/// アンドン対応レスポンス
#[derive(Debug, Serialize, ToSchema)]
pub struct AlertAcknowledgedResponse {
    pub alert_id: Uuid,
    pub status: String,
    pub acknowledged_by: Uuid,
    pub acknowledged_at: DateTime<Utc>,
}
