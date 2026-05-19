// PgHashChainBlockRepository — TBL-031 hash_chain_blocks の sqlx 実装（Append-only）
// SHA-256 ハッシュチェーンブロックの永続化と BAT-001 週次検証のための検索を担う。
// INSERT のみを提供し、UPDATE・DELETE は提供しない（Append-only 原則）。
// SQLX_PREPARE_REQUIRED: cargo sqlx prepare を実行してからビルド可能

use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use wnav_domain::{
    error::DomainError,
    repository::{HashChainBlock, HashChainBlockRepository},
};

use crate::row_types::HashChainBlockRow;

/// TBL-031 hash_chain_blocks の Append-only リポジトリ実装。
pub struct PgHashChainBlockRepository {
    pool: PgPool,
}

impl PgHashChainBlockRepository {
    /// コネクションプールを受け取ってリポジトリを構築する。
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

/// HashChainBlockRow から HashChainBlock ドメイン型への変換。
/// BYTEA 32 バイトを固定長配列に変換する。
impl TryFrom<HashChainBlockRow> for HashChainBlock {
    type Error = DomainError;

    fn try_from(row: HashChainBlockRow) -> Result<Self, Self::Error> {
        let prev_block_hash: [u8; 32] = row.prev_block_hash.try_into().map_err(|_| {
            DomainError::Internal("prev_block_hash が 32 バイトではありません".to_string())
        })?;
        let content_hash: [u8; 32] = row.content_hash.try_into().map_err(|_| {
            DomainError::Internal("content_hash が 32 バイトではありません".to_string())
        })?;
        let block_hash: [u8; 32] = row.block_hash.try_into().map_err(|_| {
            DomainError::Internal("block_hash が 32 バイトではありません".to_string())
        })?;

        Ok(Self {
            block_id: row.block_id,
            case_id: row.case_id,
            sequence_number: row.sequence_number,
            prev_block_hash,
            content_hash,
            block_hash,
            created_at: row.created_at,
        })
    }
}

#[async_trait]
impl HashChainBlockRepository for PgHashChainBlockRepository {
    /// Append-only: ハッシュチェーンブロックを INSERT する（作業イベントと同一 TX）。
    /// UPDATE・DELETE は Append-only 原則により提供しない。
    async fn insert(&self, block: HashChainBlock) -> Result<(), DomainError> {
        sqlx::query(
            r#"
            INSERT INTO hash_chain_blocks (
                block_id, case_id, sequence_number,
                prev_block_hash, content_hash, block_hash,
                created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
        )
        .bind(block.block_id)
        .bind(block.case_id)
        .bind(block.sequence_number)
        .bind(block.prev_block_hash.as_ref())
        .bind(block.content_hash.as_ref())
        .bind(block.block_hash.as_ref())
        .bind(block.created_at)
        .execute(&self.pool)
        .await
        .map_err(crate::error::map_sqlx)?;

        Ok(())
    }

    /// case_id に紐づく全ブロックを sequence_number 昇順で取得する（BAT-001 週次検証用）。
    async fn list_by_case(&self, case_id: Uuid) -> Result<Vec<HashChainBlock>, DomainError> {
        let rows = sqlx::query_as::<_, HashChainBlockRow>(
            r#"
            SELECT
                block_id, case_id, sequence_number,
                prev_block_hash, content_hash, block_hash,
                created_at
            FROM hash_chain_blocks
            WHERE case_id = $1
            ORDER BY sequence_number ASC
            "#,
        )
        .bind(case_id)
        .fetch_all(&self.pool)
        .await
        .map_err(crate::error::map_sqlx)?;

        rows.into_iter()
            .map(HashChainBlock::try_from)
            .collect::<Result<Vec<_>, _>>()
    }

    /// case_id の最新ブロックを取得する（次ブロック生成時の prev_hash 取得）。
    async fn find_latest(&self, case_id: Uuid) -> Result<Option<HashChainBlock>, DomainError> {
        let row = sqlx::query_as::<_, HashChainBlockRow>(
            r#"
            SELECT
                block_id, case_id, sequence_number,
                prev_block_hash, content_hash, block_hash,
                created_at
            FROM hash_chain_blocks
            WHERE case_id = $1
            ORDER BY sequence_number DESC
            LIMIT 1
            "#,
        )
        .bind(case_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(crate::error::map_sqlx)?;

        row.map(HashChainBlock::try_from).transpose()
    }
}
