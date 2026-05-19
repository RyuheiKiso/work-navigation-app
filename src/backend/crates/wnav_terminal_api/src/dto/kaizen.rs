// Kaizen 改善提案 API（API-kaizen-001）の DTO 定義（06_アンドン・CAPA・KaizenAPI仕様.md §6）

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

/// Kaizen 改善提案起票リクエスト（API-kaizen-001）
#[derive(Debug, Deserialize, ToSchema)]
pub struct KaizenRequest {
    /// 提案者 ID（TBL-016 に存在すること）
    pub proposer_id: Uuid,
    /// 対象工程 ID（TBL-021 に存在すること、任意）
    pub process_id: Option<Uuid>,
    /// 改善カテゴリ（efficiency / safety / quality / cost / environment）
    pub category: String,
    /// 提案タイトル（1〜200 文字）
    pub title: String,
    /// 現状説明（1〜2000 文字）
    pub current_situation: String,
    /// 改善提案の詳細（1〜5000 文字）
    pub proposal_detail: String,
    /// 期待効果（任意、最大 2000 文字）
    pub expected_benefit: Option<String>,
    /// 関連 SOP ID（TBL-007 に存在すること、任意）
    pub related_sop_id: Option<Uuid>,
    /// 添付エビデンス ID 一覧（最大 10 件、各要素が TBL-009 に存在すること）
    pub evidence_ids: Option<Vec<Uuid>>,
    /// クライアント側の起票時刻
    pub timestamp_client: DateTime<Utc>,
}

/// Kaizen 改善提案起票レスポンスの data フィールド（API-kaizen-001）
#[derive(Debug, Serialize, ToSchema)]
pub struct KaizenData {
    /// 提案 ID（TBL-015）
    pub proposal_id: Uuid,
    /// ステータス（作成直後は常に "submitted"）
    pub status: String,
    pub title: String,
    pub proposer_id: Uuid,
    /// 起票時刻（サーバー側権威タイムスタンプ）
    pub created_at: DateTime<Utc>,
}
