//! PostgreSQL 順序情報・Idempotency リポジトリ（24h 重複排除窓）
//!
//! 対応 §: ロードマップ §10.3 §10.3.1 §10.3.2 §27 F-005

// ドメイン
use wna_domain::{IdempotencyKey, ProductionOrder};
// ユースケース trait
use wna_usecase::OrderRepository;
// sqlx
use sqlx::PgPool;
// 既存エラー型を流用
use crate::postgres_repository::PostgresRepositoryError;

/// PostgreSQL Idempotency／順序情報リポジトリ
#[derive(Clone)]
pub struct PostgresOrderRepository {
    /// 接続プール
    pool: PgPool,
}

impl PostgresOrderRepository {
    /// プールから構築する
    #[must_use]
    pub const fn new(pool: PgPool) -> Self {
        // pool を保持
        Self { pool }
    }
}

impl OrderRepository for PostgresOrderRepository {
    type Error = PostgresRepositoryError;

    async fn key_seen_within_window(
        &self,
        key: &IdempotencyKey,
    ) -> Result<bool, Self::Error> {
        // 24h 窓内に同キーが存在するか（NOW() - INTERVAL '24 hours' 以降）
        let row: Option<(i64,)> = sqlx::query_as(
            "SELECT COUNT(*)::BIGINT \
             FROM idempotency_keys \
             WHERE key = $1 AND seen_at > NOW() - INTERVAL '24 hours'",
        )
        .bind(key.as_str())
        .fetch_optional(&self.pool)
        .await?;
        // 戻り値解釈
        Ok(row.map(|(c,)| c > 0).unwrap_or(false))
    }

    async fn store(&self, order: &ProductionOrder) -> Result<(), Self::Error> {
        // トランザクションで idempotency_keys と production_orders を一括挿入する想定
        // production_orders テーブルは将来 0003 マイグレーションで追加される（本実装では最低限 idempotency のみ）
        sqlx::query(
            "INSERT INTO idempotency_keys (key, related_resource) VALUES ($1, $2) \
             ON CONFLICT (key) DO UPDATE SET seen_at = NOW(), related_resource = EXCLUDED.related_resource",
        )
        .bind(order.idempotency_key().as_str())
        .bind(order.id().as_str())
        .execute(&self.pool)
        .await?;
        // 正常終了（将来 production_orders テーブルへの INSERT を追加）
        Ok(())
    }
}
