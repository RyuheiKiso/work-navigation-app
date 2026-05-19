// PgIdempotencyRepository — TBL-035 idempotency_keys の sqlx 実装
// Idempotent API 原則の実装（TTL 24h）。
// case_locks と同様に app_event_insert ロールに INSERT/UPDATE/DELETE を許可する例外制御テーブル。
// SQLX_PREPARE_REQUIRED: cargo sqlx prepare を実行してからビルド可能

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

use wnav_domain::{
    error::DomainError,
    repository::{IdempotencyRecord, IdempotencyRepository},
};

use crate::row_types::IdempotencyRow;

/// TBL-035 idempotency_keys のリポジトリ実装。
pub struct PgIdempotencyRepository {
    pool: PgPool,
}

impl PgIdempotencyRepository {
    /// コネクションプールを受け取ってリポジトリを構築する。
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

/// IdempotencyRow から IdempotencyRecord ドメイン型への変換。
impl From<IdempotencyRow> for IdempotencyRecord {
    fn from(row: IdempotencyRow) -> Self {
        Self {
            key: row.idempotency_key,
            response_body: row.response_body,
            expires_at: row.expires_at,
        }
    }
}

#[async_trait]
impl IdempotencyRepository for PgIdempotencyRepository {
    /// 冪等性キーで既存レコードを検索する。
    async fn find_by_key(&self, key: Uuid) -> Result<Option<IdempotencyRecord>, DomainError> {
        let row = sqlx::query_as::<_, IdempotencyRow>(
            r#"
            SELECT idempotency_key, response_body, expires_at
            FROM idempotency_keys
            WHERE idempotency_key = $1 AND expires_at > NOW()
            "#,
        )
        .bind(key)
        .fetch_optional(&self.pool)
        .await
        .map_err(crate::error::map_sqlx)?;

        Ok(row.map(IdempotencyRecord::from))
    }

    /// 冪等性キーとレスポンスを INSERT する。
    /// 同一キーが既に存在する場合は DuplicateExternalKey エラーを返す。
    async fn insert(
        &self,
        key: Uuid,
        response_body: Value,
        expires_at: DateTime<Utc>,
    ) -> Result<(), DomainError> {
        let result = sqlx::query(
            r#"
            INSERT INTO idempotency_keys (idempotency_key, response_body, expires_at)
            VALUES ($1, $2, $3)
            ON CONFLICT (idempotency_key) DO NOTHING
            "#,
        )
        .bind(key)
        .bind(response_body)
        .bind(expires_at)
        .execute(&self.pool)
        .await
        .map_err(crate::error::map_sqlx)?;

        // ON CONFLICT DO NOTHING で 0 行 INSERT なら重複キー
        if result.rows_affected() == 0 {
            return Err(DomainError::DuplicateExternalKey {
                key: key.to_string(),
            });
        }

        Ok(())
    }

    /// TTL 期限切れのレコードを削除する（バッチジョブ用）。
    async fn cleanup_expired(&self) -> Result<u64, DomainError> {
        let result = sqlx::query(
            r#"
            DELETE FROM idempotency_keys
            WHERE expires_at <= NOW()
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(crate::error::map_sqlx)?;

        Ok(result.rows_affected())
    }
}
