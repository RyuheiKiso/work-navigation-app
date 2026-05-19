// 改善提案リポジトリ Trait
// カイゼンレポートの永続化・検索 Trait。

use async_trait::async_trait;
use uuid::Uuid;

use crate::error::DomainError;
use crate::model::kaizen::{KaizenReport, KaizenStatus};

/// 改善提案リポジトリ Trait。
#[async_trait]
pub trait KaizenRepository: Send + Sync + 'static {
    /// 改善提案を INSERT する。
    async fn insert(&self, report: KaizenReport) -> Result<(), DomainError>;

    /// ID で改善提案を検索する。
    async fn find_by_id(&self, id: Uuid) -> Result<Option<KaizenReport>, DomainError>;

    /// 改善提案のステータスを更新する。
    async fn update_status(
        &self,
        id: Uuid,
        new_status: KaizenStatus,
        updated_by: Uuid,
    ) -> Result<KaizenReport, DomainError>;
}
