// アプリケーション共通エラー型（RFC 7807 Problem Details 準拠）
// `AppError` は全ハンドラが返す統一エラー型。`IntoResponse` で HTTP レスポンスに変換する。

use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use wnav_common::ProblemDetails;

/// wnav_master_api 全ハンドラが返す統一エラー型。
/// RFC 7807 Problem Details 形式で HTTP レスポンスに変換する。
// API エラー列挙型は全エラーコードを網羅するため、現時点でハンドラが使用していないバリアントも含む。
#[derive(Debug, thiserror::Error)]
#[allow(dead_code)]
pub enum AppError {
    // ── 認証・認可エラー ──────────────────────────────────────────────────
    /// Authorization ヘッダまたは Bearer トークンが存在しない
    #[error("Authorization token is missing")]
    Unauthorized,

    /// JWT の有効期限が切れている
    #[error("JWT token has expired")]
    JwtExpired,

    /// ロール不足（要求ロールをユーザーが持っていない）
    #[error("Insufficient role")]
    Forbidden,

    // ── リソースエラー ────────────────────────────────────────────────────
    /// 要求リソースが見つからない
    #[error("Resource not found: {0}")]
    NotFound(String),

    /// リクエストが既存リソースと競合している
    #[error("Conflict: {0}")]
    Conflict(String),

    // ── バリデーションエラー ──────────────────────────────────────────────
    /// リクエストボディのバリデーション失敗
    #[error("Validation failed: {0}")]
    Validation(String),

    /// 署名検証失敗（Webhook 受信時）
    #[error("HMAC signature verification failed")]
    InvalidSignature,

    // ── ビジネスルール違反 ────────────────────────────────────────────────
    /// Two-Person Integrity 違反（同一人物が両承認者になっている）
    #[error("Two-Person Integrity violation: approver and submitter must be different users")]
    TwoPersonIntegrityViolation,

    /// マスタバージョンの状態遷移が不正
    #[error("Invalid state transition: {0}")]
    InvalidStateTransition(String),

    // ── レートリミット ────────────────────────────────────────────────────
    /// レートリミット超過
    #[error("Rate limit exceeded")]
    RateLimited,

    // ── IQC ビジネスルール ──────────────────────────────────────────────────
    /// ERR-BIZ-017: IQC 判定済み（HTTP 409）
    #[error("IQC inspection already judged")]
    AlreadyJudged,

    // ── 外部システム ─────────────────────────────────────────────────────────
    /// ERR-EXT-001: 親機システム応答なし（HTTP 503）
    #[error("Parent system unavailable")]
    ParentSystemUnavailable,

    /// ERR-EXT-002: LDAP/AD 応答なし（HTTP 503）
    #[error("LDAP unavailable")]
    LdapUnavailable,

    // ── 帳票生成 ────────────────────────────────────────────────────────────
    /// ERR-SYS-003: 帳票生成失敗（HTTP 500）
    #[error("Report generation failed")]
    ReportGenerationFailed,

    /// ERR-SYS-004: テンプレート整合性エラー（HTTP 500）
    #[error("Template integrity error")]
    TemplateIntegrityError,

    // ── サーバーエラー ────────────────────────────────────────────────────
    /// データベースエラー
    #[error("Database error: {0}")]
    Database(String),

    /// 内部サーバーエラー
    #[error("Internal server error: {0}")]
    Internal(String),
}

impl AppError {
    /// HTTP ステータスコードを返す
    fn status_code(&self) -> StatusCode {
        match self {
            Self::Unauthorized | Self::JwtExpired => StatusCode::UNAUTHORIZED,
            Self::Forbidden => StatusCode::FORBIDDEN,
            Self::NotFound(_) => StatusCode::NOT_FOUND,
            Self::Conflict(_) => StatusCode::CONFLICT,
            Self::Validation(_) | Self::InvalidSignature => StatusCode::UNPROCESSABLE_ENTITY,
            Self::TwoPersonIntegrityViolation | Self::InvalidStateTransition(_) => {
                StatusCode::UNPROCESSABLE_ENTITY
            }
            Self::AlreadyJudged => StatusCode::CONFLICT,
            Self::RateLimited => StatusCode::TOO_MANY_REQUESTS,
            Self::ParentSystemUnavailable | Self::LdapUnavailable => {
                StatusCode::SERVICE_UNAVAILABLE
            }
            Self::ReportGenerationFailed | Self::TemplateIntegrityError | Self::Database(_) | Self::Internal(_) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        }
    }

    /// RFC 7807 type URI を返す
    fn problem_type(&self) -> &'static str {
        match self {
            Self::Unauthorized => "https://errors.wnav.example.com/auth/unauthorized",
            Self::JwtExpired => "https://errors.wnav.example.com/auth/token-expired",
            Self::Forbidden => "https://errors.wnav.example.com/auth/insufficient-role",
            Self::NotFound(_) => "https://errors.wnav.example.com/resource/not-found",
            Self::Conflict(_) => "https://errors.wnav.example.com/resource/conflict",
            Self::Validation(_) => "https://errors.wnav.example.com/validation/invalid-input",
            Self::InvalidSignature => "https://errors.wnav.example.com/webhook/invalid-signature",
            Self::TwoPersonIntegrityViolation => {
                "https://errors.wnav.example.com/business/two-person-integrity"
            }
            Self::InvalidStateTransition(_) => {
                "https://errors.wnav.example.com/business/invalid-state-transition"
            }
            Self::AlreadyJudged => "https://errors.wnav.example.com/ERR-BIZ-017",
            Self::RateLimited => "https://errors.wnav.example.com/rate-limit/exceeded",
            Self::ParentSystemUnavailable => "https://errors.wnav.example.com/ERR-EXT-001",
            Self::LdapUnavailable => "https://errors.wnav.example.com/ERR-EXT-002",
            Self::ReportGenerationFailed => "https://errors.wnav.example.com/ERR-SYS-003",
            Self::TemplateIntegrityError => "https://errors.wnav.example.com/ERR-SYS-004",
            Self::Database(_) => "https://errors.wnav.example.com/server/database-error",
            Self::Internal(_) => "https://errors.wnav.example.com/server/internal-error",
        }
    }

    /// エラー見出しを返す
    fn title(&self) -> &'static str {
        match self {
            Self::Unauthorized => "Unauthorized",
            Self::JwtExpired => "Token Expired",
            Self::Forbidden => "Insufficient Role",
            Self::NotFound(_) => "Not Found",
            Self::Conflict(_) => "Conflict",
            Self::Validation(_) => "Validation Failed",
            Self::InvalidSignature => "Invalid Signature",
            Self::TwoPersonIntegrityViolation => "Two-Person Integrity Violation",
            Self::InvalidStateTransition(_) => "Invalid State Transition",
            Self::AlreadyJudged => "Already Judged",
            Self::RateLimited => "Rate Limit Exceeded",
            Self::ParentSystemUnavailable => "Parent System Unavailable",
            Self::LdapUnavailable => "LDAP Unavailable",
            Self::ReportGenerationFailed => "Report Generation Failed",
            Self::TemplateIntegrityError => "Template Integrity Error",
            Self::Database(_) => "Database Error",
            Self::Internal(_) => "Internal Server Error",
        }
    }

    /// クライアント向けの安全なエラー詳細を返す（内部情報は含まない）
    fn user_detail(&self) -> String {
        match self {
            Self::Unauthorized => "Authentication is required to access this resource.".to_string(),
            Self::JwtExpired => "The JWT token has expired. Please log in again.".to_string(),
            Self::Forbidden => {
                "Your role does not have permission to perform this operation.".to_string()
            }
            Self::NotFound(resource) => format!("{resource} was not found."),
            Self::Conflict(msg) => msg.clone(),
            Self::Validation(msg) => msg.clone(),
            Self::InvalidSignature => "HMAC-SHA256 signature verification failed.".to_string(),
            Self::TwoPersonIntegrityViolation => {
                "The submitter and approver must be different users (FR-AU-007).".to_string()
            }
            Self::InvalidStateTransition(msg) => msg.clone(),
            Self::AlreadyJudged => "IQC inspection has already been judged.".to_string(),
            Self::RateLimited => "Too many requests. Please wait before retrying.".to_string(),
            Self::ParentSystemUnavailable => {
                "The parent system is currently unavailable. Retry later.".to_string()
            }
            Self::LdapUnavailable => {
                "LDAP authentication is unavailable. Local authentication fallback applied."
                    .to_string()
            }
            Self::ReportGenerationFailed => {
                "Report generation failed. The operation will be retried automatically.".to_string()
            }
            Self::TemplateIntegrityError => {
                "Report template integrity check failed. Contact system administrator.".to_string()
            }
            Self::Database(_) => "A database error occurred. Please try again later.".to_string(),
            Self::Internal(_) => {
                "An internal server error occurred. Please try again later.".to_string()
            }
        }
    }
}

impl IntoResponse for AppError {
    // RFC 7807 Problem Details 形式でエラーレスポンスを生成する
    fn into_response(self) -> Response {
        let status = self.status_code();
        let detail = self.user_detail();

        // エラーレベルに応じてログを出力する（内部情報はログにのみ記録する）
        match status.as_u16() {
            500..=599 => tracing::error!(
                error = %self,
                status = status.as_u16(),
                "サーバーエラーが発生しました"
            ),
            400..=499 => tracing::warn!(
                error = %self,
                status = status.as_u16(),
                "クライアントエラーが発生しました"
            ),
            _ => {}
        }

        let body = ProblemDetails::new(status.as_u16(), self.problem_type(), self.title(), &detail);

        (
            status,
            [("Content-Type", "application/problem+json")],
            Json(body),
        )
            .into_response()
    }
}

impl From<wnav_auth::AuthError> for AppError {
    // AuthError を AppError に変換する
    fn from(e: wnav_auth::AuthError) -> Self {
        use wnav_auth::AuthError as AE;
        match e {
            AE::TokenExpired | AE::JwtExpired => Self::JwtExpired,
            AE::InsufficientRole { .. } => Self::Forbidden,
            AE::Unauthorized => Self::Unauthorized,
            _ => Self::Unauthorized,
        }
    }
}

impl From<sqlx::Error> for AppError {
    // sqlx エラーを AppError に変換する（内部詳細はログにのみ記録する）
    fn from(e: sqlx::Error) -> Self {
        tracing::error!(error = %e, "データベースエラーが発生しました");
        Self::Database("database operation failed".to_string())
    }
}
