// PgMeasurementRepository — TBL-010 measurements の sqlx 実装
// 数値測定ステップで記録される測定値を管理する。
// SQLX_PREPARE_REQUIRED: cargo sqlx prepare を実行してからビルド可能

use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use wnav_domain::{
    error::DomainError, model::measurement::Measurement, repository::MeasurementRepository,
};

use crate::row_types::MeasurementRow;

/// TBL-010 measurements のリポジトリ実装。
pub struct PgMeasurementRepository {
    pool: PgPool,
}

impl PgMeasurementRepository {
    /// コネクションプールを受け取ってリポジトリを構築する。
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

/// MeasurementRow から Measurement ドメインモデルへの変換。
impl From<MeasurementRow> for Measurement {
    fn from(row: MeasurementRow) -> Self {
        Self {
            measurement_id: row.measurement_id,
            work_execution_id: row.work_execution_id,
            step_id: row.step_id,
            value: row.value,
            unit: row.unit,
            nominal: row.nominal,
            upper_limit: row.upper_limit,
            lower_limit: row.lower_limit,
            cp: row.cp,
            cpk: row.cpk,
        }
    }
}

#[async_trait]
impl MeasurementRepository for PgMeasurementRepository {
    /// 測定値を INSERT する。
    async fn insert(&self, measurement: Measurement) -> Result<(), DomainError> {
        sqlx::query(
            r#"
            INSERT INTO measurements (
                measurement_id, work_execution_id, step_id,
                value, unit, nominal, upper_limit, lower_limit, cp, cpk
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#,
        )
        .bind(measurement.measurement_id)
        .bind(measurement.work_execution_id)
        .bind(measurement.step_id)
        .bind(measurement.value)
        .bind(measurement.unit)
        .bind(measurement.nominal)
        .bind(measurement.upper_limit)
        .bind(measurement.lower_limit)
        .bind(measurement.cp)
        .bind(measurement.cpk)
        .execute(&self.pool)
        .await
        .map_err(crate::error::map_sqlx)?;

        Ok(())
    }

    /// ステップ ID に紐づく測定値一覧を取得する。
    async fn find_by_step(&self, step_id: Uuid) -> Result<Vec<Measurement>, DomainError> {
        let rows = sqlx::query_as::<_, MeasurementRow>(
            r#"
            SELECT
                measurement_id, work_execution_id, step_id,
                value, unit, nominal, upper_limit, lower_limit, cp, cpk
            FROM measurements
            WHERE step_id = $1
            ORDER BY measurement_id ASC
            "#,
        )
        .bind(step_id)
        .fetch_all(&self.pool)
        .await
        .map_err(crate::error::map_sqlx)?;

        Ok(rows.into_iter().map(Measurement::from).collect())
    }
}
