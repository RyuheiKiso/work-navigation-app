// PgReworkRepository — TBL-043/044/045 の sqlx 実装
// リワーク（手直し）の状態遷移と ADR-011 ハッシュチェーン計算を含む。
// Two-Person Integrity（FR-AU-007）による 2 名検証が必須。
// SQLX_PREPARE_REQUIRED: cargo sqlx prepare を実行してからビルド可能

use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use wnav_domain::{
    error::DomainError,
    model::rework::{Rework, ReworkStatus},
    repository::ReworkRepository,
};

use crate::row_types::ReworkRow;

/// TBL-043 reworks のリポジトリ実装。
pub struct PgReworkRepository {
    pool: PgPool,
}

impl PgReworkRepository {
    /// コネクションプールを受け取ってリポジトリを構築する。
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

/// ReworkRow から Rework ドメインモデルへの変換。
impl TryFrom<ReworkRow> for Rework {
    type Error = DomainError;

    fn try_from(row: ReworkRow) -> Result<Self, Self::Error> {
        let status = parse_rework_status(&row.status)?;
        Ok(Self {
            rework_id: row.rework_id,
            parent_nonconformity_id: row.parent_nonconformity_id,
            lot_id: row.lot_id,
            sop_id: row.sop_id,
            status,
            assignee: row.assignee,
            started_at: row.started_at,
            completed_at: row.completed_at,
            prev_hash: row.prev_hash,
            content_hash: row.content_hash,
            chain_hash: row.chain_hash,
        })
    }
}

/// DB ステータス文字列を ReworkStatus 列挙型に変換する。
fn parse_rework_status(s: &str) -> Result<ReworkStatus, DomainError> {
    match s {
        "PENDING" => Ok(ReworkStatus::Pending),
        "IN_PROGRESS" => Ok(ReworkStatus::InProgress),
        "PENDING_VERIFICATION" => Ok(ReworkStatus::PendingVerification),
        "VERIFIED" => Ok(ReworkStatus::Verified),
        "CLOSED" => Ok(ReworkStatus::Closed),
        other => Err(DomainError::Internal(format!(
            "不明な ReworkStatus: {other}"
        ))),
    }
}

/// ReworkStatus を DB 格納文字列に変換する。
fn rework_status_to_str(s: &ReworkStatus) -> &'static str {
    match s {
        ReworkStatus::Pending => "PENDING",
        ReworkStatus::InProgress => "IN_PROGRESS",
        ReworkStatus::PendingVerification => "PENDING_VERIFICATION",
        ReworkStatus::Verified => "VERIFIED",
        ReworkStatus::Closed => "CLOSED",
    }
}

#[async_trait]
impl ReworkRepository for PgReworkRepository {
    /// リワークを INSERT する（ADR-011: ハッシュチェーン計算済みの値を受け取る）。
    async fn insert(&self, rework: Rework) -> Result<(), DomainError> {
        sqlx::query(
            r#"
            INSERT INTO reworks (
                rework_id, parent_nonconformity_id, lot_id, sop_id,
                status, assignee, started_at, completed_at,
                prev_hash, content_hash, chain_hash
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            "#,
        )
        .bind(rework.rework_id)
        .bind(rework.parent_nonconformity_id)
        .bind(rework.lot_id)
        .bind(rework.sop_id)
        .bind(rework_status_to_str(&rework.status))
        .bind(rework.assignee)
        .bind(rework.started_at)
        .bind(rework.completed_at)
        .bind(rework.prev_hash)
        .bind(rework.content_hash)
        .bind(rework.chain_hash)
        .execute(&self.pool)
        .await
        .map_err(crate::error::map_sqlx)?;

        Ok(())
    }

    /// ID でリワークを検索する。
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Rework>, DomainError> {
        let row = sqlx::query_as::<_, ReworkRow>(
            r#"
            SELECT
                rework_id, parent_nonconformity_id, lot_id, sop_id,
                status, assignee, started_at, completed_at,
                prev_hash, content_hash, chain_hash
            FROM reworks
            WHERE rework_id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(crate::error::map_sqlx)?;

        row.map(Rework::try_from).transpose()
    }

    /// リワークのステータスを更新する（リワークフロー制御）。
    async fn update_status(
        &self,
        id: Uuid,
        new_status: ReworkStatus,
        _updated_by: Uuid,
    ) -> Result<Rework, DomainError> {
        let row = sqlx::query_as::<_, ReworkRow>(
            r#"
            UPDATE reworks
            SET
                status = $1,
                started_at   = CASE WHEN $1 = 'IN_PROGRESS' AND started_at IS NULL THEN NOW() ELSE started_at END,
                completed_at = CASE WHEN $1 = 'CLOSED' THEN NOW() ELSE completed_at END
            WHERE rework_id = $2
            RETURNING
                rework_id, parent_nonconformity_id, lot_id, sop_id,
                status, assignee, started_at, completed_at,
                prev_hash, content_hash, chain_hash
            "#,
        )
        .bind(rework_status_to_str(&new_status))
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(crate::error::map_sqlx)?;

        Rework::try_from(row)
    }

    /// 不適合 ID に紐づくリワーク一覧を取得する。
    async fn find_by_nonconformity(
        &self,
        nonconformity_id: Uuid,
    ) -> Result<Vec<Rework>, DomainError> {
        let rows = sqlx::query_as::<_, ReworkRow>(
            r#"
            SELECT
                rework_id, parent_nonconformity_id, lot_id, sop_id,
                status, assignee, started_at, completed_at,
                prev_hash, content_hash, chain_hash
            FROM reworks
            WHERE parent_nonconformity_id = $1
            ORDER BY rework_id ASC
            "#,
        )
        .bind(nonconformity_id)
        .fetch_all(&self.pool)
        .await
        .map_err(crate::error::map_sqlx)?;

        rows.into_iter()
            .map(Rework::try_from)
            .collect::<Result<Vec<_>, _>>()
    }
}
