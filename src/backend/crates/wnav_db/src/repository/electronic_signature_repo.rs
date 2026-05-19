// PgElectronicSignatureRepository — TBL-002 electronic_signs の sqlx 実装
// 電子サイン必須ステップ（BR-BUS-004）の記録と検証状態管理を担う。
// SQLX_PREPARE_REQUIRED: cargo sqlx prepare を実行してからビルド可能

use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use wnav_domain::{
    error::DomainError, model::electronic_signature::ElectronicSignature,
    repository::ElectronicSignatureRepository,
};

use crate::row_types::ElectronicSignatureRow;

/// TBL-002 electronic_signs のリポジトリ実装。
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
/// DB カラム名（signed_content_hash, context_id）をドメイン語彙に変換する。
impl From<ElectronicSignatureRow> for ElectronicSignature {
    fn from(row: ElectronicSignatureRow) -> Self {
        Self {
            sign_id: row.sign_id,
            // electronic_signs.context_id をドメインの work_execution_id として扱う
            work_execution_id: row.context_id,
            // step_id は DDL 上 NULL 許容だがドメインは必須。NULL の場合は nil UUID を使用する
            step_id: row.step_id.unwrap_or_default(),
            signer_id: row.signer_id,
            // electronic_signs.signed_content_hash をドメインの signature_data として扱う
            signature_data: row.signed_content_hash,
            signed_at: row.signed_at,
            // electronic_signs に verified カラムは存在しないため常に false で初期化する
            verified: false,
        }
    }
}

#[async_trait]
impl ElectronicSignatureRepository for PgElectronicSignatureRepository {
    /// 電子サインを electronic_signs に INSERT する。
    /// context_type は 'step_sign' で固定し、context_id に work_execution_id を格納する。
    async fn insert(&self, signature: ElectronicSignature) -> Result<(), DomainError> {
        sqlx::query(
            r#"
            INSERT INTO electronic_signs (
                sign_id, context_id, context_type, step_id,
                signer_id, signed_content_hash, signed_at
            )
            VALUES ($1, $2, 'step_sign', $3, $4, $5, $6)
            "#,
        )
        .bind(signature.sign_id)
        .bind(signature.work_execution_id)
        .bind(signature.step_id)
        .bind(signature.signer_id)
        .bind(signature.signature_data)
        .bind(signature.signed_at)
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
                sign_id, context_id, step_id,
                signer_id, signed_content_hash, signed_at
            FROM electronic_signs
            WHERE sign_id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(crate::error::map_sqlx)?;

        Ok(row.map(ElectronicSignature::from))
    }

    /// イベント ID（context_id）に紐づく電子サイン一覧を取得する。
    async fn find_by_event(&self, event_id: Uuid) -> Result<Vec<ElectronicSignature>, DomainError> {
        let rows = sqlx::query_as::<_, ElectronicSignatureRow>(
            r#"
            SELECT
                sign_id, context_id, step_id,
                signer_id, signed_content_hash, signed_at
            FROM electronic_signs
            WHERE context_id = $1
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
