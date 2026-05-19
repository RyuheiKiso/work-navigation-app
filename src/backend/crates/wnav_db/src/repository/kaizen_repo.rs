// PgKaizenRepository — TBL-014 kaizen_reports の sqlx 実装
// 改善提案のフロー管理（Draft → UnderReview → Approved/Rejected → Implemented）を担う。
// SQLX_PREPARE_REQUIRED: cargo sqlx prepare を実行してからビルド可能

use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use wnav_domain::{
    error::DomainError,
    model::kaizen::{KaizenReport, KaizenStatus},
    repository::KaizenRepository,
};

use crate::row_types::KaizenReportRow;

/// TBL-014 kaizen_reports のリポジトリ実装。
pub struct PgKaizenRepository {
    pool: PgPool,
}

impl PgKaizenRepository {
    /// コネクションプールを受け取ってリポジトリを構築する。
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

/// KaizenReportRow から KaizenReport ドメインモデルへの変換。
impl TryFrom<KaizenReportRow> for KaizenReport {
    type Error = DomainError;

    fn try_from(row: KaizenReportRow) -> Result<Self, Self::Error> {
        let status = parse_kaizen_status(&row.status)?;
        Ok(Self {
            report_id: row.report_id,
            reporter_id: row.reporter_id,
            category: row.category,
            title: row.title,
            description: row.description,
            status,
            impact_level: row.impact_level,
        })
    }
}

/// DB ステータス文字列を KaizenStatus 列挙型に変換する。
fn parse_kaizen_status(s: &str) -> Result<KaizenStatus, DomainError> {
    match s {
        "DRAFT" => Ok(KaizenStatus::Draft),
        "UNDER_REVIEW" => Ok(KaizenStatus::UnderReview),
        "APPROVED" => Ok(KaizenStatus::Approved),
        "REJECTED" => Ok(KaizenStatus::Rejected),
        "IMPLEMENTED" => Ok(KaizenStatus::Implemented),
        other => Err(DomainError::Internal(format!(
            "不明な KaizenStatus: {other}"
        ))),
    }
}

/// KaizenStatus を DB 格納文字列に変換する。
fn status_to_str(s: &KaizenStatus) -> &'static str {
    match s {
        KaizenStatus::Draft => "DRAFT",
        KaizenStatus::UnderReview => "UNDER_REVIEW",
        KaizenStatus::Approved => "APPROVED",
        KaizenStatus::Rejected => "REJECTED",
        KaizenStatus::Implemented => "IMPLEMENTED",
    }
}

#[async_trait]
impl KaizenRepository for PgKaizenRepository {
    /// 改善提案を INSERT する（初期ステータスは Draft）。
    async fn insert(&self, report: KaizenReport) -> Result<(), DomainError> {
        sqlx::query(
            r#"
            INSERT INTO kaizen_reports (
                report_id, reporter_id, category,
                title, description, status, impact_level
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
        )
        .bind(report.report_id)
        .bind(report.reporter_id)
        .bind(report.category)
        .bind(report.title)
        .bind(report.description)
        .bind(status_to_str(&report.status))
        .bind(report.impact_level)
        .execute(&self.pool)
        .await
        .map_err(crate::error::map_sqlx)?;

        Ok(())
    }

    /// ID で改善提案を検索する。
    async fn find_by_id(&self, id: Uuid) -> Result<Option<KaizenReport>, DomainError> {
        let row = sqlx::query_as::<_, KaizenReportRow>(
            r#"
            SELECT
                report_id, reporter_id, category,
                title, description, status, impact_level
            FROM kaizen_reports
            WHERE report_id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(crate::error::map_sqlx)?;

        row.map(KaizenReport::try_from).transpose()
    }

    /// 改善提案のステータスを更新する。
    async fn update_status(
        &self,
        id: Uuid,
        new_status: KaizenStatus,
        _updated_by: Uuid,
    ) -> Result<KaizenReport, DomainError> {
        let row = sqlx::query_as::<_, KaizenReportRow>(
            r#"
            UPDATE kaizen_reports
            SET status = $1
            WHERE report_id = $2
            RETURNING
                report_id, reporter_id, category,
                title, description, status, impact_level
            "#,
        )
        .bind(status_to_str(&new_status))
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(crate::error::map_sqlx)?;

        KaizenReport::try_from(row)
    }
}
