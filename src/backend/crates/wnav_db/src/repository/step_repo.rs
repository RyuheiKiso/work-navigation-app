// PgStepRepository — TBL-008 steps の sqlx 実装
// SOP ステップのバッチ INSERT・ドラッグ&ドロップ並び替えを含む。
// SQLX_PREPARE_REQUIRED: cargo sqlx prepare を実行してからビルド可能

use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use wnav_domain::{
    error::DomainError,
    model::step::{Step, StepType},
    repository::StepRepository,
};

use crate::row_types::StepRow;

/// TBL-008 steps のリポジトリ実装。
pub struct PgStepRepository {
    pool: PgPool,
}

impl PgStepRepository {
    /// コネクションプールを受け取ってリポジトリを構築する。
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

/// StepRow から Step ドメインモデルへの変換。
impl TryFrom<StepRow> for Step {
    type Error = DomainError;

    fn try_from(row: StepRow) -> Result<Self, Self::Error> {
        let step_type = parse_step_type(&row.step_type)?;
        Ok(Self {
            step_id: row.step_id,
            sop_id: row.sop_id,
            step_number: u32::try_from(row.step_number).unwrap_or(1),
            title: row.title,
            instruction: row.instruction,
            condition_dsl: row.condition_dsl,
            evidence_required: row.evidence_required,
            sign_required: row.sign_required,
            skippable: row.skippable,
            estimated_duration_secs: row
                .estimated_duration_secs
                .map(|v| u32::try_from(v).unwrap_or(0)),
            step_type,
        })
    }
}

/// DB ステップ種別文字列を StepType 列挙型に変換する。
fn parse_step_type(s: &str) -> Result<StepType, DomainError> {
    match s {
        "STANDARD" => Ok(StepType::Standard),
        "CRITICAL" => Ok(StepType::Critical),
        "MEASUREMENT" => Ok(StepType::Measurement),
        "SIGNATURE" => Ok(StepType::Signature),
        "QR_SCAN" => Ok(StepType::QrScan),
        "EVIDENCE" => Ok(StepType::Evidence),
        "CUSTOM" => Ok(StepType::Custom),
        other => Err(DomainError::Internal(format!("不明な StepType: {other}"))),
    }
}

/// StepType を DB 格納文字列に変換する。
fn step_type_to_str(t: &StepType) -> &'static str {
    match t {
        StepType::Standard => "STANDARD",
        StepType::Critical => "CRITICAL",
        StepType::Measurement => "MEASUREMENT",
        StepType::Signature => "SIGNATURE",
        StepType::QrScan => "QR_SCAN",
        StepType::Evidence => "EVIDENCE",
        StepType::Custom => "CUSTOM",
    }
}

#[async_trait]
impl StepRepository for PgStepRepository {
    /// SOP ID に紐づく全ステップを step_number 昇順で取得する。
    async fn find_by_sop(&self, sop_id: Uuid) -> Result<Vec<Step>, DomainError> {
        let rows = sqlx::query_as::<_, StepRow>(
            r#"
            SELECT
                step_id, sop_id, step_number, title, instruction,
                condition_dsl, evidence_required, sign_required,
                skippable, estimated_duration_secs, step_type
            FROM steps
            WHERE sop_id = $1
            ORDER BY step_number ASC
            "#,
        )
        .bind(sop_id)
        .fetch_all(&self.pool)
        .await
        .map_err(crate::error::map_sqlx)?;

        rows.into_iter().map(Step::try_from).collect()
    }

    /// ID でステップを検索する。
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Step>, DomainError> {
        let row = sqlx::query_as::<_, StepRow>(
            r#"
            SELECT
                step_id, sop_id, step_number, title, instruction,
                condition_dsl, evidence_required, sign_required,
                skippable, estimated_duration_secs, step_type
            FROM steps
            WHERE step_id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(crate::error::map_sqlx)?;

        row.map(Step::try_from).transpose()
    }

    /// 複数ステップを一括 INSERT する（SOP 保存時のバッチ操作）。
    async fn create_batch(&self, steps: Vec<Step>) -> Result<Vec<Step>, DomainError> {
        let mut result = Vec::with_capacity(steps.len());

        for step in steps {
            let row = sqlx::query_as::<_, StepRow>(
                r#"
                INSERT INTO steps (
                    step_id, sop_id, step_number, title, instruction,
                    condition_dsl, evidence_required, sign_required,
                    skippable, estimated_duration_secs, step_type
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
                RETURNING
                    step_id, sop_id, step_number, title, instruction,
                    condition_dsl, evidence_required, sign_required,
                    skippable, estimated_duration_secs, step_type
                "#,
            )
            .bind(step.step_id)
            .bind(step.sop_id)
            .bind(i32::try_from(step.step_number).unwrap_or(1))
            .bind(&step.title)
            .bind(&step.instruction)
            .bind(&step.condition_dsl)
            .bind(step.evidence_required)
            .bind(step.sign_required)
            .bind(step.skippable)
            .bind(
                step.estimated_duration_secs
                    .map(|v| i32::try_from(v).unwrap_or(0)),
            )
            .bind(step_type_to_str(&step.step_type))
            .fetch_one(&self.pool)
            .await
            .map_err(crate::error::map_sqlx)?;

            result.push(Step::try_from(row)?);
        }

        Ok(result)
    }

    /// ステップの表示順（step_number）を一括更新する（ドラッグ&ドロップ並び替え）。
    /// ordered_ids の index + 1 を step_number として割り当てる。
    async fn reorder(&self, sop_id: Uuid, ordered_ids: Vec<Uuid>) -> Result<(), DomainError> {
        for (index, step_id) in ordered_ids.iter().enumerate() {
            let new_number = i32::try_from(index + 1).unwrap_or(1);
            sqlx::query(
                r#"
                UPDATE steps
                SET step_number = $1
                WHERE step_id = $2 AND sop_id = $3
                "#,
            )
            .bind(new_number)
            .bind(step_id)
            .bind(sop_id)
            .execute(&self.pool)
            .await
            .map_err(crate::error::map_sqlx)?;
        }

        Ok(())
    }
}
