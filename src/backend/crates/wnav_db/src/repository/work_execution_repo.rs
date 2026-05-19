// PgWorkExecutionRepository — TBL-005 work_executions の sqlx 実装
// 楽観ロック付きステータス更新（update_status_if_unchanged）を含む。
// SQLX_PREPARE_REQUIRED: cargo sqlx prepare を実行してからビルド可能
//
// NOTE: sqlx::query!() マクロへの切り替えは `cargo sqlx prepare --database-url $DATABASE_URL` 実行後に行うこと。
// 現在は SQLX_OFFLINE=true 環境での動作のため sqlx::query_as() / sqlx::query() / sqlx::query_scalar() を使用している。

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use wnav_domain::{
    error::DomainError,
    model::{
        pagination::{Page, Pagination},
        work_execution::{WorkExecution, WorkExecutionStatus},
    },
    repository::{CreateWorkExecutionCmd, WorkExecutionFilter, WorkExecutionRepository},
};

use crate::row_types::WorkExecutionRow;

/// TBL-005 work_executions のリポジトリ実装。
pub struct PgWorkExecutionRepository {
    pool: PgPool,
}

impl PgWorkExecutionRepository {
    /// コネクションプールを受け取ってリポジトリを構築する。
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

/// WorkExecutionRow から WorkExecution ドメインモデルへの変換。
/// DB のステータス文字列を列挙型に変換してマッピングする。
impl TryFrom<WorkExecutionRow> for WorkExecution {
    type Error = DomainError;

    fn try_from(row: WorkExecutionRow) -> Result<Self, Self::Error> {
        let status = parse_work_execution_status(&row.status)?;
        Ok(Self {
            work_execution_id: row.work_execution_id,
            sop_version_id: row.sop_version_id,
            primary_worker_id: row.primary_worker_id,
            secondary_worker_id: row.secondary_worker_id,
            terminal_id: row.terminal_id,
            production_target_id: row.production_target_id,
            status,
            current_step_index: u32::try_from(row.current_step_index).unwrap_or(0),
            started_at: row.started_at,
            completed_at: row.completed_at,
            updated_at: row.updated_at,
        })
    }
}

/// DB ステータス文字列を WorkExecutionStatus に変換する。
fn parse_work_execution_status(s: &str) -> Result<WorkExecutionStatus, DomainError> {
    match s {
        "NOT_STARTED" => Ok(WorkExecutionStatus::NotStarted),
        "IN_PROGRESS" => Ok(WorkExecutionStatus::InProgress),
        "SUSPENDED" => Ok(WorkExecutionStatus::Suspended),
        "COMPLETED" => Ok(WorkExecutionStatus::Completed),
        "CANCELLED" => Ok(WorkExecutionStatus::Cancelled),
        other => Err(DomainError::Internal(format!(
            "不明な WorkExecutionStatus: {other}"
        ))),
    }
}

/// WorkExecutionStatus を DB 格納文字列に変換する。
fn status_to_str(s: &WorkExecutionStatus) -> &'static str {
    match s {
        WorkExecutionStatus::NotStarted => "NOT_STARTED",
        WorkExecutionStatus::InProgress => "IN_PROGRESS",
        WorkExecutionStatus::Suspended => "SUSPENDED",
        WorkExecutionStatus::Completed => "COMPLETED",
        WorkExecutionStatus::Cancelled => "CANCELLED",
    }
}

#[async_trait]
impl WorkExecutionRepository for PgWorkExecutionRepository {
    /// (FNC-BE-006) work_executions テーブルから ID 検索する。
    async fn find_by_id(&self, id: Uuid) -> Result<Option<WorkExecution>, DomainError> {
        let row = sqlx::query_as::<_, WorkExecutionRow>(
            r#"
            SELECT
                work_execution_id,
                sop_version_id,
                primary_worker_id,
                secondary_worker_id,
                terminal_id,
                production_target_id,
                status,
                current_step_index,
                started_at,
                completed_at,
                updated_at
            FROM work_executions
            WHERE work_execution_id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(crate::error::map_sqlx)?;

        row.map(WorkExecution::try_from).transpose()
    }

    /// フィルタ条件と Pagination で作業実行一覧を取得する。
    async fn list(
        &self,
        filter: WorkExecutionFilter,
        page: Pagination,
    ) -> Result<Page<WorkExecution>, DomainError> {
        // per_page と page から LIMIT / OFFSET を算出する
        let limit = i64::from(page.per_page);
        let offset = i64::from((page.page - 1) * page.per_page);

        let status_str = filter.status.as_ref().map(|s| status_to_str(s).to_owned());

        let rows = sqlx::query_as::<_, WorkExecutionRow>(
            r#"
            SELECT
                work_execution_id,
                sop_version_id,
                primary_worker_id,
                secondary_worker_id,
                terminal_id,
                production_target_id,
                status,
                current_step_index,
                started_at,
                completed_at,
                updated_at
            FROM work_executions
            WHERE
                ($1::uuid IS NULL OR primary_worker_id = $1)
                AND ($2::text IS NULL OR status = $2)
                AND ($3::uuid IS NULL OR sop_version_id = $3)
            ORDER BY started_at DESC NULLS LAST
            LIMIT $4 OFFSET $5
            "#,
        )
        .bind(filter.worker_id)
        .bind(status_str)
        .bind(filter.sop_version_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(crate::error::map_sqlx)?;

        // 総件数カウント
        let total: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*) FROM work_executions
            WHERE
                ($1::uuid IS NULL OR primary_worker_id = $1)
                AND ($2::text IS NULL OR status = $2)
                AND ($3::uuid IS NULL OR sop_version_id = $3)
            "#,
        )
        .bind(filter.worker_id)
        .bind(filter.status.as_ref().map(|s| status_to_str(s).to_owned()))
        .bind(filter.sop_version_id)
        .fetch_one(&self.pool)
        .await
        .map_err(crate::error::map_sqlx)?;

        let items = rows
            .into_iter()
            .map(WorkExecution::try_from)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Page {
            items,
            total: u64::try_from(total).unwrap_or(0),
            page: page.page,
            per_page: page.per_page,
        })
    }

    /// (FNC-BE-007) 新規 WorkExecution を INSERT する。
    async fn create(&self, cmd: CreateWorkExecutionCmd) -> Result<WorkExecution, DomainError> {
        let row = sqlx::query_as::<_, WorkExecutionRow>(
            r#"
            INSERT INTO work_executions (
                work_execution_id,
                sop_version_id,
                primary_worker_id,
                secondary_worker_id,
                terminal_id,
                production_target_id,
                status,
                current_step_index,
                updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, 'NOT_STARTED', 0, NOW())
            RETURNING
                work_execution_id,
                sop_version_id,
                primary_worker_id,
                secondary_worker_id,
                terminal_id,
                production_target_id,
                status,
                current_step_index,
                started_at,
                completed_at,
                updated_at
            "#,
        )
        .bind(cmd.work_execution_id)
        .bind(cmd.sop_version_id)
        .bind(cmd.primary_worker_id)
        .bind(cmd.secondary_worker_id)
        .bind(cmd.terminal_id)
        .bind(cmd.production_target_id)
        .fetch_one(&self.pool)
        .await
        .map_err(crate::error::map_sqlx)?;

        WorkExecution::try_from(row)
    }

    /// 楽観ロック付きステータス更新。変更された行数を返す（0 なら競合）。
    async fn update_status_if_unchanged(
        &self,
        id: Uuid,
        new_status: WorkExecutionStatus,
        expected_updated_at: DateTime<Utc>,
    ) -> Result<u64, DomainError> {
        let result = sqlx::query(
            r#"
            UPDATE work_executions
            SET status = $1, updated_at = NOW()
            WHERE work_execution_id = $2
              AND updated_at = $3
            "#,
        )
        .bind(status_to_str(&new_status))
        .bind(id)
        .bind(expected_updated_at)
        .execute(&self.pool)
        .await
        .map_err(crate::error::map_sqlx)?;

        Ok(result.rows_affected())
    }
}
