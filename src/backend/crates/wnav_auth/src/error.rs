// JWT 認証・認可エラー型（MOD-BE-005）
// RFC 7807 Problem Details 形式で HTTP レスポンスに変換する。
// スタックトレース・内部エラー詳細はクライアントに返さない（ログにのみ記録する）。

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use wnav_common::ProblemDetails;

/// JWT 認証・RBAC 認可に関するエラー列挙型。
/// `axum::response::IntoResponse` を実装し、RFC 7807 Problem Details 形式で返却する。
#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    /// Authorization ヘッダまたは Bearer トークンが存在しない
    #[error("Authorization token is missing")]
    MissingToken,

    /// JWT の署名・フォーマットが不正
    #[error("JWT token is invalid: {0}")]
    InvalidToken(String),

    /// JWT の有効期限が切れている
    #[error("JWT token has expired")]
    TokenExpired,

    /// JWT 署名検証に失敗した（公開鍵不一致）
    #[error("JWT signature is invalid")]
    InvalidSignature,

    /// aud クレームが自バイナリの `expected_audience` と一致しない
    #[error("JWT audience is invalid")]
    InvalidAudience,

    /// ロール不足（要求ロールをユーザーが持っていない）
    #[error("Insufficient role: required {required}, actual {actual:?}")]
    InsufficientRole {
        required: String,
        actual: Vec<String>,
    },

    /// ユーザーが見つからない
    #[error("User not found")]
    UserNotFound,

    /// パスワードが不正
    #[error("Invalid credentials")]
    InvalidCredentials,

    /// アカウントがロックされている
    #[error("Account is locked")]
    AccountLocked,

    /// 認証情報がない（ExtensionでCurrentUserが取れない場合）
    #[error("Unauthorized: missing authentication")]
    Unauthorized,

    /// JWT ヘッダに kid フィールドがない
    #[error("JWT header is missing kid field")]
    MissingKid,

    /// kid が鍵ストアに存在しない
    #[error("Unknown kid: {0}")]
    UnknownKid(String),

    /// 公開鍵の形式が不正
    #[error("Invalid public key format")]
    InvalidPublicKey,

    /// 秘密鍵の形式が不正
    #[error("Invalid private key format")]
    InvalidPrivateKey,

    /// JWT の有効期限切れ（decode 時）
    #[error("JWT has expired")]
    JwtExpired,

    /// JWT エンコードエラー
    #[error("JWT encode error: {0}")]
    JwtEncodeError(String),

    /// bcrypt ハッシュ化エラー
    #[error("Password hash error: {0}")]
    PasswordHashError(String),
}

impl IntoResponse for AuthError {
    // RFC 7807 Problem Details 形式で HTTP レスポンスを生成する
    fn into_response(self) -> Response {
        let (status, problem_type, title, detail) = match &self {
            Self::MissingToken => (
                StatusCode::UNAUTHORIZED,
                "https://errors.wnav.example.com/auth/missing-token",
                "Missing Token",
                "Authorization header with Bearer token is required.".to_string(),
            ),
            Self::InvalidToken(msg) => {
                tracing::warn!(error = %msg, "JWT token is invalid");
                (
                    StatusCode::UNAUTHORIZED,
                    "https://errors.wnav.example.com/auth/invalid-token",
                    "Invalid Token",
                    "The provided JWT token is invalid.".to_string(),
                )
            }
            Self::TokenExpired | Self::JwtExpired => (
                StatusCode::UNAUTHORIZED,
                "https://errors.wnav.example.com/auth/token-expired",
                "Token Expired",
                "The JWT token has expired. Please log in again.".to_string(),
            ),
            Self::InvalidSignature => (
                StatusCode::UNAUTHORIZED,
                "https://errors.wnav.example.com/auth/invalid-signature",
                "Invalid Signature",
                "JWT signature verification failed.".to_string(),
            ),
            Self::InvalidAudience => (
                StatusCode::UNAUTHORIZED,
                "https://errors.wnav.example.com/auth/invalid-audience",
                "Invalid Audience",
                "The JWT token audience does not match this service.".to_string(),
            ),
            Self::InsufficientRole { required, .. } => (
                StatusCode::FORBIDDEN,
                "https://errors.wnav.example.com/auth/insufficient-role",
                "Insufficient Role",
                format!("Required role: '{required}'. Your roles do not include it."),
            ),
            Self::UserNotFound => (
                StatusCode::UNAUTHORIZED,
                "https://errors.wnav.example.com/auth/user-not-found",
                "User Not Found",
                "The user specified in the token does not exist.".to_string(),
            ),
            Self::InvalidCredentials => (
                StatusCode::UNAUTHORIZED,
                "https://errors.wnav.example.com/auth/invalid-credentials",
                "Invalid Credentials",
                "Login ID or password is incorrect.".to_string(),
            ),
            Self::AccountLocked => (
                StatusCode::FORBIDDEN,
                "https://errors.wnav.example.com/auth/account-locked",
                "Account Locked",
                "This account has been locked. Please contact your administrator.".to_string(),
            ),
            Self::Unauthorized => (
                StatusCode::UNAUTHORIZED,
                "https://errors.wnav.example.com/auth/unauthorized",
                "Unauthorized",
                "Authentication is required to access this resource.".to_string(),
            ),
            Self::MissingKid => (
                StatusCode::UNAUTHORIZED,
                "https://errors.wnav.example.com/auth/missing-kid",
                "Missing Key ID",
                "JWT header is missing the 'kid' field.".to_string(),
            ),
            Self::UnknownKid(kid) => {
                tracing::warn!(kid = %kid, "Unknown kid in JWT header");
                (
                    StatusCode::UNAUTHORIZED,
                    "https://errors.wnav.example.com/auth/unknown-kid",
                    "Unknown Key ID",
                    "The key ID in the JWT header is not recognized.".to_string(),
                )
            }
            Self::InvalidPublicKey | Self::InvalidPrivateKey => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "https://errors.wnav.example.com/auth/key-error",
                "Key Configuration Error",
                "A server-side key configuration error occurred.".to_string(),
            ),
            Self::JwtEncodeError(msg) => {
                tracing::error!(error = %msg, "JWT encode error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "https://errors.wnav.example.com/auth/encode-error",
                    "JWT Encode Error",
                    "Failed to issue JWT token.".to_string(),
                )
            }
            Self::PasswordHashError(msg) => {
                tracing::error!(error = %msg, "Password hash error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "https://errors.wnav.example.com/auth/hash-error",
                    "Password Hash Error",
                    "A server-side password processing error occurred.".to_string(),
                )
            }
        };

        let body = ProblemDetails::new(status.as_u16(), problem_type, title, &detail);

        (
            status,
            [("Content-Type", "application/problem+json")],
            serde_json::to_string(&body).unwrap_or_else(|_| {
                r#"{"type":"about:blank","title":"Internal Server Error","status":500,"detail":"Failed to serialize error response"}"#.to_string()
            }),
        )
            .into_response()
    }
}
