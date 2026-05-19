// ステップリポジトリ Trait
// SOP ステップの CRUD 操作のための Trait。

use async_trait::async_trait;
use uuid::Uuid;

use crate::error::DomainError;
use crate::model::step::Step;

/// ステップリポジトリ Trait。
#[async_trait]
pub trait StepRepository: Send + Sync + 'static {
    /// SOP ID に紐づく全ステップを step_number 順で取得する。
    async fn find_by_sop(&self, sop_id: Uuid) -> Result<Vec<Step>, DomainError>;

    /// ID でステップを検索する。
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Step>, DomainError>;

    /// 複数ステップを一括 INSERT する（SOP 保存時のバッチ操作）。
    async fn create_batch(&self, steps: Vec<Step>) -> Result<Vec<Step>, DomainError>;

    /// ステップの表示順（step_number）を一括更新する（ドラッグ&ドロップ並び替え）。
    async fn reorder(&self, sop_id: Uuid, ordered_ids: Vec<Uuid>) -> Result<(), DomainError>;
}
