// 入荷検査リポジトリ Trait
// IQC（入荷検査）の永続化・検索 Trait。

use async_trait::async_trait;
use uuid::Uuid;

use crate::error::DomainError;
use crate::model::incoming_inspection::{IncomingInspection, IqcStatus};

/// 入荷検査リポジトリ Trait。
#[async_trait]
pub trait IncomingInspectionRepository: Send + Sync + 'static {
    /// 入荷検査を INSERT する。
    async fn insert(&self, inspection: IncomingInspection) -> Result<(), DomainError>;

    /// ID で入荷検査を検索する。
    async fn find_by_id(&self, id: Uuid) -> Result<Option<IncomingInspection>, DomainError>;

    /// 入荷検査のステータスを更新する（検査フロー制御）。
    async fn update_status(
        &self,
        id: Uuid,
        new_status: IqcStatus,
        updated_by: Uuid,
    ) -> Result<IncomingInspection, DomainError>;

    /// ロット ID に紐づく入荷検査一覧を取得する。
    async fn find_by_lot(&self, lot_id: Uuid) -> Result<Vec<IncomingInspection>, DomainError>;
}
