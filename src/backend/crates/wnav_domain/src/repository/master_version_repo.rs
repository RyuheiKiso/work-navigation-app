// マスタバージョンリポジトリ Trait
// SOP バージョンの CRUD 操作と公開フロー管理のための Trait。

use async_trait::async_trait;
use uuid::Uuid;

use crate::error::DomainError;
use crate::model::master_version::{MasterVersion, MasterVersionStatus};
use crate::model::pagination::{Page, Pagination};

/// マスタバージョンリポジトリ Trait。
#[async_trait]
pub trait MasterVersionRepository: Send + Sync + 'static {
    /// ID でマスタバージョンを検索する。
    async fn find_by_id(&self, id: Uuid) -> Result<Option<MasterVersion>, DomainError>;

    /// 指定 SOP の公開済みバージョンを取得する。
    async fn find_published_by_sop(
        &self,
        sop_id: Uuid,
    ) -> Result<Option<MasterVersion>, DomainError>;

    /// マスタバージョンの一覧を取得する。
    async fn list(
        &self,
        sop_id: Option<Uuid>,
        page: Pagination,
    ) -> Result<Page<MasterVersion>, DomainError>;

    /// 新規マスタバージョンを INSERT する。
    async fn create(&self, cmd: CreateMasterVersionCmd) -> Result<MasterVersion, DomainError>;

    /// マスタバージョンのステータスを更新する（公開フロー制御）。
    async fn update_status(
        &self,
        id: Uuid,
        new_status: MasterVersionStatus,
        approved_by: Option<Uuid>,
    ) -> Result<MasterVersion, DomainError>;
}

/// 新規マスタバージョン作成コマンド。
#[derive(Debug)]
pub struct CreateMasterVersionCmd {
    /// マスタバージョン ID（UUID v7）
    pub master_version_id: Uuid,
    /// SOP ID
    pub sop_id: Uuid,
    /// バージョン番号文字列
    pub version_number: String,
    /// 作成者 ID
    pub created_by: Uuid,
}
