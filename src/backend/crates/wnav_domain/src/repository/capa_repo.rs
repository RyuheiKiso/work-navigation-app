// CAPA リポジトリ Trait
// CAPA（是正・予防措置）の永続化・検索 Trait。

use async_trait::async_trait;
use uuid::Uuid;

use crate::error::DomainError;
use crate::model::capa::{Capa, CapaPhase};

/// CAPA リポジトリ Trait。
#[async_trait]
pub trait CapaRepository: Send + Sync + 'static {
    /// CAPA を INSERT する。
    async fn insert(&self, capa: Capa) -> Result<(), DomainError>;

    /// ID で CAPA を検索する。
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Capa>, DomainError>;

    /// CAPA のフェーズを更新する（Investigation → Corrective → Preventive → Closed）。
    async fn update_phase(
        &self,
        id: Uuid,
        new_phase: CapaPhase,
        updated_by: Uuid,
    ) -> Result<Capa, DomainError>;
}
