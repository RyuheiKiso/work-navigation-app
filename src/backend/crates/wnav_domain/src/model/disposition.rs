// 処置判定（Disposition）のドメインモデル
// 不適合品の処置判定を管理する。
// Two-Person Integrity（FR-AU-007）で 2 名の承認者による判定を強制する。

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 処置判定エンティティ。
/// 不適合品（IQC 不合格・リワーク対象等）の処置を決定する。
/// Two-Person Integrity（FR-AU-007）で 2 名の承認者が必須。
/// ハッシュチェーンで記録の改ざんを検出する。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Disposition {
    /// 処置判定 ID（UUID v7）
    pub disposition_id: Uuid,
    /// 対象ロット ID
    pub lot_id: Uuid,
    /// 関連リワーク ID（リワーク起因の場合）
    pub rework_id: Option<Uuid>,
    /// 処置種別
    pub disposition_type: DispositionType,
    /// 第 1 承認者 ID
    pub approved_by_1: Option<Uuid>,
    /// 第 2 承認者 ID（第 1 承認者と異なる人物でなければならない）
    pub approved_by_2: Option<Uuid>,
    /// 承認日時（両者の承認完了後に設定）
    pub approved_at: Option<DateTime<Utc>>,
    /// 前ブロックのチェーンハッシュ（SHA-256 hex 64 桁）
    pub prev_hash: String,
    /// 本レコードのコンテンツハッシュ（SHA-256 hex 64 桁）
    pub content_hash: String,
    /// チェーンハッシュ（SHA-256(prev_hash || content_hash)）
    pub chain_hash: String,
}

/// 処置種別。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DispositionType {
    /// 廃棄
    Scrap,
    /// 返品
    Return,
    /// 特採（条件付き合格）
    Concession,
    /// 手直し（リワーク）
    Rework,
    /// 合格（使用可）
    Accept,
}

/// 処置判定の承認レコード（Two-Person Integrity）。
/// FR-AU-007 に基づき 2 名の独立した承認者による承認を記録する。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DispositionApproval {
    /// 承認 ID（UUID v7）
    pub approval_id: Uuid,
    /// 対象処置判定 ID
    pub disposition_id: Uuid,
    /// 承認者 ID
    pub approver_id: Uuid,
    /// 承認シーケンス（1=第 1 承認者, 2=第 2 承認者）
    pub sequence: u8,
    /// 承認日時
    pub approved_at: DateTime<Utc>,
    /// 承認コメント
    pub comment: Option<String>,
}
