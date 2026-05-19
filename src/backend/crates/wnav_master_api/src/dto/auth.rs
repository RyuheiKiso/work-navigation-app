// 認証 API DTO（API-auth-001〜004）
//
// ログイン・トークン更新・ログアウト・鍵ローテーション の Request/Response 型。

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// ログインリクエスト（API-auth-001）
#[derive(Debug, Deserialize, ToSchema)]
pub struct LoginRequest {
    /// ログイン ID（メールアドレスまたは社員番号）
    pub login_id: String,
    /// 平文パスワード（TLS 上でのみ送信する）
    pub password: String,
}

/// ログインレスポンス（API-auth-001）
///
/// JWT アクセストークン（TTL 8h）とリフレッシュトークン（TTL 7d）を返す。
#[derive(Debug, Serialize, ToSchema)]
pub struct LoginResponse {
    /// JWT アクセストークン（Bearer として Authorization ヘッダに付与する）
    pub access_token: String,
    /// リフレッシュトークン（/api/v1/auth/refresh で使用する）
    pub refresh_token: String,
    /// アクセストークンの有効期限（秒）
    pub expires_in: u64,
    /// トークン種別（常に "Bearer"）
    pub token_type: String,
}

/// トークン更新リクエスト（API-auth-002）
#[derive(Debug, Deserialize, ToSchema)]
pub struct RefreshRequest {
    /// 有効なリフレッシュトークン
    pub refresh_token: String,
}

/// トークン更新レスポンス（API-auth-002）
#[derive(Debug, Serialize, ToSchema)]
pub struct RefreshResponse {
    /// 新規発行されたアクセストークン
    pub access_token: String,
    /// アクセストークンの有効期限（秒）
    pub expires_in: u64,
    /// トークン種別（常に "Bearer"）
    pub token_type: String,
}

/// ログアウトリクエスト（API-auth-003）
#[derive(Debug, Deserialize, ToSchema)]
pub struct LogoutRequest {
    /// 失効させるリフレッシュトークン
    pub refresh_token: String,
}

/// JWT 鍵ローテーションリクエスト（API-auth-004）
///
/// AdminRole 必須。新しい RSA-4096 秘密鍵・公開鍵ペアを設定する。
#[derive(Debug, Deserialize, ToSchema)]
#[allow(dead_code)]
pub struct KeyRotateRequest {
    /// 新しい RSA-4096 秘密鍵（PEM 形式）
    pub new_private_key_pem: String,
    /// 新しい RSA-4096 公開鍵（PEM 形式）
    pub new_public_key_pem: String,
    /// 新しい鍵 ID（例: "2026-Q3"）
    pub new_kid: String,
    /// Grace Period の秒数（新旧鍵を同時に受け入れる期間）
    pub grace_period_sec: u64,
}

/// JWT 鍵ローテーションレスポンス（API-auth-004）
#[derive(Debug, Serialize, ToSchema)]
pub struct KeyRotateResponse {
    /// 新しい鍵 ID
    pub new_kid: String,
    /// ローテーション完了メッセージ
    pub message: String,
    /// 旧鍵の失効予定時刻（Unix 秒）
    pub old_key_expires_at: i64,
}
