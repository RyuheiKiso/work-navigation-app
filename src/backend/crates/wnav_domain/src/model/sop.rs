// SOP（標準作業手順書）のドメインモデル
// マスタの公開フローを状態機械として管理し、Draft→UnderReview→Published→Archived の遷移を制御する。

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

/// SOP（標準作業手順書）エンティティ。
/// 多言語テキストは JSONB 形式 `{"ja": "...", "en": "...", "zh": "..."}` で保持する（src/CLAUDE.md 多言語対応）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sop {
    /// SOP ID（UUID v7）
    pub sop_id: Uuid,
    /// 対象工程 ID
    pub operation_id: Uuid,
    /// SOP 名称（JSONB 多言語: {"ja": "...", "en": "...", "zh": "..."}）
    pub name_json: Value,
    /// バージョン文字列（例: "1.0.0"）
    pub version: String,
    /// 公開ステータス
    pub status: SopStatus,
    /// アクティブフラグ（論理削除。物理削除は禁止）
    pub is_active: bool,
    /// 作成日時
    pub created_at: DateTime<Utc>,
    /// 最終更新日時（楽観ロック用）
    pub updated_at: DateTime<Utc>,
}

/// SOP の公開ステータス。
/// 公開フロー: Draft → UnderReview → Published → Archived
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SopStatus {
    /// 下書き（編集可能）
    Draft,
    /// レビュー中（承認待ち）
    UnderReview,
    /// 公開済み（現場使用可能。BR-BUS-012: 電子サイン必須）
    Published,
    /// アーカイブ済み（廃止。参照のみ）
    Archived,
}

impl SopStatus {
    /// SOP ステータスの合法な遷移かどうかを返す。
    /// 公開には電子サインと quality_admin ロールが必要（BR-BUS-012）。
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
