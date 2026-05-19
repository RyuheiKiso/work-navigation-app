// PgCaseLockRepository — TBL-051 case_locks の sqlx 実装（ADR-009）
// マルチデバイス排他原則: 1 case_id = 1 端末を保証する。
// app_event_insert ロールに INSERT/UPDATE/DELETE を許可する例外制御テーブル。
// SQLX_PREPARE_REQUIRED: cargo sqlx prepare を実行してからビルド可能

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use wnav_domain::{
    error::DomainError,
    model::case_lock::{CaseLock, LockStatus},
    repository::CaseLockRepository,
};

use crate::row_types::CaseLockRow;

/// TBL-051 case_locks のリポジトリ実装。
pub struct PgCaseLockRepository {
    pool: PgPool,
}

impl PgCaseLockRepository {
    /// コネクションプールを受け取ってリポジトリを構築する。
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

/// CaseLockRow から CaseLock ドメインモデルへの変換。
impl TryFrom<CaseLockRow> for CaseLock {
    type Error = DomainError;

    fn try_from(row: CaseLockRow) -> Result<Self, Self::Error> {
        let status = parse_lock_status(&row.status)?;
        Ok(Self {
            case_id: row.case_id,
            terminal_id: row.terminal_id,
            locked_by: row.locked_by,
            locked_at: row.locked_at,
            heartbeat_at: row.heartbeat_at,
            status,
        })
    }
}

/// DB ロックステータス文字列を LockStatus 列挙型に変換する。
fn parse_lock_status(s: &str) -> Result<LockStatus, DomainError> {
    match s {
        "ACTIVE" => Ok(LockStatus::Active),
        "EXPIRED" => Ok(LockStatus::Expired),
        other => Err(DomainError::Internal(format!("不明な LockStatus: {other}"))),
    }
}

#[async_trait]
impl CaseLockRepository for PgCaseLockRepository {
    /// Case の占有を取得する。
    /// 他端末が ACTIVE 占有中の場合は CaseLocked エラーを返す（ERR-BIZ-026）。
    async fn acquire(
        &self,
        case_id: Uuid,
        terminal_id: Uuid,
        user_id: Uuid,
    ) -> Result<CaseLock, DomainError> {
        // INSERT OR UPDATE: 既存 ACTIVE 占有があれば競合を検出する
        // UPSERT で同一端末なら heartbeat のみ更新し、別端末なら INSERT 失敗とする
        let row = sqlx::query_as::<_, CaseLockRow>(
            r#"
            INSERT INTO case_locks (case_id, terminal_id, locked_by, locked_at, heartbeat_at, status)
            VALUES ($1, $2, $3, NOW(), NOW(), 'ACTIVE')
            ON CONFLICT (case_id)
            DO UPDATE SET
                terminal_id  = EXCLUDED.terminal_id,
                locked_by    = EXCLUDED.locked_by,
                locked_at    = NOW(),
                heartbeat_at = NOW(),
                status       = 'ACTIVE'
            WHERE case_locks.status = 'EXPIRED'
            RETURNING case_id, terminal_id, locked_by, locked_at, heartbeat_at, status
            "#,
        )
        .bind(case_id)
        .bind(terminal_id)
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(crate::error::map_sqlx)?;

        match row {
            Some(r) => CaseLock::try_from(r),
            None => {
                // ACTIVE ロックが存在するため占有できなかった。占有端末 ID を返す。
                let existing: Option<CaseLockRow> = sqlx::query_as::<_, CaseLockRow>(
                    r#"
                    SELECT case_id, terminal_id, locked_by, locked_at, heartbeat_at, status
                    FROM case_locks
                    WHERE case_id = $1 AND status = 'ACTIVE'
                    "#,
                )
                .bind(case_id)
                .fetch_optional(&self.pool)
                .await
                .map_err(crate::error::map_sqlx)?;

                if let Some(lock) = existing {
                    Err(DomainError::CaseLocked {
                        locked_by_terminal: lock.terminal_id,
                    })
                } else {
                    Err(DomainError::Internal(
                        "case_lock 取得に失敗しました".to_string(),
                    ))
                }
            }
        }
    }

    /// ハートビートを更新する（60 秒ごとに呼び出す）。
    async fn heartbeat(&self, case_id: Uuid, terminal_id: Uuid) -> Result<(), DomainError> {
        sqlx::query(
            r#"
            UPDATE case_locks
            SET heartbeat_at = NOW()
            WHERE case_id = $1 AND terminal_id = $2 AND status = 'ACTIVE'
            "#,
        )
        .bind(case_id)
        .bind(terminal_id)
        .execute(&self.pool)
        .await
        .map_err(crate::error::map_sqlx)?;

        Ok(())
    }

    /// Case の占有を解放する（正常終了・中断時）。
    async fn release(&self, case_id: Uuid, terminal_id: Uuid) -> Result<(), DomainError> {
        sqlx::query(
            r#"
            DELETE FROM case_locks
            WHERE case_id = $1 AND terminal_id = $2
            "#,
        )
        .bind(case_id)
        .bind(terminal_id)
        .execute(&self.pool)
        .await
        .map_err(crate::error::map_sqlx)?;

        Ok(())
    }

    /// アクティブな占有ロックを取得する。
    async fn find_active(&self, case_id: Uuid) -> Result<Option<CaseLock>, DomainError> {
        let row = sqlx::query_as::<_, CaseLockRow>(
            r#"
            SELECT case_id, terminal_id, locked_by, locked_at, heartbeat_at, status
            FROM case_locks
            WHERE case_id = $1 AND status = 'ACTIVE'
            "#,
        )
        .bind(case_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(crate::error::map_sqlx)?;

        row.map(CaseLock::try_from).transpose()
    }

    /// EXPIRED 閾値を超えたロックを検索する（バッチジョブ BAT-013 用）。
    async fn find_expired(&self, threshold: DateTime<Utc>) -> Result<Vec<CaseLock>, DomainError> {
        let rows = sqlx::query_as::<_, CaseLockRow>(
            r#"
            SELECT case_id, terminal_id, locked_by, locked_at, heartbeat_at, status
            FROM case_locks
            WHERE status = 'ACTIVE' AND heartbeat_at < $1
            "#,
        )
        .bind(threshold)
        .fetch_all(&self.pool)
        .await
        .map_err(crate::error::map_sqlx)?;

        rows.into_iter().map(CaseLock::try_from).collect()
    }
}
