// リワークリポジトリ Trait
// リワーク（手直し）の永続化・検索 Trait。

use async_trait::async_trait;
use uuid::Uuid;

use crate::error::DomainError;
use crate::model::rework::{Rework, ReworkStatus};

/// リワークリポジトリ Trait。
#[async_trait]
pub trait ReworkRepository: Send + Sync + 'static {
    /// リワークを INSERT する。
    async fn insert(&self, rework: Rework) -> Result<(), DomainError>;

    /// ID でリワークを検索する。
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Rework>, DomainError>;

    /// リワークのステータスを更新する（リワークフロー制御）。
    async fn update_status(
        &self,
        id: Uuid,
        new_status: ReworkStatus,
        updated_by: Uuid,
    ) -> Result<Rework, DomainError>;

    /// 不適合 ID に紐づくリワーク一覧を取得する。
    async fn find_by_nonconformity(
        &self,
        nonconformity_id: Uuid,
    ) -> Result<Vec<Rework>, DomainError>;
}
