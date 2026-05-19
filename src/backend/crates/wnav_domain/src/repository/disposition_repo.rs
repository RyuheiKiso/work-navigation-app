// 処置判定リポジトリ Trait
// 処置判定（Disposition）の永続化・検索 Trait。
// Two-Person Integrity（FR-AU-007）で 2 名の承認者が必要。

use async_trait::async_trait;
use uuid::Uuid;

use crate::error::DomainError;
use crate::model::disposition::{Disposition, DispositionApproval};

/// 処置判定リポジトリ Trait。
#[async_trait]
pub trait DispositionRepository: Send + Sync + 'static {
    /// 処置判定を INSERT する。
    async fn insert(&self, disposition: Disposition) -> Result<(), DomainError>;

    /// ID で処置判定を検索する。
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Disposition>, DomainError>;

    /// 処置判定に承認を追加する（Two-Person Integrity: 1 人目または 2 人目の承認）。
    async fn add_approval(&self, approval: DispositionApproval) -> Result<(), DomainError>;

    /// リワーク ID に紐づく処置判定一覧を取得する。
    async fn find_by_rework(&self, rework_id: Uuid) -> Result<Vec<Disposition>, DomainError>;
}
