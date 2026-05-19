// PgOutboxRepository — TBL-003 outbox_events の sqlx 実装
// Transactional Outbox パターン: 配信待ちは FOR UPDATE SKIP LOCKED で排他取得する。
// SQLX_PREPARE_REQUIRED: cargo sqlx prepare を実行してからビルド可能

use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use wnav_domain::{
    error::DomainError,
    model::outbox::{OutboxEvent, OutboxStatus},
    repository::OutboxRepository,
};

use crate::row_types::OutboxEventRow;

/// TBL-003 outbox_events のリポジトリ実装。
pub struct PgOutboxRepository {
    pool: PgPool,
}

impl PgOutboxRepository {
    /// コネクションプールを受け取ってリポジトリを構築する。
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

/// OutboxEventRow から OutboxEvent ドメインモデルへの変換。
impl TryFrom<OutboxEventRow> for OutboxEvent {
    type Error = DomainError;

    fn try_from(row: OutboxEventRow) -> Result<Self, Self::Error> {
        let status = parse_outbox_status(&row.status)?;
        Ok(Self {
            outbox_id: row.outbox_id,
            event_id: row.event_id,
            idempotency_key: row.idempotency_key,
            event_type: row.event_type,
            payload: row.payload,
            status,
            retry_count: u32::try_from(row.retry_count).unwrap_or(0),
            last_attempted_at: row.last_attempted_at,
        })
    }
}

/// DB ステータス文字列を OutboxStatus 列挙型に変換する。
fn parse_outbox_status(s: &str) -> Result<OutboxStatus, DomainError> {
    match s {
        "PENDING" => Ok(OutboxStatus::Pending),
        "PROCESSING" => Ok(OutboxStatus::Processing),
        "SENT" => Ok(OutboxStatus::Sent),
        "FAILED" => Ok(OutboxStatus::Failed),
        "DEAD_LETTERED" => Ok(OutboxStatus::DeadLettered),
        other => Err(DomainError::Internal(format!(
            "不明な OutboxStatus: {other}"
        ))),
    }
}

#[async_trait]
impl OutboxRepository for PgOutboxRepository {
    /// Outbox イベントを INSERT する（作業イベントと同一 TX で呼び出す）。
    async fn insert(&self, event: OutboxEvent) -> Result<(), DomainError> {
        sqlx::query(
            r#"
            INSERT INTO outbox_events (
                outbox_id, event_id, idempotency_key, event_type,
                payload, status, retry_count, last_attempted_at
            )
            VALUES ($1, $2, $3, $4, $5, 'PENDING', 0, NULL)
            "#,
        )
        .bind(event.outbox_id)
        .bind(event.event_id)
        .bind(event.idempotency_key)
        .bind(event.event_type)
        .bind(event.payload)
        .execute(&self.pool)
        .await
        .map_err(crate::error::map_sqlx)?;

        Ok(())
    }

    /// 配信待ちの Outbox イベントを取得する（FOR UPDATE SKIP LOCKED で排他取得）。
    async fn list_pending(&self, limit: u32) -> Result<Vec<OutboxEvent>, DomainError> {
        let rows = sqlx::query_as::<_, OutboxEventRow>(
            r#"
            SELECT
                outbox_id, event_id, idempotency_key, event_type,
                payload, status, retry_count, last_attempted_at
            FROM outbox_events
            WHERE status IN ('PENDING', 'FAILED')
            ORDER BY outbox_id ASC
            LIMIT $1
            FOR UPDATE SKIP LOCKED
            "#,
        )
        .bind(i64::from(limit))
        .fetch_all(&self.pool)
        .await
        .map_err(crate::error::map_sqlx)?;

        rows.into_iter()
            .map(OutboxEvent::try_from)
            .collect::<Result<Vec<_>, _>>()
    }

    /// 配信成功を記録する（ステータスを SENT に更新）。
    async fn mark_sent(&self, outbox_id: Uuid) -> Result<(), DomainError> {
        sqlx::query(
            r#"
            UPDATE outbox_events
            SET status = 'SENT', last_attempted_at = NOW()
            WHERE outbox_id = $1
            "#,
        )
        .bind(outbox_id)
        .execute(&self.pool)
        .await
        .map_err(crate::error::map_sqlx)?;

        Ok(())
    }

    /// 配信失敗を記録する（retry_count を増やし、ステータスを FAILED に更新）。
    async fn mark_failed(&self, outbox_id: Uuid) -> Result<(), DomainError> {
        sqlx::query(
            r#"
            UPDATE outbox_events
            SET
                status = 'FAILED',
                retry_count = retry_count + 1,
                last_attempted_at = NOW()
            WHERE outbox_id = $1
            "#,
        )
        .bind(outbox_id)
        .execute(&self.pool)
        .await
        .map_err(crate::error::map_sqlx)?;

        Ok(())
    }

    /// 最大リトライ超過のイベントをデッドレターキューに移動する。
    async fn move_to_dlq(&self, outbox_id: Uuid) -> Result<(), DomainError> {
        sqlx::query(
            r#"
            UPDATE outbox_events
            SET status = 'DEAD_LETTERED', last_attempted_at = NOW()
            WHERE outbox_id = $1
            "#,
        )
        .bind(outbox_id)
        .execute(&self.pool)
        .await
        .map_err(crate::error::map_sqlx)?;

        Ok(())
    }
}
