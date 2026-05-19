// 測定値リポジトリ Trait
// 数値測定の永続化 Trait。

use async_trait::async_trait;
use uuid::Uuid;

use crate::error::DomainError;
use crate::model::measurement::Measurement;

/// 測定値リポジトリ Trait。
#[async_trait]
pub trait MeasurementRepository: Send + Sync + 'static {
    /// 測定値を INSERT する。
    async fn insert(&self, measurement: Measurement) -> Result<(), DomainError>;

    /// ステップ ID に紐づく測定値一覧を取得する。
    async fn find_by_step(&self, step_id: Uuid) -> Result<Vec<Measurement>, DomainError>;
}
