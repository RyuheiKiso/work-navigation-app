// リワーク（手直し）のドメインモデル
// 不合格品・不適合品の手直し作業を管理する。
// Two-Person Integrity（FR-AU-007）による 2 名検証を強制する。

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// リワークエンティティ。
/// 不適合品の手直し作業を管理する。
/// Two-Person Integrity（FR-AU-007）で 2 名の検証が必須。
/// ハッシュチェーン（prev_hash/content_hash/chain_hash）で記録の改ざんを検出する。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rework {
    /// リワーク ID（UUID v7）
    pub rework_id: Uuid,
    /// 起因となった不適合 ID
    pub parent_nonconformity_id: Uuid,
    /// リワーク対象ロット ID
    pub lot_id: Uuid,
    /// 使用するリワーク SOP ID
    pub sop_id: Uuid,
    /// リワークステータス
    pub status: ReworkStatus,
    /// 担当者 ID
    pub assignee: Uuid,
    /// 開始日時
    pub started_at: Option<DateTime<Utc>>,
    /// 完了日時
    pub completed_at: Option<DateTime<Utc>>,
    /// 前ブロックのチェーンハッシュ（SHA-256 hex 64 桁）
    pub prev_hash: String,
    /// 本レコードのコンテンツハッシュ（SHA-256 hex 64 桁）
    pub content_hash: String,
    /// チェーンハッシュ（SHA-256(prev_hash || content_hash)）
    pub chain_hash: String,
}

/// リワークステータス。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ReworkStatus {
    /// 待機中
    Pending,
    /// 作業中
    InProgress,
    /// 検証待ち（2 名検証が未完了）
    PendingVerification,
    /// 検証済み（Two-Person Integrity 完了）
    Verified,
    /// クローズ（完了）
    Closed,
}

/// リワーク検証レコード（Two-Person Integrity）。
/// FR-AU-007 に基づき 2 名の独立した承認者による検証を記録する。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReworkVerification {
    /// 検証 ID（UUID v7）
    pub verification_id: Uuid,
    /// リワーク ID
    pub rework_id: Uuid,
    /// 第 1 承認者 ID
    pub verifier_primary: Uuid,
    /// 第 2 承認者 ID（第 1 承認者と異なる人物でなければならない）
    pub verifier_secondary: Uuid,
    /// 検証日時
    pub verified_at: DateTime<Utc>,
    /// 検証コメント
    pub comment: Option<String>,
}
