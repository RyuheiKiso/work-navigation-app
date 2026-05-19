// PgAndonRepository — TBL-015 andon_events の sqlx 実装
// アンドン発報・一覧取得・解決処理を担う。
// SQLX_PREPARE_REQUIRED: cargo sqlx prepare を実行してからビルド可能

use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use wnav_domain::{
    error::DomainError,
    model::andon::{AndonEvent, AndonStatus},
    repository::AndonRepository,
};

use crate::row_types::AndonEventRow;

/// TBL-015 andon_events のリポジトリ実装。
pub struct PgAndonRepository {
    pool: PgPool,
}

impl PgAndonRepository {
    /// コネクションプールを受け取ってリポジトリを構築する。
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

/// AndonEventRow から AndonEvent ドメインモデルへの変換。
impl TryFrom<AndonEventRow> for AndonEvent {
    type Error = DomainError;

    fn try_from(row: AndonEventRow) -> Result<Self, Self::Error> {
        let status = parse_andon_status(&row.status)?;
        Ok(Self {
            andon_id: row.andon_id,
            work_execution_id: row.work_execution_id,
            triggered_by: row.triggered_by,
            reason_code: row.reason_code,
            reason_text: row.reason_text,
            status,
            created_at: row.created_at,
        })
    }
}

/// DB ステータス文字列を AndonStatus 列挙型に変換する。
fn parse_andon_status(s: &str) -> Result<AndonStatus, DomainError> {
    match s {
        "OPEN" => Ok(AndonStatus::Open),
        "RESOLVED" => Ok(AndonStatus::Resolved),
        "ESCALATED" => Ok(AndonStatus::Escalated),
        other => Err(DomainError::Internal(format!(
            "不明な AndonStatus: {other}"
        ))),
    }
}

#[async_trait]
impl AndonRepository for PgAndonRepository {
    /// アンドンイベントを INSERT する。
    async fn insert(&self, event: AndonEvent) -> Result<(), DomainError> {
        sqlx::query(
            r#"
            INSERT INTO andon_events (
                andon_id, work_execution_id, triggered_by,
                reason_code, reason_text, status, created_at
            )
            VALUES ($1, $2, $3, $4, $5, 'OPEN', $6)
            "#,
        )
        .bind(event.andon_id)
        .bind(event.work_execution_id)
        .bind(event.triggered_by)
        .bind(event.reason_code)
        .bind(event.reason_text)
        .bind(event.created_at)
        .execute(&self.pool)
        .await
        .map_err(crate::error::map_sqlx)?;

        Ok(())
    }

    /// アクティブ（OPEN）なアンドンイベント一覧を取得する。
    async fn list_active(&self) -> Result<Vec<AndonEvent>, DomainError> {
        let rows = sqlx::query_as::<_, AndonEventRow>(
            r#"
            SELECT
                andon_id, work_execution_id, triggered_by,
                reason_code, reason_text, status, created_at
            FROM andon_events
            WHERE status = 'OPEN'
            ORDER BY created_at DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(crate::error::map_sqlx)?;

        rows.into_iter()
            .map(AndonEvent::try_from)
            .collect::<Result<Vec<_>, _>>()
    }

    /// アンドンを解決済みに更新する（監督者操作）。
    async fn resolve(&self, andon_id: Uuid, resolved_by: Uuid) -> Result<(), DomainError> {
        sqlx::query(
            r#"
            UPDATE andon_events
            SET
                status = 'RESOLVED',
                resolved_by = $1,
                resolved_at = NOW()
            WHERE andon_id = $2 AND status = 'OPEN'
            "#,
        )
        .bind(resolved_by)
        .bind(andon_id)
        .execute(&self.pool)
        .await
        .map_err(crate::error::map_sqlx)?;

        Ok(())
    }
}
