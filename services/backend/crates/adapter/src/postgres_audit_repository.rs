//! PostgreSQL 監査ログ書込み
//!
//! 対応 §: ロードマップ §11.4.1 INV-07 §10.5 §20.2

// sqlx
use sqlx::PgPool;
// chrono
use chrono::{DateTime, Utc};
// 既存エラー
use crate::postgres_repository::PostgresRepositoryError;

/// 監査エントリ（書込み専用、追記不変）
#[derive(Debug, Clone)]
pub struct AuditEntry {
    /// 主体（user_id または device_id）
    pub actor_id: String,
    /// 操作種別（"login" / "start_task" / "complete_task" / "publish_flow" 等）
    pub action: String,
    /// 対象識別子（task_id / flow_id / user_id 等）
    pub target_id: Option<String>,
    /// 端末時刻（UTC、§20.2）
    pub terminal_time: Option<DateTime<Utc>>,
    /// payload（JSON 文字列）
    pub payload: Option<String>,
}

/// 監査ログリポジトリ
#[derive(Clone)]
pub struct PostgresAuditRepository {
    /// 接続プール
    pool: PgPool,
}

impl PostgresAuditRepository {
    /// プールから構築する
    #[must_use]
    pub const fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// エントリを追記する（INV-07）
    pub async fn append(&self, entry: &AuditEntry) -> Result<(), PostgresRepositoryError> {
        sqlx::query(
            "INSERT INTO audit_log (actor_id, action, target_id, terminal_time, payload) \
             VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(&entry.actor_id)
        .bind(&entry.action)
        .bind(entry.target_id.as_deref())
        .bind(entry.terminal_time)
        .bind(entry.payload.as_deref())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// 直近 N 件の監査ログを取得する（監査画面 UI 用、§11.4.1）
    pub async fn list_recent(
        &self,
        limit: i64,
    ) -> Result<Vec<AuditRow>, PostgresRepositoryError> {
        let rows: Vec<(uuid::Uuid, String, String, Option<String>, Option<DateTime<Utc>>, DateTime<Utc>, Option<String>)> =
            sqlx::query_as(
                "SELECT id, actor_id, action, target_id, terminal_time, server_time, payload \
                 FROM audit_log ORDER BY server_time DESC LIMIT $1",
            )
            .bind(limit)
            .fetch_all(&self.pool)
            .await?;
        Ok(rows
            .into_iter()
            .map(|(id, actor, action, target, ttime, stime, payload)| AuditRow {
                id: id.to_string(),
                actor_id: actor,
                action,
                target_id: target,
                terminal_time: ttime,
                server_time: stime,
                payload,
            })
            .collect())
    }
}

/// 監査ログ 1 行（読取側 DTO）
#[derive(Debug, Clone)]
pub struct AuditRow {
    pub id: String,
    pub actor_id: String,
    pub action: String,
    pub target_id: Option<String>,
    pub terminal_time: Option<DateTime<Utc>>,
    pub server_time: DateTime<Utc>,
    pub payload: Option<String>,
}
