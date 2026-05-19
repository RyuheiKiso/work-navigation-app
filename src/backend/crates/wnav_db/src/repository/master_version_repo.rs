// PgMasterVersionRepository — TBL-037 master_versions の sqlx 実装
// SOP バージョン管理と公開フロー（Draft→UnderReview→Published→Archived）を担う。
// SQLX_PREPARE_REQUIRED: cargo sqlx prepare を実行してからビルド可能

use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use wnav_domain::{
    error::DomainError,
    model::{
        master_version::{MasterVersion, MasterVersionStatus},
        pagination::{Page, Pagination},
    },
    repository::{CreateMasterVersionCmd, MasterVersionRepository},
};

use crate::row_types::MasterVersionRow;

/// TBL-037 master_versions のリポジトリ実装。
pub struct PgMasterVersionRepository {
    pool: PgPool,
}

impl PgMasterVersionRepository {
    /// コネクションプールを受け取ってリポジトリを構築する。
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

/// MasterVersionRow から MasterVersion ドメインモデルへの変換。
impl TryFrom<MasterVersionRow> for MasterVersion {
    type Error = DomainError;

    fn try_from(row: MasterVersionRow) -> Result<Self, Self::Error> {
        let status = parse_master_version_status(&row.status)?;
        Ok(Self {
            master_version_id: row.master_version_id,
            sop_id: row.sop_id,
            version_number: row.version_number,
            status,
            approved_by: row.approved_by,
            approved_at: row.approved_at,
            published_at: row.published_at,
            created_by: row.created_by,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

/// DB ステータス文字列を MasterVersionStatus に変換する。
fn parse_master_version_status(s: &str) -> Result<MasterVersionStatus, DomainError> {
    match s {
        "DRAFT" => Ok(MasterVersionStatus::Draft),
        "UNDER_REVIEW" => Ok(MasterVersionStatus::UnderReview),
        "PUBLISHED" => Ok(MasterVersionStatus::Published),
        "ARCHIVED" => Ok(MasterVersionStatus::Archived),
        other => Err(DomainError::Internal(format!(
            "不明な MasterVersionStatus: {other}"
        ))),
    }
}

/// MasterVersionStatus を DB 格納文字列に変換する。
fn status_to_str(s: &MasterVersionStatus) -> &'static str {
    match s {
        MasterVersionStatus::Draft => "DRAFT",
        MasterVersionStatus::UnderReview => "UNDER_REVIEW",
        MasterVersionStatus::Published => "PUBLISHED",
        MasterVersionStatus::Archived => "ARCHIVED",
    }
}

#[async_trait]
impl MasterVersionRepository for PgMasterVersionRepository {
    /// ID でマスタバージョンを検索する。
    async fn find_by_id(&self, id: Uuid) -> Result<Option<MasterVersion>, DomainError> {
        let row = sqlx::query_as::<_, MasterVersionRow>(
            r#"
            SELECT
                master_version_id, sop_id, version_number, status,
                approved_by, approved_at, published_at,
                created_by, created_at, updated_at
            FROM master_versions
            WHERE master_version_id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(crate::error::map_sqlx)?;

        row.map(MasterVersion::try_from).transpose()
    }

    /// 指定 SOP の公開済みバージョンを取得する（最新の PUBLISHED 版のみ）。
    async fn find_published_by_sop(
        &self,
        sop_id: Uuid,
    ) -> Result<Option<MasterVersion>, DomainError> {
        let row = sqlx::query_as::<_, MasterVersionRow>(
            r#"
            SELECT
                master_version_id, sop_id, version_number, status,
                approved_by, approved_at, published_at,
                created_by, created_at, updated_at
            FROM master_versions
            WHERE sop_id = $1 AND status = 'PUBLISHED'
            ORDER BY published_at DESC
            LIMIT 1
            "#,
        )
        .bind(sop_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(crate::error::map_sqlx)?;

        row.map(MasterVersion::try_from).transpose()
    }

    /// マスタバージョンの一覧を取得する（SOP ID でフィルタ可）。
    async fn list(
        &self,
        sop_id: Option<Uuid>,
        page: Pagination,
    ) -> Result<Page<MasterVersion>, DomainError> {
        let limit = i64::from(page.per_page);
        let offset = i64::from((page.page - 1) * page.per_page);

        let rows = sqlx::query_as::<_, MasterVersionRow>(
            r#"
            SELECT
                master_version_id, sop_id, version_number, status,
                approved_by, approved_at, published_at,
                created_by, created_at, updated_at
            FROM master_versions
            WHERE ($1::uuid IS NULL OR sop_id = $1)
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(sop_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(crate::error::map_sqlx)?;

        let total: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM master_versions WHERE ($1::uuid IS NULL OR sop_id = $1)",
        )
        .bind(sop_id)
        .fetch_one(&self.pool)
        .await
        .map_err(crate::error::map_sqlx)?;

        let items = rows
            .into_iter()
            .map(MasterVersion::try_from)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Page {
            items,
            total: u64::try_from(total).unwrap_or(0),
            page: page.page,
            per_page: page.per_page,
        })
    }

    /// 新規マスタバージョンを INSERT する（初期ステータスは Draft）。
    async fn create(&self, cmd: CreateMasterVersionCmd) -> Result<MasterVersion, DomainError> {
        let row = sqlx::query_as::<_, MasterVersionRow>(
            r#"
            INSERT INTO master_versions (
                master_version_id, sop_id, version_number, status,
                created_by, created_at, updated_at
            )
            VALUES ($1, $2, $3, 'DRAFT', $4, NOW(), NOW())
            RETURNING
                master_version_id, sop_id, version_number, status,
                approved_by, approved_at, published_at,
                created_by, created_at, updated_at
            "#,
        )
        .bind(cmd.master_version_id)
        .bind(cmd.sop_id)
        .bind(cmd.version_number)
        .bind(cmd.created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(crate::error::map_sqlx)?;

        MasterVersion::try_from(row)
    }

    /// マスタバージョンのステータスを更新する（公開フロー制御）。
    /// Published への遷移時は approved_by と published_at も設定する。
    async fn update_status(
        &self,
        id: Uuid,
        new_status: MasterVersionStatus,
        approved_by: Option<Uuid>,
    ) -> Result<MasterVersion, DomainError> {
        let row = sqlx::query_as::<_, MasterVersionRow>(
            r#"
            UPDATE master_versions
            SET
                status = $1,
                approved_by = CASE WHEN $1 = 'PUBLISHED' THEN $2 ELSE approved_by END,
                approved_at = CASE WHEN $1 = 'PUBLISHED' THEN NOW() ELSE approved_at END,
                published_at = CASE WHEN $1 = 'PUBLISHED' THEN NOW() ELSE published_at END,
                updated_at = NOW()
            WHERE master_version_id = $3
            RETURNING
                master_version_id, sop_id, version_number, status,
                approved_by, approved_at, published_at,
                created_by, created_at, updated_at
            "#,
        )
        .bind(status_to_str(&new_status))
        .bind(approved_by)
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(crate::error::map_sqlx)?;

        MasterVersion::try_from(row)
    }
}
