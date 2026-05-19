// Outbox リポジトリ Trait
// Transactional Outbox パターンの永続化 Trait。
// INSERT はドメインサービスが呼び出し、配信操作は Outbox Worker が行う。

use async_trait::async_trait;
use uuid::Uuid;

use crate::error::DomainError;
use crate::model::outbox::OutboxEvent;

/// Outbox リポジトリ Trait。
/// INSERT はドメインサービスが、配信操作は Outbox Worker（wnav_outbox クレート）が担う。
#[async_trait]
pub trait OutboxRepository: Send + Sync + 'static {
    /// Outbox イベントを INSERT する（作業イベントと同一 TX で呼び出す）。
    async fn insert(&self, event: OutboxEvent) -> Result<(), DomainError>;

    /// 配信待ちの Outbox イベントを取得する（Outbox Worker が呼び出す）。
    async fn list_pending(&self, limit: u32) -> Result<Vec<OutboxEvent>, DomainError>;

    /// 配信成功を記録する。
    async fn mark_sent(&self, outbox_id: Uuid) -> Result<(), DomainError>;

    /// 配信失敗を記録する（リトライカウントを増やす）。
    async fn mark_failed(&self, outbox_id: Uuid) -> Result<(), DomainError>;

    /// 最大リトライ超過のイベントをデッドレターキューに移動する。
    async fn move_to_dlq(&self, outbox_id: Uuid) -> Result<(), DomainError>;
}
