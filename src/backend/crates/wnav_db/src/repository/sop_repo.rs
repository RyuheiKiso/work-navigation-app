// PgSopRepository — TBL-007 sops の sqlx 実装
// SOP の CRUD と公開ステータス管理を担う。
// SQLX_PREPARE_REQUIRED: cargo sqlx prepare を実行してからビルド可能

use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use wnav_domain::{
    error::DomainError,
    model::{
        pagination::{Page, Pagination},
        sop::{Sop, SopStatus},
    },
    repository::{CreateSopCmd, SopRepository, UpdateSopCmd},
};

use crate::row_types::SopRow;

/// TBL-007 sops のリポジトリ実装。
pub struct PgSopRepository {
    pool: PgPool,
}

impl PgSopRepository {
    /// コネクションプールを受け取ってリポジトリを構築する。
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

/// SopRow から Sop ドメインモデルへの変換。
impl TryFrom<SopRow> for Sop {
    type Error = DomainError;

    fn try_from(row: SopRow) -> Result<Self, Self::Error> {
        let status = parse_sop_status(&row.status)?;
        Ok(Self {
            sop_id: row.sop_id,
            operation_id: row.operation_id,
            name_json: row.name_json,
            version: row.version,
            status,
            is_active: row.is_active,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

/// DB ステータス文字列を SopStatus 列挙型に変換する。
fn parse_sop_status(s: &str) -> Result<SopStatus, DomainError> {
    match s {
        "DRAFT" => Ok(SopStatus::Draft),
        "UNDER_REVIEW" => Ok(SopStatus::UnderReview),
        "PUBLISHED" => Ok(SopStatus::Published),
        "ARCHIVED" => Ok(SopStatus::Archived),
        other => Err(DomainError::Internal(format!("不明な SopStatus: {other}"))),
    }
}

/// SopStatus を DB 格納文字列に変換する。
fn status_to_str(s: &SopStatus) -> &'static str {
    match s {
        SopStatus::Draft => "DRAFT",
        SopStatus::UnderReview => "UNDER_REVIEW",
        SopStatus::Published => "PUBLISHED",
        SopStatus::Archived => "ARCHIVED",
    }
}

#[async_trait]
impl SopRepository for PgSopRepository {
    /// ID で SOP を検索する。
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Sop>, DomainError> {
        let row = sqlx::query_as::<_, SopRow>(
            r#"
            SELECT
                sop_id, operation_id, name_json, version,
                status, is_active, created_at, updated_at
            FROM sops
            WHERE sop_id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(crate::error::map_sqlx)?;

        row.map(Sop::try_from).transpose()
    }

    /// 工程 ID に紐づく SOP 一覧を取得する（is_active = TRUE のみ）。
    async fn find_by_operation(&self, operation_id: Uuid) -> Result<Vec<Sop>, DomainError> {
        let rows = sqlx::query_as::<_, SopRow>(
            r#"
            SELECT
                sop_id, operation_id, name_json, version,
                status, is_active, created_at, updated_at
            FROM sops
            WHERE operation_id = $1 AND is_active = TRUE
            ORDER BY created_at DESC
            "#,
        )
        .bind(operation_id)
        .fetch_all(&self.pool)
        .await
        .map_err(crate::error::map_sqlx)?;

        rows.into_iter().map(Sop::try_from).collect()
    }

    /// SOP の一覧を取得する（ステータスフィルタと Pagination 対応）。
    async fn list(
        &self,
        status: Option<SopStatus>,
        page: Pagination,
    ) -> Result<Page<Sop>, DomainError> {
        let limit = i64::from(page.per_page);
        let offset = i64::from((page.page - 1) * page.per_page);
        let status_str = status.as_ref().map(|s| status_to_str(s).to_owned());

        let rows = sqlx::query_as::<_, SopRow>(
            r#"
            SELECT
                sop_id, operation_id, name_json, version,
                status, is_active, created_at, updated_at
            FROM sops
            WHERE ($1::text IS NULL OR status = $1)
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(status_str.as_deref())
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(crate::error::map_sqlx)?;

        let total: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM sops WHERE ($1::text IS NULL OR status = $1)",
        )
        .bind(status_str.as_deref())
        .fetch_one(&self.pool)
        .await
        .map_err(crate::error::map_sqlx)?;

        let items = rows
            .into_iter()
            .map(Sop::try_from)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Page {
            items,
            total: u64::try_from(total).unwrap_or(0),
            page: page.page,
            per_page: page.per_page,
        })
    }

    /// 新規 SOP を INSERT する（初期ステータスは Draft）。
    async fn create(&self, cmd: CreateSopCmd) -> Result<Sop, DomainError> {
        let row = sqlx::query_as::<_, SopRow>(
            r#"
            INSERT INTO sops (
                sop_id, operation_id, name_json, version,
                status, is_active, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, 'DRAFT', TRUE, NOW(), NOW())
            RETURNING
                sop_id, operation_id, name_json, version,
                status, is_active, created_at, updated_at
            "#,
        )
        .bind(cmd.sop_id)
        .bind(cmd.operation_id)
        .bind(cmd.name_json)
        .bind(cmd.version)
        .fetch_one(&self.pool)
        .await
        .map_err(crate::error::map_sqlx)?;

        Sop::try_from(row)
    }

    /// SOP を更新する（name_json / status / is_active を個別に更新可能）。
    async fn update(&self, cmd: UpdateSopCmd) -> Result<Sop, DomainError> {
        let status_str = cmd.status.as_ref().map(|s| status_to_str(s).to_owned());

        let row = sqlx::query_as::<_, SopRow>(
            r#"
            UPDATE sops
            SET
                name_json  = COALESCE($1, name_json),
                status     = COALESCE($2, status),
                is_active  = COALESCE($3, is_active),
                updated_at = NOW()
            WHERE sop_id = $4
            RETURNING
                sop_id, operation_id, name_json, version,
                status, is_active, created_at, updated_at
            "#,
        )
        .bind(cmd.name_json)
        .bind(status_str)
        .bind(cmd.is_active)
        .bind(cmd.sop_id)
        .fetch_one(&self.pool)
        .await
        .map_err(crate::error::map_sqlx)?;

        Sop::try_from(row)
    }
}
