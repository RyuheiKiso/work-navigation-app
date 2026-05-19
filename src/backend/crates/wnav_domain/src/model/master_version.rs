// SOP バージョン管理エンティティ（EN-006）
// `Published` への遷移には電子サイン + quality_admin ロールが必須（BR-BUS-012）。

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// SOP のバージョン管理エンティティ（EN-006）。
/// `Published` への遷移には電子サイン + quality_admin ロールが必須（BR-BUS-012）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MasterVersion {
    /// マスタバージョン ID（UUID v7）
    pub master_version_id: Uuid,
    /// 対象 SOP ID
    pub sop_id: Uuid,
    /// バージョン番号文字列（例: "1.0.0"）
    pub version_number: String,
    /// ステータス
    pub status: MasterVersionStatus,
    /// 承認者 ID（Published 時に設定）
    pub approved_by: Option<Uuid>,
    /// 承認日時
    pub approved_at: Option<DateTime<Utc>>,
    /// 公開日時
    pub published_at: Option<DateTime<Utc>>,
    /// 作成者 ID
    pub created_by: Uuid,
    /// 作成日時
    pub created_at: DateTime<Utc>,
    /// 最終更新日時（楽観ロック用）
    pub updated_at: DateTime<Utc>,
}

/// マスタバージョンのステータス。
/// 公開フロー: Draft → UnderReview → Published → Archived
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MasterVersionStatus {
    /// 下書き
    Draft,
    /// レビュー中
    UnderReview,
    /// 公開済み（現場使用可能）
    Published,
    /// アーカイブ済み
    Archived,
}

impl MasterVersionStatus {
    /// マスタバージョンステータスの合法な遷移かどうかを返す。
    pub fn can_transition_to(&self, next: &Self) -> bool {
        matches!(
            (self, next),
            (Self::Draft, Self::UnderReview)
                | (Self::UnderReview, Self::Draft)
                | (Self::UnderReview, Self::Published)
                | (Self::Published, Self::Archived)
        )
    }
}
