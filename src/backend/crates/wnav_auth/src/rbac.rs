// RBAC 6 ロール定義と型レベルロール強制（MOD-BE-005 §3 / FNC-BE-015）
// ロールマーカー型と AuthenticatedUser<R> で、コンパイル時にロールを強制する。
// effective_roles() でロール階層を表現し、上位ロールが下位ロールの権限を包含する。

use std::marker::PhantomData;

use axum::{
    extract::FromRequestParts,
    http::request::Parts,
};

use crate::{current_user::CurrentUser, error::AuthError};

// ─────────────────────────────────────────────────────────────────────────────
// ロールマーカー型（コンパイル時強制）
// ─────────────────────────────────────────────────────────────────────────────

/// RBAC ロールを型レベルで表現するマーカー Trait。
/// `AuthenticatedUser<R>` の型パラメータ R に使用する。
pub trait Role: Send + Sync + 'static {
    /// DB / JWT の roles フィールドで使用するロール名（snake_case）
    fn role_name() -> &'static str;
}

/// ロール: システム管理者（ユーザー管理・設定。全権限を包含）
pub struct AdminRole;
/// ロール: 経営者（閲覧のみ。Dashboard・監査 Trail の Read-only）
pub struct AuditorRole;
/// ロール: 品質管理者（SOP 承認・公開。BR-BUS-012）
pub struct ApproverRole;
/// ロール: マスタ編集者（SOP 作成・編集）
pub struct MasterEditorRole;
/// ロール: 監督者（Cancel 権限・アンドン解除）
pub struct SupervisorRole;
/// ロール: 作業者（現場端末操作）
pub struct OperatorRole;

impl Role for AdminRole {
    // SystemAdmin ロール: 全権限を包含する最上位ロール
    fn role_name() -> &'static str {
        "system_admin"
    }
}

impl Role for AuditorRole {
    // Executive ロール: 閲覧のみ（個人特定監視禁止の倫理品質制約を遵守）
    fn role_name() -> &'static str {
        "executive"
    }
}

impl Role for ApproverRole {
    // QualityAdmin ロール: SOP 承認・公開権限
    fn role_name() -> &'static str {
        "quality_admin"
    }
}

impl Role for MasterEditorRole {
    // MasterAdmin ロール: SOP 作成・編集権限
    fn role_name() -> &'static str {
        "master_admin"
    }
}

impl Role for SupervisorRole {
    // Supervisor ロール: Cancel 権限・アンドン解除権限
    fn role_name() -> &'static str {
        "supervisor"
    }
}

impl Role for OperatorRole {
    // Operator ロール: 現場端末操作の基本ロール
    fn role_name() -> &'static str {
        "operator"
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// AuthenticatedUser<R>: 型パラメータ R でロールを制限する Extractor
// ─────────────────────────────────────────────────────────────────────────────

/// JWT 検証済みかつ指定ロールを保有するユーザーを表す型。
///
/// # 使用例
/// ```rust
/// // QualityAdmin 以上でないとコンパイルエラーになる型設計
/// async fn get_audit_trail(
///     user: AuthenticatedUser<ApproverRole>,
/// ) -> impl IntoResponse {
///     // user.user_id, user.factory_id などにアクセス可能
/// }
/// ```
pub struct AuthenticatedUser<R: Role> {
    /// ユーザー ID
    pub user_id: uuid::Uuid,
    /// 工場 ID
    pub factory_id: uuid::Uuid,
    /// 端末 ID（terminal-api のみ）
    pub device_id: Option<uuid::Uuid>,
    /// JWT ID
    pub jti: uuid::Uuid,
    /// 実際に保有するロール全件（ロール階層展開後）
    pub all_roles: Vec<String>,
    /// コンパイル時ロール型（実行時に値を持たない）
    _role: PhantomData<R>,
}

impl<S, R> FromRequestParts<S> for AuthenticatedUser<R>
where
    S: Send + Sync,
    R: Role,
{
    type Rejection = AuthError;

    /// Bearer トークンを検証し、要求ロールを保有するかチェックする。
    /// 失敗時は RFC 7807 Problem Details 形式の 401/403 レスポンスを返す。
    /// axum 0.8 では async_trait を使わず impl Future を直接返す形式を使用する。
    fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> impl std::future::Future<Output = Result<Self, Self::Rejection>> + Send {
        // Future を返すクロージャとして実装する（axum 0.8 の要求形式）
        let result = (|| {
            // Request Extension から CurrentUser を取得する（AuthMiddleware が挿入済み）
            let current_user = parts
                .extensions
                .get::<CurrentUser>()
                .ok_or(AuthError::Unauthorized)?;

            // ロール階層を展開して要求ロールを含むか確認する
            let required = R::role_name();
            let has_role = current_user
                .roles
                .iter()
                .any(|r| effective_role_names(r).contains(&required));

            if !has_role {
                return Err(AuthError::InsufficientRole {
                    required: required.to_string(),
                    actual: current_user.roles.clone(),
                });
            }

            Ok(Self {
                user_id: current_user.user_id,
                factory_id: current_user.factory_id,
                device_id: current_user.device_id,
                jti: current_user.jti,
                all_roles: current_user.roles.clone(),
                _role: PhantomData,
            })
        })();

        std::future::ready(result)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// ロール階層評価（FNC-BE-015）
// ─────────────────────────────────────────────────────────────────────────────

/// ロール名から effective なロール名一覧を返す（ロール階層展開）。
///
/// ロール階層: SystemAdmin > QualityAdmin / MasterAdmin > Supervisor > Operator
/// SystemAdmin は全ロールの権限を包含する最上位ロール。
/// Executive は独立した閲覧専用ロール（ロール階層に含まれない）。
pub fn effective_role_names(role: &str) -> Vec<&'static str> {
    match role {
        // SystemAdmin: 全ロール権限を包含する
        "system_admin" => vec![
            "system_admin",
            "quality_admin",
            "master_admin",
            "supervisor",
            "operator",
            "executive",
        ],
        // QualityAdmin: Supervisor・Operator 権限を包含する
        "quality_admin" => vec!["quality_admin", "supervisor", "operator"],
        // MasterAdmin: Supervisor・Operator 権限を包含する
        "master_admin" => vec!["master_admin", "supervisor", "operator"],
        // Supervisor: Operator 権限を包含する
        "supervisor" => vec!["supervisor", "operator"],
        // Operator: 基本ロール（包含なし）
        "operator" => vec!["operator"],
        // Executive: 独立した閲覧専用ロール（ロール階層外）
        "executive" => vec!["executive"],
        // 未知のロールは空リストを返す
        _ => vec![],
    }
}

/// ユーザーが要求ロールを少なくとも 1 つ保有するか評価する（FNC-BE-015）。
///
/// ロール階層を考慮して評価する（上位ロールは下位ロールの権限を包含する）。
pub fn evaluate_roles(current_roles: &[String], required_role: &str) -> bool {
    current_roles
        .iter()
        .any(|r| effective_role_names(r).contains(&required_role))
}
