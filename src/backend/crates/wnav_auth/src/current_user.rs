// AuthMiddleware が JWT 検証後に Request Extension に追加するユーザー情報（MOD-BE-005 §3）
// 全ハンドラから Extension<CurrentUser> で取得可能。

use uuid::Uuid;

/// JWT 検証済みの現在ユーザー情報。
/// AuthMiddleware が JWT を検証した後、`Request::extensions()` に挿入する。
/// ハンドラでは `Extension<CurrentUser>` または `AuthenticatedUser<R>` で取得する。
#[derive(Debug, Clone)]
pub struct CurrentUser {
    /// ユーザー ID（JWT の sub クレーム）
    pub user_id: Uuid,
    /// 付与されているロール名リスト（RoleId の snake_case 表現）
    pub roles: Vec<String>,
    /// 工場 ID（JWT の factory_id クレーム）
    pub factory_id: Uuid,
    /// 端末 ID（JWT の device_id クレーム、terminal-api のみ）
    pub device_id: Option<Uuid>,
    /// JWT ID（失効チェック用）
    pub jti: Uuid,
}
