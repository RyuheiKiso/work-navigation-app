// SOP リポジトリ Trait
// SOP の CRUD 操作のための Trait。

use async_trait::async_trait;
use serde_json::Value;
use uuid::Uuid;

use crate::error::DomainError;
use crate::model::pagination::{Page, Pagination};
use crate::model::sop::{Sop, SopStatus};

/// SOP リポジトリ Trait。
#[async_trait]
pub trait SopRepository: Send + Sync + 'static {
    /// ID で SOP を検索する。
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Sop>, DomainError>;

    /// 工程 ID に紐づく SOP 一覧を取得する。
    async fn find_by_operation(&self, operation_id: Uuid) -> Result<Vec<Sop>, DomainError>;

    /// SOP の一覧を取得する。
    async fn list(
        &self,
        status: Option<SopStatus>,
        page: Pagination,
    ) -> Result<Page<Sop>, DomainError>;

    /// 新規 SOP を INSERT する。
    async fn create(&self, cmd: CreateSopCmd) -> Result<Sop, DomainError>;

    /// SOP を更新する。
    async fn update(&self, cmd: UpdateSopCmd) -> Result<Sop, DomainError>;
}

/// 新規 SOP 作成コマンド。
#[derive(Debug)]
pub struct CreateSopCmd {
    /// SOP ID（UUID v7）
    pub sop_id: Uuid,
    /// 対象工程 ID
    pub operation_id: Uuid,
    /// SOP 名称（JSONB 多言語）
    pub name_json: Value,
    /// バージョン文字列
    pub version: String,
}

/// SOP 更新コマンド。
#[derive(Debug)]
pub struct UpdateSopCmd {
    /// 更新対象 SOP ID
    pub sop_id: Uuid,
    /// 新しい SOP 名称（JSONB 多言語）
    pub name_json: Option<Value>,
    /// 新しいステータス
    pub status: Option<SopStatus>,
    /// アクティブフラグ
    pub is_active: Option<bool>,
}
