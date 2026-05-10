//! PostgreSQL マスタ／フロー／タスクステップ Repository
//!
//! 対応 §: ロードマップ §10.2.1 §10.3.6 §3.6.3

use sqlx::PgPool;
use chrono::{DateTime, Utc};
use crate::postgres_repository::PostgresRepositoryError;

#[derive(Debug, Clone)]
pub struct MasterRow {
    pub code: String,
    pub name: String,
    pub extra: Option<String>,
}

#[derive(Debug, Clone)]
pub struct TaskStepRow {
    pub id: String,
    pub sequence: i32,
    pub label: String,
    pub completion_criteria: String,
    pub standard_time_seconds: i32,
    pub done: bool,
}

#[derive(Debug, Clone)]
pub struct TaskListItem {
    pub id: String,
    pub title: Option<String>,
    pub state: String,
    pub device_id: String,
    pub responsible_user: Option<String>,
    pub current_step_id: Option<String>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct FlowSummary {
    pub id: String,
    pub version: i32,
    pub name: String,
    pub status: String,
    pub industry: Option<String>,
}

#[derive(Clone)]
pub struct PostgresMasterRepository {
    pool: PgPool,
}

impl PostgresMasterRepository {
    #[must_use]
    pub const fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // ===== 製品マスタ =====
    pub async fn list_products(&self) -> Result<Vec<MasterRow>, PostgresRepositoryError> {
        let rows: Vec<(String, String, Option<String>)> =
            sqlx::query_as("SELECT code, name, industry FROM products ORDER BY code")
                .fetch_all(&self.pool).await?;
        Ok(rows.into_iter().map(|(c, n, i)| MasterRow { code: c, name: n, extra: i }).collect())
    }

    pub async fn upsert_product(
        &self,
        code: &str,
        name: &str,
        industry: Option<&str>,
    ) -> Result<(), PostgresRepositoryError> {
        sqlx::query(
            "INSERT INTO products (code, name, industry) VALUES ($1, $2, $3) \
             ON CONFLICT (code) DO UPDATE SET name = EXCLUDED.name, industry = EXCLUDED.industry, updated_at = NOW()",
        )
        .bind(code).bind(name).bind(industry)
        .execute(&self.pool).await?;
        Ok(())
    }

    pub async fn delete_product(&self, code: &str) -> Result<(), PostgresRepositoryError> {
        sqlx::query("DELETE FROM products WHERE code = $1").bind(code).execute(&self.pool).await?;
        Ok(())
    }

    // ===== 設備マスタ =====
    pub async fn list_equipments(&self) -> Result<Vec<MasterRow>, PostgresRepositoryError> {
        let rows: Vec<(String, String, Option<String>)> =
            sqlx::query_as("SELECT code, name, location FROM equipments ORDER BY code")
                .fetch_all(&self.pool).await?;
        Ok(rows.into_iter().map(|(c, n, l)| MasterRow { code: c, name: n, extra: l }).collect())
    }

    pub async fn upsert_equipment(
        &self, code: &str, name: &str, location: Option<&str>,
    ) -> Result<(), PostgresRepositoryError> {
        sqlx::query(
            "INSERT INTO equipments (code, name, location) VALUES ($1, $2, $3) \
             ON CONFLICT (code) DO UPDATE SET name = EXCLUDED.name, location = EXCLUDED.location, updated_at = NOW()",
        )
        .bind(code).bind(name).bind(location)
        .execute(&self.pool).await?;
        Ok(())
    }

    pub async fn delete_equipment(&self, code: &str) -> Result<(), PostgresRepositoryError> {
        sqlx::query("DELETE FROM equipments WHERE code = $1").bind(code).execute(&self.pool).await?;
        Ok(())
    }

    // ===== 部材マスタ =====
    pub async fn list_parts(&self) -> Result<Vec<MasterRow>, PostgresRepositoryError> {
        let rows: Vec<(String, String, Option<String>)> =
            sqlx::query_as("SELECT code, name, unit FROM parts ORDER BY code")
                .fetch_all(&self.pool).await?;
        Ok(rows.into_iter().map(|(c, n, u)| MasterRow { code: c, name: n, extra: u }).collect())
    }

    pub async fn upsert_part(
        &self, code: &str, name: &str, unit: Option<&str>,
    ) -> Result<(), PostgresRepositoryError> {
        sqlx::query(
            "INSERT INTO parts (code, name, unit) VALUES ($1, $2, $3) \
             ON CONFLICT (code) DO UPDATE SET name = EXCLUDED.name, unit = EXCLUDED.unit, updated_at = NOW()",
        )
        .bind(code).bind(name).bind(unit)
        .execute(&self.pool).await?;
        Ok(())
    }

    pub async fn delete_part(&self, code: &str) -> Result<(), PostgresRepositoryError> {
        sqlx::query("DELETE FROM parts WHERE code = $1").bind(code).execute(&self.pool).await?;
        Ok(())
    }

    // ===== タスクステップ =====
    pub async fn list_steps(
        &self, task_id: &str,
    ) -> Result<Vec<TaskStepRow>, PostgresRepositoryError> {
        let rows: Vec<(String, i32, String, String, i32, bool)> = sqlx::query_as(
            "SELECT id, sequence, label, completion_criteria, standard_time_seconds, done \
             FROM task_steps WHERE task_id = $1 ORDER BY sequence",
        )
        .bind(task_id)
        .fetch_all(&self.pool).await?;
        Ok(rows.into_iter().map(|(id, seq, lab, cri, sec, done)| TaskStepRow {
            id, sequence: seq, label: lab, completion_criteria: cri, standard_time_seconds: sec, done,
        }).collect())
    }

    pub async fn upsert_step(
        &self,
        task_id: &str,
        step: &TaskStepRow,
    ) -> Result<(), PostgresRepositoryError> {
        sqlx::query(
            "INSERT INTO task_steps (id, task_id, sequence, label, completion_criteria, standard_time_seconds, done) \
             VALUES ($1, $2, $3, $4, $5, $6, $7) \
             ON CONFLICT (task_id, id) DO UPDATE SET \
               sequence = EXCLUDED.sequence, \
               label = EXCLUDED.label, \
               completion_criteria = EXCLUDED.completion_criteria, \
               standard_time_seconds = EXCLUDED.standard_time_seconds, \
               done = EXCLUDED.done",
        )
        .bind(&step.id).bind(task_id).bind(step.sequence).bind(&step.label)
        .bind(&step.completion_criteria).bind(step.standard_time_seconds).bind(step.done)
        .execute(&self.pool).await?;
        Ok(())
    }

    pub async fn mark_step_done(
        &self, task_id: &str, step_id: &str,
    ) -> Result<(), PostgresRepositoryError> {
        sqlx::query("UPDATE task_steps SET done = TRUE WHERE task_id = $1 AND id = $2")
            .bind(task_id).bind(step_id)
            .execute(&self.pool).await?;
        Ok(())
    }

    // ===== タスク一覧（班長監視用） =====
    pub async fn list_tasks(&self) -> Result<Vec<TaskListItem>, PostgresRepositoryError> {
        self.list_tasks_since(None).await
    }

    /// 増分同期用：updated_at > cursor の行を新しい順に返す。
    /// cursor が None なら全件（上限 100）。§10.6 オフライン耐性で端末側の
    /// 帯域を抑えるために使う。
    pub async fn list_tasks_since(
        &self,
        cursor: Option<DateTime<Utc>>,
    ) -> Result<Vec<TaskListItem>, PostgresRepositoryError> {
        let rows: Vec<(String, Option<String>, String, String, Option<String>, Option<String>, DateTime<Utc>)> =
            match cursor {
                Some(c) => sqlx::query_as(
                    "SELECT id, title, state, device_id, responsible_user, current_step_id, updated_at \
                     FROM tasks WHERE updated_at > $1 ORDER BY updated_at DESC LIMIT 100",
                )
                .bind(c)
                .fetch_all(&self.pool).await?,
                None => sqlx::query_as(
                    "SELECT id, title, state, device_id, responsible_user, current_step_id, updated_at \
                     FROM tasks ORDER BY updated_at DESC LIMIT 100",
                )
                .fetch_all(&self.pool).await?,
            };
        Ok(rows.into_iter().map(|(id, title, state, device, ru, cs, ut)| TaskListItem {
            id, title, state, device_id: device, responsible_user: ru, current_step_id: cs, updated_at: ut,
        }).collect())
    }

    pub async fn update_current_step(
        &self, task_id: &str, step_id: Option<&str>,
    ) -> Result<(), PostgresRepositoryError> {
        sqlx::query("UPDATE tasks SET current_step_id = $1, updated_at = NOW() WHERE id = $2")
            .bind(step_id).bind(task_id)
            .execute(&self.pool).await?;
        Ok(())
    }

    /// タスクの表示メタ（タイトル／フロー／担当者）を一括更新する。
    ///
    /// `0003_master_data.sql` で追加された ALTER 由来の補助カラムを埋める用途。
    /// Aggregate ルートの値ではなく「班長監視ビュー（§10.3.6）の射影」専用のため
    /// `TaskRepository::save` には含めず、本リポジトリの inherent に置く。
    ///
    /// # Errors
    /// sqlx エラーを返す。`task_id` 未存在は無視（影響行 0 件）。
    pub async fn update_task_meta(
        &self,
        task_id: &str,
        title: Option<&str>,
        flow_id: Option<&str>,
        responsible_user: Option<&str>,
    ) -> Result<(), PostgresRepositoryError> {
        sqlx::query(
            "UPDATE tasks SET \
               title = $1, \
               flow_id = $2, \
               responsible_user = $3, \
               updated_at = NOW() \
             WHERE id = $4",
        )
        .bind(title)
        .bind(flow_id)
        .bind(responsible_user)
        .bind(task_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    // ===== フロー =====
    pub async fn list_flows(&self) -> Result<Vec<FlowSummary>, PostgresRepositoryError> {
        let rows: Vec<(String, i32, String, String, Option<String>)> = sqlx::query_as(
            "SELECT DISTINCT ON (id) id, version, name, status, industry FROM flows ORDER BY id, version DESC"
        )
        .fetch_all(&self.pool).await?;
        Ok(rows.into_iter().map(|(id, ver, n, s, i)| FlowSummary {
            id, version: ver, name: n, status: s, industry: i,
        }).collect())
    }

    pub async fn upsert_flow(
        &self,
        id: &str,
        version: i32,
        name: &str,
        industry: Option<&str>,
        status: &str,
        body_json: &str,
    ) -> Result<(), PostgresRepositoryError> {
        sqlx::query(
            "INSERT INTO flows (id, version, name, industry, status, body) VALUES ($1, $2, $3, $4, $5, $6) \
             ON CONFLICT (id, version) DO UPDATE SET name = EXCLUDED.name, industry = EXCLUDED.industry, \
             status = EXCLUDED.status, body = EXCLUDED.body, updated_at = NOW()",
        )
        .bind(id).bind(version).bind(name).bind(industry).bind(status).bind(body_json)
        .execute(&self.pool).await?;
        Ok(())
    }
}
