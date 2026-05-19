// PgWorkAssignmentRepository — TBL-052/053 の sqlx 実装
// 外部システムからの作業指示受信と端末への配信管理を担う。
// SQLX_PREPARE_REQUIRED: cargo sqlx prepare を実行してからビルド可能

use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use wnav_domain::{
    error::DomainError,
    model::work_assignment::{AssignmentStatus, WorkAssignment},
    repository::WorkAssignmentRepository,
};

use crate::row_types::WorkAssignmentRow;

/// TBL-052 work_assignments のリポジトリ実装。
pub struct PgWorkAssignmentRepository {
    pool: PgPool,
}

impl PgWorkAssignmentRepository {
    /// コネクションプールを受け取ってリポジトリを構築する。
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

/// WorkAssignmentRow から WorkAssignment ドメインモデルへの変換。
impl TryFrom<WorkAssignmentRow> for WorkAssignment {
    type Error = DomainError;

    fn try_from(row: WorkAssignmentRow) -> Result<Self, Self::Error> {
        let status = parse_assignment_status(&row.status)?;
        Ok(Self {
            assignment_id: row.assignment_id,
            sop_id: row.sop_id,
            case_id: row.case_id,
            lot_id: row.lot_id,
            priority: row.priority,
            status,
            target_terminal_id: row.target_terminal_id,
            external_system: row.external_system,
            idempotency_key: row.idempotency_key,
            received_at: row.received_at,
        })
    }
}

/// DB ステータス文字列を AssignmentStatus 列挙型に変換する。
fn parse_assignment_status(s: &str) -> Result<AssignmentStatus, DomainError> {
    match s {
        "PENDING" => Ok(AssignmentStatus::Pending),
        "DISPATCHED" => Ok(AssignmentStatus::Dispatched),
        "ACCEPTED" => Ok(AssignmentStatus::Accepted),
        "INPROGRESS" => Ok(AssignmentStatus::Inprogress),
        "COMPLETED" => Ok(AssignmentStatus::Completed),
        "CANCELLED" => Ok(AssignmentStatus::Cancelled),
        other => Err(DomainError::Internal(format!(
            "不明な AssignmentStatus: {other}"
        ))),
    }
}

/// AssignmentStatus を DB 格納文字列に変換する。
/// 現在は直接 SQL リテラルで更新しているが、汎用更新時に使用する。
#[allow(dead_code)]
fn assignment_status_to_str(s: &AssignmentStatus) -> &'static str {
    match s {
        AssignmentStatus::Pending => "PENDING",
        AssignmentStatus::Dispatched => "DISPATCHED",
        AssignmentStatus::Accepted => "ACCEPTED",
        AssignmentStatus::Inprogress => "INPROGRESS",
        AssignmentStatus::Completed => "COMPLETED",
        AssignmentStatus::Cancelled => "CANCELLED",
    }
}

#[async_trait]
impl WorkAssignmentRepository for PgWorkAssignmentRepository {
    /// 作業指示を INSERT する（冪等性キー重複時は DuplicateExternalKey エラー）。
    async fn insert(&self, assignment: WorkAssignment) -> Result<(), DomainError> {
        let result = sqlx::query(
            r#"
            INSERT INTO work_assignments (
                assignment_id, sop_id, case_id, lot_id,
                priority, status, target_terminal_id,
                external_system, idempotency_key, received_at
            )
            VALUES ($1, $2, $3, $4, $5, 'PENDING', $6, $7, $8, $9)
            ON CONFLICT (idempotency_key) DO NOTHING
            "#,
        )
        .bind(assignment.assignment_id)
        .bind(assignment.sop_id)
        .bind(assignment.case_id)
        .bind(assignment.lot_id)
        .bind(assignment.priority)
        .bind(assignment.target_terminal_id)
        .bind(assignment.external_system)
        .bind(assignment.idempotency_key)
        .bind(assignment.received_at)
        .execute(&self.pool)
        .await
        .map_err(crate::error::map_sqlx)?;

        // 冪等性キー重複（0 行 INSERT）の場合はエラーを返す
        if result.rows_affected() == 0 {
            return Err(DomainError::DuplicateExternalKey {
                key: assignment.idempotency_key.to_string(),
            });
        }

        Ok(())
    }

    /// 端末への配信待ち作業指示を取得する。
    async fn find_pending_for_terminal(
        &self,
        terminal_id: Uuid,
    ) -> Result<Vec<WorkAssignment>, DomainError> {
        let rows = sqlx::query_as::<_, WorkAssignmentRow>(
            r#"
            SELECT
                assignment_id, sop_id, case_id, lot_id,
                priority, status, target_terminal_id,
                external_system, idempotency_key, received_at
            FROM work_assignments
            WHERE status = 'PENDING'
              AND (target_terminal_id = $1 OR target_terminal_id IS NULL)
            ORDER BY priority ASC, received_at ASC
            "#,
        )
        .bind(terminal_id)
        .fetch_all(&self.pool)
        .await
        .map_err(crate::error::map_sqlx)?;

        rows.into_iter()
            .map(WorkAssignment::try_from)
            .collect::<Result<Vec<_>, _>>()
    }

    /// 作業指示を配信済みに更新する。
    async fn mark_dispatched(&self, assignment_id: Uuid) -> Result<(), DomainError> {
        sqlx::query(
            r#"
            UPDATE work_assignments
            SET status = 'DISPATCHED'
            WHERE assignment_id = $1
            "#,
        )
        .bind(assignment_id)
        .execute(&self.pool)
        .await
        .map_err(crate::error::map_sqlx)?;

        Ok(())
    }

    /// 冪等性キーで作業指示を検索する（重複受信チェック）。
    async fn find_by_idempotency_key(
        &self,
        key: Uuid,
    ) -> Result<Option<WorkAssignment>, DomainError> {
        let row = sqlx::query_as::<_, WorkAssignmentRow>(
            r#"
            SELECT
                assignment_id, sop_id, case_id, lot_id,
                priority, status, target_terminal_id,
                external_system, idempotency_key, received_at
            FROM work_assignments
            WHERE idempotency_key = $1
            "#,
        )
        .bind(key)
        .fetch_optional(&self.pool)
        .await
        .map_err(crate::error::map_sqlx)?;

        row.map(WorkAssignment::try_from).transpose()
    }
}
