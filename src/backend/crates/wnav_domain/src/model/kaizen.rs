// 改善提案（カイゼン）のドメインモデル
// 現場からの改善提案を管理する。Draft → UnderReview → Approved/Rejected → Implemented のフローで進める。

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 改善提案レポートエンティティ。
/// 現場作業員からの改善提案を管理する。
/// 承認後は SOP や作業環境への変更として実施する。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KaizenReport {
    /// レポート ID（UUID v7）
    pub report_id: Uuid,
    /// 提案者 ID
    pub reporter_id: Uuid,
    /// カテゴリ（安全・品質・効率・環境等）
    pub category: String,
    /// タイトル
    pub title: String,
    /// 説明・提案内容
    pub description: String,
    /// ステータス
    pub status: KaizenStatus,
    /// 影響レベル（High/Medium/Low）
    pub impact_level: String,
}

/// 改善提案のステータス。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum KaizenStatus {
    /// 下書き
    Draft,
    /// レビュー中
    UnderReview,
    /// 承認済み（実施予定）
    Approved,
    /// 却下
    Rejected,
    /// 実施済み
    Implemented,
}
