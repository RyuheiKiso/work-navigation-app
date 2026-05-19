// PgCapaRepository — TBL-013 capas の sqlx 実装
// CAPA（是正・予防措置）のフェーズ管理と担当者追跡を担う。
// SQLX_PREPARE_REQUIRED: cargo sqlx prepare を実行してからビルド可能

use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use wnav_domain::{
    error::DomainError,
    model::capa::{Capa, CapaPhase},
    repository::CapaRepository,
};

use crate::row_types::CapaRow;

/// TBL-013 capas のリポジトリ実装。
pub struct PgCapaRepository {
    pool: PgPool,
}

impl PgCapaRepository {
    /// コネクションプールを受け取ってリポジトリを構築する。
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

/// CapaRow から Capa ドメインモデルへの変換。
impl TryFrom<CapaRow> for Capa {
    type Error = DomainError;

    fn try_from(row: CapaRow) -> Result<Self, Self::Error> {
        let phase = parse_capa_phase(&row.phase)?;
        Ok(Self {
            capa_id: row.capa_id,
            andon_id: row.andon_id,
            deviation_id: row.deviation_id,
            phase,
            assignee: row.assignee,
            due_date: row.due_date,
            description: row.description,
            root_cause_json: row.root_cause_json,
            corrective_action: row.corrective_action,
            closed_at: row.closed_at,
        })
    }
}

/// DB フェーズ文字列を CapaPhase 列挙型に変換する。
fn parse_capa_phase(s: &str) -> Result<CapaPhase, DomainError> {
    match s {
        "INVESTIGATION" => Ok(CapaPhase::Investigation),
        "CORRECTIVE" => Ok(CapaPhase::Corrective),
        "PREVENTIVE" => Ok(CapaPhase::Preventive),
        "CLOSED" => Ok(CapaPhase::Closed),
        other => Err(DomainError::Internal(format!("不明な CapaPhase: {other}"))),
    }
}

/// CapaPhase を DB 格納文字列に変換する。
fn phase_to_str(p: &CapaPhase) -> &'static str {
    match p {
        CapaPhase::Investigation => "INVESTIGATION",
        CapaPhase::Corrective => "CORRECTIVE",
        CapaPhase::Preventive => "PREVENTIVE",
        CapaPhase::Closed => "CLOSED",
    }
}

#[async_trait]
impl CapaRepository for PgCapaRepository {
    /// CAPA を INSERT する。
    async fn insert(&self, capa: Capa) -> Result<(), DomainError> {
        sqlx::query(
            r#"
            INSERT INTO capas (
                capa_id, andon_id, deviation_id, phase,
                assignee, due_date, description,
                root_cause_json, corrective_action, closed_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#,
        )
        .bind(capa.capa_id)
        .bind(capa.andon_id)
        .bind(capa.deviation_id)
        .bind(phase_to_str(&capa.phase))
        .bind(capa.assignee)
        .bind(capa.due_date)
        .bind(capa.description)
        .bind(capa.root_cause_json)
        .bind(capa.corrective_action)
        .bind(capa.closed_at)
        .execute(&self.pool)
        .await
        .map_err(crate::error::map_sqlx)?;

        Ok(())
    }

    /// ID で CAPA を検索する。
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Capa>, DomainError> {
        let row = sqlx::query_as::<_, CapaRow>(
            r#"
            SELECT
                capa_id, andon_id, deviation_id, phase,
                assignee, due_date, description,
                root_cause_json, corrective_action, closed_at
            FROM capas
            WHERE capa_id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(crate::error::map_sqlx)?;

        row.map(Capa::try_from).transpose()
    }

    /// CAPA のフェーズを更新する（Investigation → Corrective → Preventive → Closed）。
    async fn update_phase(
        &self,
        id: Uuid,
        new_phase: CapaPhase,
        _updated_by: Uuid,
    ) -> Result<Capa, DomainError> {
        let row = sqlx::query_as::<_, CapaRow>(
            r#"
            UPDATE capas
            SET
                phase = $1,
                closed_at = CASE WHEN $1 = 'CLOSED' THEN NOW() ELSE closed_at END
            WHERE capa_id = $2
            RETURNING
                capa_id, andon_id, deviation_id, phase,
                assignee, due_date, description,
                root_cause_json, corrective_action, closed_at
            "#,
        )
        .bind(phase_to_str(&new_phase))
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(crate::error::map_sqlx)?;

        Capa::try_from(row)
    }
}
