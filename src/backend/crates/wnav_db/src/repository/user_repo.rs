// PgUserRepository — TBL-016 users の sqlx 実装
// RBAC ロール管理・認証用ログイン ID 検索を含む。
// SQLX_PREPARE_REQUIRED: cargo sqlx prepare を実行してからビルド可能

use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use wnav_domain::{
    error::DomainError,
    model::{
        pagination::{Page, Pagination},
        user::{RoleId, User},
    },
    repository::{CreateUserCmd, UserRepository},
};

use crate::row_types::UserRow;

/// TBL-016 users のリポジトリ実装。
pub struct PgUserRepository {
    pool: PgPool,
}

impl PgUserRepository {
    /// コネクションプールを受け取ってリポジトリを構築する。
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

/// UserRow から User ドメインモデルへの変換。
/// roles フィールドは JSONB 文字列配列から RoleId 配列に変換する。
impl TryFrom<UserRow> for User {
    type Error = DomainError;

    fn try_from(row: UserRow) -> Result<Self, Self::Error> {
        let roles = parse_roles(&row.roles)?;
        Ok(Self {
            user_id: row.user_id,
            login_id: row.login_id,
            password_hash: row.password_hash,
            display_name: row.display_name,
            email: row.email,
            factory_id: row.factory_id,
            roles,
            skill_level: u8::try_from(row.skill_level).unwrap_or(0),
            is_active: row.is_active,
        })
    }
}

/// JSONB roles 配列を RoleId 列挙型の Vec に変換する。
fn parse_roles(roles_json: &serde_json::Value) -> Result<Vec<RoleId>, DomainError> {
    let arr = roles_json.as_array().ok_or_else(|| {
        DomainError::Internal("roles フィールドが JSON 配列ではありません".to_string())
    })?;

    arr.iter()
        .map(|v| {
            let s = v.as_str().ok_or_else(|| {
                DomainError::Internal("roles 要素が文字列ではありません".to_string())
            })?;
            parse_role_id(s)
        })
        .collect()
}

/// DB 格納文字列を RoleId 列挙型に変換する。
fn parse_role_id(s: &str) -> Result<RoleId, DomainError> {
    match s {
        "operator" => Ok(RoleId::Operator),
        "supervisor" => Ok(RoleId::Supervisor),
        "master_admin" => Ok(RoleId::MasterAdmin),
        "quality_admin" => Ok(RoleId::QualityAdmin),
        "system_admin" => Ok(RoleId::SystemAdmin),
        "executive" => Ok(RoleId::Executive),
        other => Err(DomainError::Internal(format!("不明な RoleId: {other}"))),
    }
}

/// RoleId を DB 格納文字列に変換する（serde_json の snake_case と一致させる）。
fn role_to_str(role: &RoleId) -> &'static str {
    match role {
        RoleId::Operator => "operator",
        RoleId::Supervisor => "supervisor",
        RoleId::MasterAdmin => "master_admin",
        RoleId::QualityAdmin => "quality_admin",
        RoleId::SystemAdmin => "system_admin",
        RoleId::Executive => "executive",
    }
}

#[async_trait]
impl UserRepository for PgUserRepository {
    /// ID でユーザーを検索する。
    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, DomainError> {
        let row = sqlx::query_as::<_, UserRow>(
            r#"
            SELECT
                user_id, login_id, password_hash, display_name,
                email, factory_id, roles, skill_level, is_active
            FROM users
            WHERE user_id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(crate::error::map_sqlx)?;

        row.map(User::try_from).transpose()
    }

    /// ログイン ID でユーザーを検索する（認証フローで使用）。
    async fn find_by_login_id(&self, login_id: &str) -> Result<Option<User>, DomainError> {
        let row = sqlx::query_as::<_, UserRow>(
            r#"
            SELECT
                user_id, login_id, password_hash, display_name,
                email, factory_id, roles, skill_level, is_active
            FROM users
            WHERE login_id = $1
            "#,
        )
        .bind(login_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(crate::error::map_sqlx)?;

        row.map(User::try_from).transpose()
    }

    /// アクティブなユーザー一覧を Pagination で取得する。
    async fn list_active(&self, page: Pagination) -> Result<Page<User>, DomainError> {
        let limit = i64::from(page.per_page);
        let offset = i64::from((page.page - 1) * page.per_page);

        let rows = sqlx::query_as::<_, UserRow>(
            r#"
            SELECT
                user_id, login_id, password_hash, display_name,
                email, factory_id, roles, skill_level, is_active
            FROM users
            WHERE is_active = TRUE
            ORDER BY display_name ASC
            LIMIT $1 OFFSET $2
            "#,
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(crate::error::map_sqlx)?;

        let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users WHERE is_active = TRUE")
            .fetch_one(&self.pool)
            .await
            .map_err(crate::error::map_sqlx)?;

        let items = rows
            .into_iter()
            .map(User::try_from)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Page {
            items,
            total: u64::try_from(total).unwrap_or(0),
            page: page.page,
            per_page: page.per_page,
        })
    }

    /// 新規ユーザーを INSERT する。
    async fn create(&self, cmd: CreateUserCmd) -> Result<User, DomainError> {
        // roles を JSONB 配列文字列に変換する
        let roles_json: serde_json::Value = serde_json::Value::Array(
            cmd.roles
                .iter()
                .map(|r| serde_json::Value::String(role_to_str(r).to_owned()))
                .collect(),
        );

        let row = sqlx::query_as::<_, UserRow>(
            r#"
            INSERT INTO users (
                user_id, login_id, password_hash, display_name,
                email, factory_id, roles, skill_level, is_active
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, TRUE)
            RETURNING
                user_id, login_id, password_hash, display_name,
                email, factory_id, roles, skill_level, is_active
            "#,
        )
        .bind(cmd.user_id)
        .bind(cmd.login_id)
        .bind(cmd.password_hash)
        .bind(cmd.display_name)
        .bind(cmd.email)
        .bind(cmd.factory_id)
        .bind(roles_json)
        .bind(i16::from(cmd.skill_level))
        .fetch_one(&self.pool)
        .await
        .map_err(crate::error::map_sqlx)?;

        User::try_from(row)
    }

    /// ユーザーのロール一覧を更新する。
    async fn update_roles(&self, id: Uuid, roles: Vec<RoleId>) -> Result<User, DomainError> {
        let roles_json: serde_json::Value = serde_json::Value::Array(
            roles
                .iter()
                .map(|r| serde_json::Value::String(role_to_str(r).to_owned()))
                .collect(),
        );

        let row = sqlx::query_as::<_, UserRow>(
            r#"
            UPDATE users
            SET roles = $1
            WHERE user_id = $2
            RETURNING
                user_id, login_id, password_hash, display_name,
                email, factory_id, roles, skill_level, is_active
            "#,
        )
        .bind(roles_json)
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(crate::error::map_sqlx)?;

        User::try_from(row)
    }
}
