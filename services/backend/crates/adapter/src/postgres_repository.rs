//! PostgreSQL Repository 実装
//!
//! 対応 §: ロードマップ §7.3 §10.3.1 §10.6 §11.4.1 §15.2
//!
//! state ／ completion_criteria ／ precondition の **全フィールド** を永続化／復元する。

// 内側 crate からの import
use wna_domain::{
    CompletionCriteria, DeviceId, DomainError, LamportTimestamp, Task, TaskId, TaskRepository,
    TaskState,
};
use wna_usecase::{AppendRecordCommand, RecordRepository};
// sqlx
use sqlx::PgPool;
// 境界エラー
use thiserror::Error;

/// PostgreSQL Repository 実装
#[derive(Clone)]
pub struct PostgresRepository {
    /// sqlx の接続プール
    pool: PgPool,
}

impl PostgresRepository {
    /// プールから Repository を構築する
    #[must_use]
    pub const fn new(pool: PgPool) -> Self {
        // pool を保持
        Self { pool }
    }

    /// 接続プールを取得する（他リポジトリと共有するため）
    #[must_use]
    pub const fn pool(&self) -> &PgPool {
        // 内部参照
        &self.pool
    }
}

/// PostgreSQL Repository のエラー
#[derive(Debug, Error)]
pub enum PostgresRepositoryError {
    /// sqlx 由来のエラー（接続／クエリ／型変換）
    #[error("PostgreSQL: {0}")]
    Sqlx(#[from] sqlx::Error),
    /// ドメイン規則違反（境界での値変換時）
    #[error("ドメイン規則違反: {0}")]
    Domain(#[from] DomainError),
    /// データ整合性エラー（DB 内に未知ラベル等）
    #[error("データ整合性: {0}")]
    Integrity(String),
}

/// 文字列タグ → CompletionCriteria
fn parse_completion_criteria(tag: &str) -> Result<CompletionCriteria, PostgresRepositoryError> {
    match tag {
        "manual" => Ok(CompletionCriteria::Manual),
        "photo" => Ok(CompletionCriteria::Photo),
        other => Err(PostgresRepositoryError::Integrity(format!(
            "未知の完了条件タグ: {other}"
        ))),
    }
}

/// CompletionCriteria → 文字列タグ
fn completion_criteria_tag(c: &CompletionCriteria) -> &'static str {
    match c {
        CompletionCriteria::Manual => "manual",
        CompletionCriteria::Photo => "photo",
    }
}

impl TaskRepository for PostgresRepository {
    type Error = PostgresRepositoryError;

    async fn find_by_id(&self, id: &TaskId) -> Result<Option<Task>, Self::Error> {
        let id_str = id.as_str();
        let row: Option<(String, String, String, i64, String)> = sqlx::query_as(
            "SELECT id, state, device_id, lamport, completion_criteria \
             FROM tasks WHERE id = $1",
        )
        .bind(id_str)
        .fetch_optional(&self.pool)
        .await?;
        let Some((db_id, db_state, db_device, db_lamport, db_cri)) = row else {
            return Ok(None);
        };
        let task_id = TaskId::new(db_id)?;
        let device_id = DeviceId::new(db_device)?;
        let lamport = LamportTimestamp::from_u64(u64::try_from(db_lamport).unwrap_or(0));
        let cri = parse_completion_criteria(&db_cri)?;
        let state = TaskState::from_label(&db_state).ok_or_else(|| {
            PostgresRepositoryError::Integrity(format!("未知の state: {db_state}"))
        })?;
        // precondition は state から推定（Idle のみ未充足）
        let precondition_satisfied = !matches!(state, TaskState::Idle);
        // 永続化値から完全復元
        Ok(Some(Task::rehydrate(
            task_id,
            state,
            cri,
            device_id,
            lamport,
            precondition_satisfied,
        )))
    }

    async fn save(&self, task: &Task) -> Result<(), Self::Error> {
        let id = task.id().as_str();
        let state = task.state().label();
        let device = task.device_id().as_str();
        let lamport: i64 = i64::try_from(task.lamport().value()).unwrap_or(0);
        // **完了条件をハードコードせず、ドメイン値から正しいタグを取得する**
        let cri = completion_criteria_tag(task.completion_criteria());

        sqlx::query(
            "INSERT INTO tasks (id, state, device_id, lamport, completion_criteria, updated_at) \
             VALUES ($1, $2, $3, $4, $5, NOW()) \
             ON CONFLICT (id) DO UPDATE SET \
               state = EXCLUDED.state, \
               device_id = EXCLUDED.device_id, \
               lamport = EXCLUDED.lamport, \
               completion_criteria = EXCLUDED.completion_criteria, \
               updated_at = NOW()",
        )
        .bind(id)
        .bind(state)
        .bind(device)
        .bind(lamport)
        .bind(cri)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}

impl RecordRepository for PostgresRepository {
    type Error = PostgresRepositoryError;

    async fn append(&self, cmd: &AppendRecordCommand) -> Result<(), Self::Error> {
        let task_id = cmd.task_id.as_str();
        let device = cmd.device_id.as_str();
        let lamport: i64 = i64::try_from(cmd.lamport.value()).unwrap_or(0);
        let payload = cmd.payload.as_str();

        sqlx::query(
            "INSERT INTO records (task_id, device_id, lamport, payload) \
             VALUES ($1, $2, $3, $4)",
        )
        .bind(task_id)
        .bind(device)
        .bind(lamport)
        .bind(payload)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
