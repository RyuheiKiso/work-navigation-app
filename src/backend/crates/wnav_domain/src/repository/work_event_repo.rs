// 作業イベントリポジトリ Trait（Append-only）
// UPDATE・DELETE メソッドを提供しない設計が Append-only 原則を強制する。
// 実装は `crates/wnav_db/` が担う。

use async_trait::async_trait;
use uuid::Uuid;

use crate::error::DomainError;
use crate::model::work_event::WorkEvent;

/// WorkEvent リポジトリ Trait（Append-only）。
/// INSERT のみを提供し、UPDATE・DELETE は提供しない。
/// これにより Append-only 原則（src/CLAUDE.md）をリポジトリ層で強制する。
#[async_trait]
pub trait WorkEventRepository: Send + Sync + 'static {
    /// 単一の WorkEvent を INSERT する。
    /// UPDATE・DELETE は提供しない（Append-only 設計）。
    async fn insert(&self, event: WorkEvent) -> Result<(), DomainError>;

    /// 指定 case_id の最新チェーンハッシュ（chain_hash）を取得する。
    /// genesis（初回）は "0"×64 の文字列を返す。
    async fn latest_hash(&self, case_id: Uuid) -> Result<String, DomainError>;

    /// case_id に紐づく全 WorkEvent を時系列順に取得する。
    async fn list_by_case(&self, case_id: Uuid) -> Result<Vec<WorkEvent>, DomainError>;
}
