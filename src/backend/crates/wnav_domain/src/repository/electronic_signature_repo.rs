// 電子サインリポジトリ Trait
// 電子サインの永続化・検索 Trait。

use async_trait::async_trait;
use uuid::Uuid;

use crate::error::DomainError;
use crate::model::electronic_signature::ElectronicSignature;

/// 電子サインリポジトリ Trait。
#[async_trait]
pub trait ElectronicSignatureRepository: Send + Sync + 'static {
    /// 電子サインを INSERT する。
    async fn insert(&self, signature: ElectronicSignature) -> Result<(), DomainError>;

    /// ID で電子サインを検索する。
    async fn find_by_id(&self, id: Uuid) -> Result<Option<ElectronicSignature>, DomainError>;

    /// イベント ID に紐づく電子サイン一覧を取得する。
    async fn find_by_event(&self, event_id: Uuid) -> Result<Vec<ElectronicSignature>, DomainError>;
}
