// PgWorkEventRepository — TBL-001 work_events の sqlx 実装（Append-only）
// INSERT のみを提供し、UPDATE・DELETE は提供しない（Append-only 原則）。
// ハッシュチェーン計算は wnav_hash_chain クレートに委譲する。
// SQLX_PREPARE_REQUIRED: cargo sqlx prepare を実行してからビルド可能

use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use wnav_domain::{
    error::DomainError, model::work_event::WorkEvent, repository::WorkEventRepository,
};

use crate::row_types::WorkEventRow;

/// TBL-001 work_events の Append-only リポジトリ実装。
/// app_event_insert プール経由で INSERT 専用に使用する。
pub struct PgWorkEventRepository {
    pool: PgPool,
}

impl PgWorkEventRepository {
    /// コネクションプールを受け取ってリポジトリを構築する。
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

/// WorkEventRow から WorkEvent ドメインモデルへの変換。
impl TryFrom<WorkEventRow> for WorkEvent {
    type Error = DomainError;

    fn try_from(row: WorkEventRow) -> Result<Self, Self::Error> {
        Ok(Self {
            event_id: row.event_id,
            case_id: row.case_id,
            activity: row.activity,
            step_id: row.step_id,
            timestamp_client: row.timestamp_client,
            timestamp_server: row.timestamp_server,
            resource: row.resource,
            sop_version_id: row.sop_version_id,
            terminal_id: row.terminal_id,
            payload: row.payload,
            prev_hash: row.prev_hash,
            content_hash: row.content_hash,
        })
    }
}

#[async_trait]
impl WorkEventRepository for PgWorkEventRepository {
    /// Append-only: 単一の WorkEvent を INSERT する。
    /// UPDATE・DELETE は Append-only 原則により提供しない。
    async fn insert(&self, event: WorkEvent) -> Result<(), DomainError> {
        sqlx::query(
            r#"
            INSERT INTO work_events (
                event_id,
                case_id,
                activity,
                step_id,
                timestamp_client,
                timestamp_server,
                resource,
                sop_version_id,
                terminal_id,
                payload,
                prev_hash,
                content_hash
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            "#,
        )
        .bind(event.event_id)
        .bind(event.case_id)
        .bind(event.activity)
        .bind(event.step_id)
        .bind(event.timestamp_client)
        .bind(event.timestamp_server)
        .bind(event.resource)
        .bind(event.sop_version_id)
        .bind(event.terminal_id)
        .bind(event.payload)
        .bind(event.prev_hash)
        .bind(event.content_hash)
        .execute(&self.pool)
        .await
        .map_err(crate::error::map_sqlx)?;

        Ok(())
    }

    /// 指定 case_id の最新チェーンハッシュ（content_hash）を取得する。
    /// genesis（初回）は "0"×64 の文字列を返す。
    async fn latest_hash(&self, case_id: Uuid) -> Result<String, DomainError> {
        let hash: Option<String> = sqlx::query_scalar(
            r#"
            SELECT content_hash
            FROM work_events
            WHERE case_id = $1
            ORDER BY event_id DESC
            LIMIT 1
            "#,
        )
        .bind(case_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(crate::error::map_sqlx)?;

        // genesis（初回）は 64 桁の "0" を返す（詳細設計 §2）
        Ok(hash.unwrap_or_else(|| "0".repeat(64)))
    }

    /// case_id に紐づく全 WorkEvent を時系列順（event_id 昇順）で取得する。
    async fn list_by_case(&self, case_id: Uuid) -> Result<Vec<WorkEvent>, DomainError> {
        let rows = sqlx::query_as::<_, WorkEventRow>(
            r#"
            SELECT
                event_id,
                case_id,
                activity,
                step_id,
                timestamp_client,
                timestamp_server,
                resource,
                sop_version_id,
                terminal_id,
                payload,
                prev_hash,
                content_hash
            FROM work_events
            WHERE case_id = $1
            ORDER BY event_id ASC
            "#,
        )
        .bind(case_id)
        .fetch_all(&self.pool)
        .await
        .map_err(crate::error::map_sqlx)?;

        rows.into_iter()
            .map(WorkEvent::try_from)
            .collect::<Result<Vec<_>, _>>()
    }
}
