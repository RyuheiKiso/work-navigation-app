// ハッシュチェーンブロックリポジトリ Trait
// SHA-256 ハッシュチェーンブロックの永続化・検証 Trait。
// BAT-001 週次検証ジョブが list_by_case を使用してチェーン連続性を検証する。

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::DomainError;

/// ハッシュチェーンブロックリポジトリ Trait。
/// BAT-001 週次検証ジョブで list_by_case を呼び出してチェーン連続性を検証する。
#[async_trait]
pub trait HashChainBlockRepository: Send + Sync + 'static {
    /// ハッシュチェーンブロックを INSERT する（作業イベントと同一 TX）。
    async fn insert(&self, block: HashChainBlock) -> Result<(), DomainError>;

    /// case_id に紐づく全ブロックを sequence_number 順で取得する（BAT-001 週次検証用）。
    async fn list_by_case(&self, case_id: Uuid) -> Result<Vec<HashChainBlock>, DomainError>;

    /// case_id の最新ブロックを取得する（次ブロック生成時の prev_hash 取得）。
    async fn find_latest(&self, case_id: Uuid) -> Result<Option<HashChainBlock>, DomainError>;
}

/// ハッシュチェーンブロックエンティティ。
/// CompleteStep トランザクション内で work_events と同時に INSERT する（ALG-002）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HashChainBlock {
    /// ブロック ID（UUID v7。work_event の event_id と同一）
    pub block_id: Uuid,
    /// ケース ID
    pub case_id: Uuid,
    /// シーケンス番号（1 基準。連番チェック用）
    pub sequence_number: i64,
    /// 前ブロックのチェーンハッシュ（SHA-256 32 バイト）
    pub prev_block_hash: [u8; 32],
    /// 本ブロックのコンテンツハッシュ（SHA-256 32 バイト）
    pub content_hash: [u8; 32],
    /// チェーンハッシュ（SHA-256(prev_block_hash || content_hash)）
    pub block_hash: [u8; 32],
    /// 作成日時
    pub created_at: DateTime<Utc>,
}
