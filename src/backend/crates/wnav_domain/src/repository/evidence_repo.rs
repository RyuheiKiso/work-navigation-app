// 証拠ファイルリポジトリ Trait
// 作業証拠（写真・測定値・QR スキャン等）の永続化 Trait。

use async_trait::async_trait;
use uuid::Uuid;

use crate::error::DomainError;
use crate::model::evidence::EvidenceFile;

/// 証拠ファイルリポジトリ Trait。
#[async_trait]
pub trait EvidenceRepository: Send + Sync + 'static {
    /// 証拠ファイルを INSERT する。
    async fn insert(&self, evidence: EvidenceFile) -> Result<(), DomainError>;

    /// イベント ID に紐づく証拠ファイル一覧を取得する。
    async fn find_by_event(&self, event_id: Uuid) -> Result<Vec<EvidenceFile>, DomainError>;
}
