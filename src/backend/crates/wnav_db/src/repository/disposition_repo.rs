// PgDispositionRepository — TBL-047/048 の sqlx 実装
// 処置判定（Disposition）の Two-Person Integrity（FR-AU-007）と
// ADR-011 ハッシュチェーン計算を含む。
// SQLX_PREPARE_REQUIRED: cargo sqlx prepare を実行してからビルド可能

use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use wnav_domain::{
    error::DomainError,
    model::disposition::{Disposition, DispositionApproval, DispositionType},
    repository::DispositionRepository,
};

use crate::row_types::DispositionRow;

/// TBL-047 dispositions のリポジトリ実装。
pub struct PgDispositionRepository {
    pool: PgPool,
}

impl PgDispositionRepository {
    /// コネクションプールを受け取ってリポジトリを構築する。
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

/// DispositionRow から Disposition ドメインモデルへの変換。
impl TryFrom<DispositionRow> for Disposition {
    type Error = DomainError;

    fn try_from(row: DispositionRow) -> Result<Self, Self::Error> {
        let disposition_type = parse_disposition_type(&row.disposition_type)?;
        Ok(Self {
            disposition_id: row.disposition_id,
            lot_id: row.lot_id,
            rework_id: row.rework_id,
            disposition_type,
            approved_by_1: row.approved_by_1,
            approved_by_2: row.approved_by_2,
            approved_at: row.approved_at,
            prev_hash: row.prev_hash,
            content_hash: row.content_hash,
            chain_hash: row.chain_hash,
        })
    }
}

/// DB 処置種別文字列を DispositionType 列挙型に変換する。
fn parse_disposition_type(s: &str) -> Result<DispositionType, DomainError> {
    match s {
        "SCRAP" => Ok(DispositionType::Scrap),
        "RETURN" => Ok(DispositionType::Return),
        "CONCESSION" => Ok(DispositionType::Concession),
        "REWORK" => Ok(DispositionType::Rework),
        "ACCEPT" => Ok(DispositionType::Accept),
        other => Err(DomainError::Internal(format!(
            "不明な DispositionType: {other}"
        ))),
    }
}

/// DispositionType を DB 格納文字列に変換する。
fn disposition_type_to_str(t: &DispositionType) -> &'static str {
    match t {
        DispositionType::Scrap => "SCRAP",
        DispositionType::Return => "RETURN",
        DispositionType::Concession => "CONCESSION",
        DispositionType::Rework => "REWORK",
        DispositionType::Accept => "ACCEPT",
    }
}

#[async_trait]
impl DispositionRepository for PgDispositionRepository {
    /// 処置判定を INSERT する（ADR-011: ハッシュチェーン計算済みの値を受け取る）。
    async fn insert(&self, disposition: Disposition) -> Result<(), DomainError> {
        sqlx::query(
            r#"
            INSERT INTO dispositions (
                disposition_id, lot_id, rework_id,
                disposition_type, approved_by_1, approved_by_2, approved_at,
                prev_hash, content_hash, chain_hash
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#,
        )
        .bind(disposition.disposition_id)
        .bind(disposition.lot_id)
        .bind(disposition.rework_id)
        .bind(disposition_type_to_str(&disposition.disposition_type))
        .bind(disposition.approved_by_1)
        .bind(disposition.approved_by_2)
        .bind(disposition.approved_at)
        .bind(disposition.prev_hash)
        .bind(disposition.content_hash)
        .bind(disposition.chain_hash)
        .execute(&self.pool)
        .await
        .map_err(crate::error::map_sqlx)?;

        Ok(())
    }

    /// ID で処置判定を検索する。
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Disposition>, DomainError> {
        let row = sqlx::query_as::<_, DispositionRow>(
            r#"
            SELECT
                disposition_id, lot_id, rework_id,
                disposition_type, approved_by_1, approved_by_2, approved_at,
                prev_hash, content_hash, chain_hash
            FROM dispositions
            WHERE disposition_id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(crate::error::map_sqlx)?;

        row.map(Disposition::try_from).transpose()
    }

    /// 処置判定に承認を追加する（Two-Person Integrity: FR-AU-007）。
    /// sequence=1 なら approved_by_1 を、sequence=2 なら approved_by_2 を設定する。
    /// 2 名の承認が揃ったときに approved_at を設定する。
    async fn add_approval(&self, approval: DispositionApproval) -> Result<(), DomainError> {
        // 現在の承認状態を確認して同一人物による 2 回承認を防ぐ
        let current: Option<DispositionRow> = sqlx::query_as::<_, DispositionRow>(
            r#"
            SELECT
                disposition_id, lot_id, rework_id,
                disposition_type, approved_by_1, approved_by_2, approved_at,
                prev_hash, content_hash, chain_hash
            FROM dispositions
            WHERE disposition_id = $1
            "#,
        )
        .bind(approval.disposition_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(crate::error::map_sqlx)?;

        let row = current.ok_or(DomainError::NotFound)?;

        // 両者が同一人物でないことを確認する（Two-Person Integrity チェック）
        if approval.sequence == 2 {
            if let Some(first) = row.approved_by_1 {
                if first == approval.approver_id {
                    return Err(DomainError::ReworkRequiresTwoApprovers);
                }
            }
        }

        // シーケンスに応じて承認者を設定し、2 名揃ったら approved_at を設定する
        sqlx::query(
            r#"
            UPDATE dispositions
            SET
                approved_by_1 = CASE WHEN $1 = 1 THEN $2 ELSE approved_by_1 END,
                approved_by_2 = CASE WHEN $1 = 2 THEN $2 ELSE approved_by_2 END,
                approved_at   = CASE
                    WHEN ($1 = 1 AND approved_by_2 IS NOT NULL)
                      OR ($1 = 2 AND approved_by_1 IS NOT NULL)
                    THEN NOW()
                    ELSE approved_at
                END
            WHERE disposition_id = $3
            "#,
        )
        .bind(i16::from(approval.sequence))
        .bind(approval.approver_id)
        .bind(approval.disposition_id)
        .execute(&self.pool)
        .await
        .map_err(crate::error::map_sqlx)?;

        Ok(())
    }

    /// リワーク ID に紐づく処置判定一覧を取得する。
    async fn find_by_rework(&self, rework_id: Uuid) -> Result<Vec<Disposition>, DomainError> {
        let rows = sqlx::query_as::<_, DispositionRow>(
            r#"
            SELECT
                disposition_id, lot_id, rework_id,
                disposition_type, approved_by_1, approved_by_2, approved_at,
                prev_hash, content_hash, chain_hash
            FROM dispositions
            WHERE rework_id = $1
            ORDER BY disposition_id ASC
            "#,
        )
        .bind(rework_id)
        .fetch_all(&self.pool)
        .await
        .map_err(crate::error::map_sqlx)?;

        rows.into_iter()
            .map(Disposition::try_from)
            .collect::<Result<Vec<_>, _>>()
    }
}
