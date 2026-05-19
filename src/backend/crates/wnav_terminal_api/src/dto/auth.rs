// 認証 API（API-auth-001〜003）の DTO 定義（02_認証・認可API仕様.md）

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

// ─────────────────────────────────────────────────────────────────────────────
// API-auth-001: POST /api/v1/auth/login
// ─────────────────────────────────────────────────────────────────────────────

/// ログインリクエスト（API-auth-001）
#[derive(Debug, Deserialize, ToSchema)]
#[allow(dead_code)]
pub struct LoginRequest {
    /// LDAP uid / ローカルユーザー名（1〜64 文字、英数字・アンダースコア・ハイフン）
    pub login_id: String,
    /// パスワード（8〜128 文字、TLS 上での送信のみ許可）
    pub password: String,
    /// ハンディ端末 ID（TBL-033）
    pub device_id: Uuid,
    /// 工場 ID
    pub factory_id: Uuid,
}

/// ログインレスポンスの data フィールド（API-auth-001）
#[derive(Debug, Serialize, ToSchema)]
pub struct LoginData {
    /// RS256 署名済み JWT（aud: "terminal-api"、有効期限 8 時間）
    pub access_token: String,
    /// リフレッシュトークン（有効期限 7 日）
    pub refresh_token: Uuid,
    /// トークン種別（常に "Bearer"）
    pub token_type: &'static str,
    /// アクセストークン有効秒数（28800 = 8h）
    pub expires_in: u64,
    /// リフレッシュトークン有効秒数（604800 = 7d）
    pub refresh_expires_in: u64,
    /// ログインユーザーの RBAC ロール一覧
    pub roles: Vec<String>,
    /// ログインユーザーの ID（TBL-016）
    pub user_id: Uuid,
    /// 認証された工場 ID
    pub factory_id: Uuid,
}

// ─────────────────────────────────────────────────────────────────────────────
// API-auth-002: POST /api/v1/auth/refresh
// ─────────────────────────────────────────────────────────────────────────────

/// トークンリフレッシュリクエスト（API-auth-002）
#[derive(Debug, Deserialize, ToSchema)]
pub struct RefreshRequest {
    /// リフレッシュトークン（API-auth-001 で発行されたもの）
    pub refresh_token: Uuid,
}

/// トークンリフレッシュレスポンスの data フィールド（API-auth-002）
#[derive(Debug, Serialize, ToSchema)]
pub struct RefreshData {
    /// 新規発行 JWT（有効期限 8 時間）
    pub access_token: String,
    /// トークン種別（常に "Bearer"）
    pub token_type: &'static str,
    /// アクセストークン有効秒数（28800）
    pub expires_in: u64,
}

// ─────────────────────────────────────────────────────────────────────────────
// API-auth-003: POST /api/v1/auth/logout
// ─────────────────────────────────────────────────────────────────────────────

/// ログアウトリクエスト（API-auth-003）
/// ボディは空オブジェクト {}。ログアウト対象トークンは Authorization ヘッダから取得する
#[derive(Debug, Deserialize, ToSchema, Default)]
pub struct LogoutRequest {}
