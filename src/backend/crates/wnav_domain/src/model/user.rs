// システムユーザーのドメインモデル（EN-001）
// 全コンテキストで参照される横断エンティティ（Shared Kernel）。
// RBAC 6 ロール（FR-AU-007 / src/backend/CLAUDE.md RBAC 6 ロール）を保持する。

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// システムユーザー（EN-001）。
/// 全コンテキストで参照される横断エンティティ（Shared Kernel）。
/// password_hash は bcrypt ハッシュ化済み（LDAP 不可時のローカル認証用）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    /// ユーザー ID（UUID v7）
    pub user_id: Uuid,
    /// ログイン ID（一意）
    pub login_id: String,
    /// bcrypt ハッシュ化済みパスワード（LDAP 不可時のローカル認証用）
    pub password_hash: String,
    /// 表示名
    pub display_name: String,
    /// メールアドレス（任意）
    pub email: Option<String>,
    /// 所属工場 ID
    pub factory_id: Uuid,
    /// 付与されているロール一覧（RBAC 6 ロール）
    pub roles: Vec<RoleId>,
    /// スキルレベル（スキルゲート BR-BUS-002 で使用）
    pub skill_level: u8,
    /// アクティブフラグ（論理削除。物理削除は禁止）
    pub is_active: bool,
}

/// RBAC ロール ID。6 ロール定義（src/backend/CLAUDE.md RBAC 6 ロール）。
/// 各ロールの権限はエンドポイントの型で強制する（axum::extract パターン）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RoleId {
    /// 作業者（現場端末操作）
    Operator,
    /// 監督者（Cancel 権限・アンドン解除）
    Supervisor,
    /// マスタ編集者（SOP 作成・編集）
    MasterAdmin,
    /// 品質管理者（SOP 承認・公開。BR-BUS-012）
    QualityAdmin,
    /// システム管理者（ユーザー管理・設定）
    SystemAdmin,
    /// 経営者（閲覧のみ）
    Executive,
}
