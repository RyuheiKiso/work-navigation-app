// アンドンリポジトリ Trait
// アンドンイベントの永続化・検索 Trait。

use async_trait::async_trait;
use uuid::Uuid;

use crate::error::DomainError;
use crate::model::andon::AndonEvent;

/// アンドンリポジトリ Trait。
#[async_trait]
pub trait AndonRepository: Send + Sync + 'static {
    /// アンドンイベントを INSERT する。
    async fn insert(&self, event: AndonEvent) -> Result<(), DomainError>;

    /// アクティブ（未解決）なアンドンイベント一覧を取得する。
    async fn list_active(&self) -> Result<Vec<AndonEvent>, DomainError>;

    /// アンドンを解決済みに更新する（監督者操作）。
    async fn resolve(&self, andon_id: Uuid, resolved_by: Uuid) -> Result<(), DomainError>;
}
