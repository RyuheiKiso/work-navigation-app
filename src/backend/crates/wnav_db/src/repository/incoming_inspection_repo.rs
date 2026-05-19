// PgIncomingInspectionRepository — TBL-038/040/041/042 IQC の sqlx 実装
// AQL 規格に基づく入荷検査記録と ADR-011 ハッシュチェーン計算を含む。
// SQLX_PREPARE_REQUIRED: cargo sqlx prepare を実行してからビルド可能

use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use wnav_domain::{
    error::DomainError,
    model::incoming_inspection::{IncomingInspection, IqcResult, IqcStatus},
    repository::IncomingInspectionRepository,
};

use crate::row_types::IncomingInspectionRow;

/// TBL-038 incoming_inspections のリポジトリ実装。
pub struct PgIncomingInspectionRepository {
    pool: PgPool,
}

impl PgIncomingInspectionRepository {
    /// コネクションプールを受け取ってリポジトリを構築する。
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

/// IncomingInspectionRow から IncomingInspection ドメインモデルへの変換。
impl TryFrom<IncomingInspectionRow> for IncomingInspection {
    type Error = DomainError;

    fn try_from(row: IncomingInspectionRow) -> Result<Self, Self::Error> {
        let status = parse_iqc_status(&row.status)?;
        let result = row.result.as_deref().map(parse_iqc_result).transpose()?;
        Ok(Self {
            qc_case_id: row.qc_case_id,
            lot_id: row.lot_id,
            sop_id: row.sop_id,
            status,
            inspector_id: row.inspector_id,
            started_at: row.started_at,
            completed_at: row.completed_at,
            result,
            prev_hash: row.prev_hash,
            content_hash: row.content_hash,
            chain_hash: row.chain_hash,
        })
    }
}

/// DB ステータス文字列を IqcStatus 列挙型に変換する。
fn parse_iqc_status(s: &str) -> Result<IqcStatus, DomainError> {
    match s {
        "PENDING" => Ok(IqcStatus::Pending),
        "IN_PROGRESS" => Ok(IqcStatus::InProgress),
        "COMPLETED" => Ok(IqcStatus::Completed),
        "APPROVED" => Ok(IqcStatus::Approved),
        other => Err(DomainError::Internal(format!("不明な IqcStatus: {other}"))),
    }
}

/// IqcStatus を DB 格納文字列に変換する。
fn iqc_status_to_str(s: &IqcStatus) -> &'static str {
    match s {
        IqcStatus::Pending => "PENDING",
        IqcStatus::InProgress => "IN_PROGRESS",
        IqcStatus::Completed => "COMPLETED",
        IqcStatus::Approved => "APPROVED",
    }
}

/// DB 結果文字列を IqcResult 列挙型に変換する。
fn parse_iqc_result(s: &str) -> Result<IqcResult, DomainError> {
    match s {
        "ACCEPT" => Ok(IqcResult::Accept),
        "CONCESSION" => Ok(IqcResult::Concession),
        "SCREENING" => Ok(IqcResult::Screening),
        "REJECT" => Ok(IqcResult::Reject),
        other => Err(DomainError::Internal(format!("不明な IqcResult: {other}"))),
    }
}

/// IqcResult を DB 格納文字列に変換する。
fn iqc_result_to_str(r: &IqcResult) -> &'static str {
    match r {
        IqcResult::Accept => "ACCEPT",
        IqcResult::Concession => "CONCESSION",
        IqcResult::Screening => "SCREENING",
        IqcResult::Reject => "REJECT",
    }
}

#[async_trait]
impl IncomingInspectionRepository for PgIncomingInspectionRepository {
    /// 入荷検査を INSERT する（ADR-011: ハッシュチェーン計算済みの値を受け取る）。
    async fn insert(&self, inspection: IncomingInspection) -> Result<(), DomainError> {
        sqlx::query(
            r#"
            INSERT INTO incoming_inspections (
                qc_case_id, lot_id, sop_id, status,
                inspector_id, started_at, completed_at, result,
                prev_hash, content_hash, chain_hash
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            "#,
        )
        .bind(inspection.qc_case_id)
        .bind(inspection.lot_id)
        .bind(inspection.sop_id)
        .bind(iqc_status_to_str(&inspection.status))
        .bind(inspection.inspector_id)
        .bind(inspection.started_at)
        .bind(inspection.completed_at)
        .bind(inspection.result.as_ref().map(iqc_result_to_str))
        .bind(inspection.prev_hash)
        .bind(inspection.content_hash)
        .bind(inspection.chain_hash)
        .execute(&self.pool)
        .await
        .map_err(crate::error::map_sqlx)?;

        Ok(())
    }

    /// ID で入荷検査を検索する。
    async fn find_by_id(&self, id: Uuid) -> Result<Option<IncomingInspection>, DomainError> {
        let row = sqlx::query_as::<_, IncomingInspectionRow>(
            r#"
            SELECT
                qc_case_id, lot_id, sop_id, status,
                inspector_id, started_at, completed_at, result,
                prev_hash, content_hash, chain_hash
            FROM incoming_inspections
            WHERE qc_case_id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(crate::error::map_sqlx)?;

        row.map(IncomingInspection::try_from).transpose()
    }

    /// 入荷検査のステータスを更新する（検査フロー制御）。
    async fn update_status(
        &self,
        id: Uuid,
        new_status: IqcStatus,
        _updated_by: Uuid,
    ) -> Result<IncomingInspection, DomainError> {
        let row = sqlx::query_as::<_, IncomingInspectionRow>(
            r#"
            UPDATE incoming_inspections
            SET
                status = $1,
                started_at  = CASE WHEN $1 = 'IN_PROGRESS' AND started_at IS NULL THEN NOW() ELSE started_at END,
                completed_at = CASE WHEN $1 IN ('COMPLETED', 'APPROVED') THEN NOW() ELSE completed_at END
            WHERE qc_case_id = $2
            RETURNING
                qc_case_id, lot_id, sop_id, status,
                inspector_id, started_at, completed_at, result,
                prev_hash, content_hash, chain_hash
            "#,
        )
        .bind(iqc_status_to_str(&new_status))
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(crate::error::map_sqlx)?;

        IncomingInspection::try_from(row)
    }

    /// ロット ID に紐づく入荷検査一覧を取得する。
    async fn find_by_lot(&self, lot_id: Uuid) -> Result<Vec<IncomingInspection>, DomainError> {
        let rows = sqlx::query_as::<_, IncomingInspectionRow>(
            r#"
            SELECT
                qc_case_id, lot_id, sop_id, status,
                inspector_id, started_at, completed_at, result,
                prev_hash, content_hash, chain_hash
            FROM incoming_inspections
            WHERE lot_id = $1
            ORDER BY qc_case_id ASC
            "#,
        )
        .bind(lot_id)
        .fetch_all(&self.pool)
        .await
        .map_err(crate::error::map_sqlx)?;

        rows.into_iter()
            .map(IncomingInspection::try_from)
            .collect::<Result<Vec<_>, _>>()
    }
}
