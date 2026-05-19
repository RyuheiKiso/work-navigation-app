// ユーザーリポジトリ Trait
// ユーザー管理・認証・スキルゲート検証のための Trait。

use async_trait::async_trait;
use uuid::Uuid;

use crate::error::DomainError;
use crate::model::pagination::{Page, Pagination};
use crate::model::user::{RoleId, User};

/// ユーザーリポジトリ Trait。
#[async_trait]
pub trait UserRepository: Send + Sync + 'static {
    /// ID でユーザーを検索する。
    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, DomainError>;

    /// ログイン ID でユーザーを検索する（認証フローで使用）。
    async fn find_by_login_id(&self, login_id: &str) -> Result<Option<User>, DomainError>;

    /// アクティブなユーザー一覧を取得する。
    async fn list_active(&self, page: Pagination) -> Result<Page<User>, DomainError>;

    /// 新規ユーザーを INSERT する。
    async fn create(&self, cmd: CreateUserCmd) -> Result<User, DomainError>;

    /// ユーザーのロールを更新する。
    async fn update_roles(&self, id: Uuid, roles: Vec<RoleId>) -> Result<User, DomainError>;
}

/// 新規ユーザー作成コマンド。
#[derive(Debug)]
pub struct CreateUserCmd {
    /// ユーザー ID（UUID v7）
    pub user_id: Uuid,
    /// ログイン ID（一意）
    pub login_id: String,
    /// bcrypt ハッシュ化済みパスワード
    pub password_hash: String,
    /// 表示名
    pub display_name: String,
    /// メールアドレス（任意）
    pub email: Option<String>,
    /// 所属工場 ID
    pub factory_id: Uuid,
    /// 初期ロール一覧
    pub roles: Vec<RoleId>,
    /// スキルレベル
    pub skill_level: u8,
}
