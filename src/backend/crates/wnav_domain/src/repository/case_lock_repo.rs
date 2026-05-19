// Case 端末占有ロックリポジトリ Trait
// マルチデバイス排他原則のためのリポジトリ Trait。
// case_locks テーブルは app_event_insert ロールに INSERT/UPDATE/DELETE を許可する例外テーブル。

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::error::DomainError;
use crate::model::case_lock::CaseLock;

/// Case 端末占有ロックリポジトリ Trait。
/// case_locks テーブルは Append-only 原則の例外制御テーブル。
/// heartbeat 更新と解放 DELETE が必要なため、INSERT/UPDATE/DELETE を許可する。
#[async_trait]
pub trait CaseLockRepository: Send + Sync + 'static {
    /// Case の占有を取得する（他端末が占有中の場合は CaseLocked エラー）。
    async fn acquire(
        &self,
        case_id: Uuid,
        terminal_id: Uuid,
        user_id: Uuid,
    ) -> Result<CaseLock, DomainError>;

    /// ハートビートを更新する（60 秒ごとに呼び出す）。
    async fn heartbeat(&self, case_id: Uuid, terminal_id: Uuid) -> Result<(), DomainError>;

    /// Case の占有を解放する（正常終了・中断時）。
    async fn release(&self, case_id: Uuid, terminal_id: Uuid) -> Result<(), DomainError>;

    /// アクティブな占有ロックを取得する。
    async fn find_active(&self, case_id: Uuid) -> Result<Option<CaseLock>, DomainError>;

    /// EXPIRED 閾値を超えたロックを検索する（バッチジョブ BAT-013 用）。
    async fn find_expired(&self, threshold: DateTime<Utc>) -> Result<Vec<CaseLock>, DomainError>;
}
