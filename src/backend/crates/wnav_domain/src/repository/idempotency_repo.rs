// 冪等性キーリポジトリ Trait
// API の冪等性保証のための Trait（Idempotent API 原則）。
// 同一キーの再送は保存済みレスポンスを返す（TTL 24h）。

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde_json::Value;
use uuid::Uuid;

use crate::error::DomainError;

/// 冪等性キーリポジトリ Trait（Idempotent API 原則実装）。
/// `idempotency_keys` テーブルは app_event_insert ロールに INSERT/UPDATE/DELETE を許可する
/// 例外制御テーブルである（src/CLAUDE.md）。
#[async_trait]
pub trait IdempotencyRepository: Send + Sync + 'static {
    /// 冪等性キーで既存レコードを検索する。
    /// 存在する場合は保存済みレスポンス、存在しない場合は None を返す。
    async fn find_by_key(&self, key: Uuid) -> Result<Option<IdempotencyRecord>, DomainError>;

    /// 冪等性キーとレスポンスを INSERT する。
    /// 同一キーが既に存在する場合は DuplicateExternalKey エラーを返す。
    async fn insert(
        &self,
        key: Uuid,
        response_body: Value,
        expires_at: DateTime<Utc>,
    ) -> Result<(), DomainError>;

    /// TTL 期限切れのレコードを削除する（バッチジョブ用）。
    async fn cleanup_expired(&self) -> Result<u64, DomainError>;
}

/// 冪等性キーレコード。
#[derive(Debug, Clone)]
pub struct IdempotencyRecord {
    /// 冪等性キー（UUID v4）
    pub key: Uuid,
    /// 保存済みレスポンスボディ（JSONB）
    pub response_body: Value,
    /// TTL 期限
    pub expires_at: DateTime<Utc>,
}
