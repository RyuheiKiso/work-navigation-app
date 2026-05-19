// PgElectronicSignatureRepository — TBL-011 electronic_signatures の sqlx 実装
// 電子サイン必須ステップ（BR-BUS-004）の記録と検証状態管理を担う。
// SQLX_PREPARE_REQUIRED: cargo sqlx prepare を実行してからビルド可能

use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use wnav_domain::{
    error::DomainError,
    model::electronic_signature::ElectronicSignature,
    repository::ElectronicSignatureRepository,
};

use crate::row_types::ElectronicSignatureRow;

/// TBL-011 electronic_signatures のリポジトリ実装。
pub struct PgElectronicSignatureRepository {
    pool: PgPool,
}

impl PgElectronicSignatureRepository {
    /// コネクションプールを受け取ってリポジトリを構築する。
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

/// ElectronicSignatureRow から ElectronicSignature ドメインモデルへの変換。
impl From<ElectronicSignatureRow> for ElectronicSignature {
    fn from(row: ElectronicSignatureRow) -> Self {
        Self {
            sign_id: row.sign_id,
            work_execution_id: row.work_execution_id,
            step_id: row.step_id,
            signer_id: row.signer_id,
            signature_data: row.signature_data,
            signed_at: row.signed_at,
            verified: row.verified,
        }
    }
}

#[async_trait]
impl ElectronicSignatureRepository for PgElectronicSignatureRepository {
    /// 電子サインを INSERT する。
    async fn insert(&self, signature: ElectronicSignature) -> Result<(), DomainError> {
        sqlx::query(
            r#"
            INSERT INTO electronic_signatures (
                sign_id, work_execution_id, step_id,
                signer_id, signature_data, signed_at, verified
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
        )
        .bind(signature.sign_id)
        .bind(signature.work_execution_id)
        .bind(signature.step_id)
        .bind(signature.signer_id)
        .bind(signature.signature_data)
        .bind(signature.signed_at)
        .bind(signature.verified)
        .execute(&self.pool)
        .await
        .map_err(crate::error::map_sqlx)?;

        Ok(())
    }

    /// ID で電子サインを検索する。
    async fn find_by_id(&self, id: Uuid) -> Result<Option<ElectronicSignature>, DomainError> {
        let row = sqlx::query_as::<_, ElectronicSignatureRow>(
            r#"
            SELECT
                sign_id, work_execution_id, step_id,
                signer_id, signature_data, signed_at, verified
            FROM electronic_signatures
            WHERE sign_id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(crate::error::map_sqlx)?;

        Ok(row.map(ElectronicSignature::from))
    }

    /// イベント ID に紐づく電子サイン一覧を取得する。
    async fn find_by_event(
        &self,
        event_id: Uuid,
    ) -> Result<Vec<ElectronicSignature>, DomainError> {
        let rows = sqlx::query_as::<_, ElectronicSignatureRow>(
            r#"
            SELECT
                sign_id, work_execution_id, step_id,
                signer_id, signature_data, signed_at, verified
            FROM electronic_signatures
            WHERE event_id = $1
            ORDER BY signed_at ASC
            "#,
        )
        .bind(event_id)
        .fetch_all(&self.pool)
        .await
        .map_err(crate::error::map_sqlx)?;

        Ok(rows.into_iter().map(ElectronicSignature::from).collect())
    }
}
