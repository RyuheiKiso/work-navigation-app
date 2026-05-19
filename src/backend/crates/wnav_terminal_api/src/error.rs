// wnav_terminal_api エラー型定義（MOD-BE-001 §4）
//
// AppError は thiserror で定義し、IntoResponse で RFC 9457 Problem Details に変換する。
// 全エラーコード（ERR-AUTH-001〜004, ERR-VAL-001〜004, ERR-BIZ-001〜008, ERR-DB-001〜004,
// ERR-SYS-001〜005）を網羅する。

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use thiserror::Error;
use uuid::Uuid;

// ─────────────────────────────────────────────────────────────────────────────
// RFC 9457 Problem Details レスポンスボディ
// ─────────────────────────────────────────────────────────────────────────────

/// RFC 9457 / RFC 7807 Problem Details レスポンスボディ。
///
/// Content-Type: application/problem+json で返却する。
#[derive(Debug, Serialize)]
pub struct ProblemDetails {
    /// エラー種別 URI（例: "https://errors.wnav.example.com/ERR-AUTH-001"）
    #[serde(rename = "type")]
    pub type_: String,
    /// エラー名（英語・機械可読）
    pub title: String,
    /// HTTP ステータスコード
    pub status: u16,
    /// ユーザー向けメッセージ（多言語対応済み文字列）
    pub detail: String,
    /// リクエスト識別子（"/requests/{uuid}"）
    pub instance: String,
    /// ERR-NNN 識別子（例: "ERR-AUTH-001"）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_id: Option<String>,
    /// バリデーション違反の詳細（ERR-VAL-* のみ）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub violations: Option<Vec<Violation>>,
}

/// バリデーション違反の詳細フィールド
#[derive(Debug, Serialize, Clone)]
pub struct Violation {
    /// 違反フィールド名
    pub field: String,
    /// 違反内容のメッセージ
    pub message: String,
}

// ─────────────────────────────────────────────────────────────────────────────
// AppError 列挙型
// ─────────────────────────────────────────────────────────────────────────────

/// wnav_terminal_api 全エンドポイントで使用するエラー型。
///
/// 全エラーは `IntoResponse` 実装で RFC 9457 Problem Details に変換される。
#[derive(Debug, Error)]
pub enum AppError {
    // ─── ERR-AUTH ───────────────────────────────────────────────────────────
    /// ERR-AUTH-001: JWT 不正・有効期限切れ・login_id 不存在など（HTTP 401）
    #[error("Unauthorized")]
    Unauthorized,

    /// ERR-AUTH-002: PIN 検証失敗（HTTP 401）
    #[error("PIN verification failed")]
    PinVerificationFailed,

    /// ERR-AUTH-003: アカウントロック中（HTTP 423）
    #[error("Account locked")]
    AccountLocked,

    /// ERR-AUTH-004: ロール権限不足・他工場リソースアクセス（HTTP 403）
    #[error("Access denied")]
    Forbidden,

    // ─── ERR-VAL ────────────────────────────────────────────────────────────
    /// ERR-VAL-001: 必須フィールド不足（HTTP 422）
    #[error("Required field missing")]
    RequiredFieldMissing(Option<Vec<Violation>>),

    /// ERR-VAL-002: 値が範囲外（HTTP 422）
    #[error("Value out of range")]
    ValueOutOfRange(Option<Vec<Violation>>),

    /// ERR-VAL-003: 形式不正（UUID・MIME・ISO 8601・SHA-256 等）（HTTP 422）
    #[error("Invalid format")]
    InvalidFormat(Option<Vec<Violation>>),

    /// ERR-VAL-004: 最大長超過（HTTP 422）
    #[error("Max length exceeded")]
    MaxLengthExceeded(Option<Vec<Violation>>),

    // ─── ERR-BIZ ────────────────────────────────────────────────────────────
    /// ERR-BIZ-001: ステップ順序違反（HTTP 409）
    #[error("Step sequence violation")]
    StepSequenceViolation,

    /// ERR-BIZ-002: エビデンスゲート（必須エビデンス未添付）（HTTP 409）
    #[error("Evidence gate blocked")]
    EvidenceGate,

    /// ERR-BIZ-003: SOP 未公開バージョン（HTTP 409）
    #[error("SOP version not published")]
    SopNotPublished,

    /// ERR-BIZ-004: 計測器校正期限切れ（HTTP 409）
    #[error("Instrument calibration expired")]
    CalibrationExpired,

    /// ERR-BIZ-005: SOP Freeze 後の変更試行（HTTP 409）
    #[error("SOP frozen after publish")]
    SopFrozen,

    /// ERR-BIZ-006: スキルレベル不足（HTTP 403）
    #[error("Skill level insufficient")]
    SkillInsufficient,

    /// ERR-BIZ-007: バージョン重複（HTTP 409）
    #[error("Version already published")]
    VersionAlreadyPublished,

    /// ERR-BIZ-008: CAPA クローズ済み（HTTP 409）
    #[error("CAPA already closed")]
    CapaAlreadyClosed,

    /// ERR-BIZ-026: ケース占有中（case_locks 競合）（HTTP 409）
    #[error("Case already occupied by another terminal")]
    CaseOccupied,

    // ─── ERR-DB ─────────────────────────────────────────────────────────────
    /// ERR-DB-001: DB エラー（接続失敗・タイムアウト等）（HTTP 500）
    #[error("Database error")]
    DatabaseError,

    /// ERR-DB-002: 外部キー制約違反（HTTP 409）
    #[error("Foreign key violation")]
    ForeignKeyViolation,

    /// ERR-DB-003: ハッシュチェーン破断（HTTP 500）
    #[error("Hash chain broken")]
    HashChainBroken,

    /// ERR-DB-004: 楽観的ロック競合（HTTP 500）
    #[error("Optimistic lock failure")]
    OptimisticLockFailure,

    // ─── ERR-SYS ────────────────────────────────────────────────────────────
    /// ERR-SYS-001: 予期しない内部エラー（HTTP 500）
    #[error("Internal server error")]
    InternalServerError,

    /// ERR-SYS-002: レート制限超過（HTTP 429）
    #[error("Rate limit exceeded")]
    RateLimited,

    /// ERR-SYS-005: DLQ オーバーフロー（HTTP 503）
    #[error("DLQ overflow")]
    DlqOverflow,

    // ─── 404 ────────────────────────────────────────────────────────────────
    /// リソースが見つからない（HTTP 404）
    #[error("Not found")]
    NotFound,

    // ─── Idempotency ────────────────────────────────────────────────────────
    /// Idempotency-Key ヘッダが欠落している（書き込みリクエストで必須）
    #[error("Idempotency-Key header is required")]
    MissingIdempotencyKey,
}

impl AppError {
    /// エラーコード文字列（ERR-NNN 形式）を返す
    fn error_code(&self) -> &'static str {
        match self {
            AppError::Unauthorized => "ERR-AUTH-001",
            AppError::PinVerificationFailed => "ERR-AUTH-002",
            AppError::AccountLocked => "ERR-AUTH-003",
            AppError::Forbidden => "ERR-AUTH-004",
            AppError::RequiredFieldMissing(_) => "ERR-VAL-001",
            AppError::ValueOutOfRange(_) => "ERR-VAL-002",
            AppError::InvalidFormat(_) => "ERR-VAL-003",
            AppError::MaxLengthExceeded(_) => "ERR-VAL-004",
            AppError::StepSequenceViolation => "ERR-BIZ-001",
            AppError::EvidenceGate => "ERR-BIZ-002",
            AppError::SopNotPublished => "ERR-BIZ-003",
            AppError::CalibrationExpired => "ERR-BIZ-004",
            AppError::SopFrozen => "ERR-BIZ-005",
            AppError::SkillInsufficient => "ERR-BIZ-006",
            AppError::VersionAlreadyPublished => "ERR-BIZ-007",
            AppError::CapaAlreadyClosed => "ERR-BIZ-008",
            AppError::CaseOccupied => "ERR-BIZ-026",
            AppError::DatabaseError => "ERR-DB-001",
            AppError::ForeignKeyViolation => "ERR-DB-002",
            AppError::HashChainBroken => "ERR-DB-003",
            AppError::OptimisticLockFailure => "ERR-DB-004",
            AppError::InternalServerError => "ERR-SYS-001",
            AppError::RateLimited => "ERR-SYS-002",
            AppError::DlqOverflow => "ERR-SYS-005",
            AppError::NotFound => "ERR-NOT-FOUND",
            AppError::MissingIdempotencyKey => "ERR-VAL-001",
        }
    }

    /// HTTP ステータスコードを返す
    fn status_code(&self) -> StatusCode {
        match self {
            AppError::Unauthorized | AppError::PinVerificationFailed => StatusCode::UNAUTHORIZED,
            AppError::Forbidden | AppError::SkillInsufficient => StatusCode::FORBIDDEN,
            AppError::AccountLocked => StatusCode::LOCKED,
            AppError::RequiredFieldMissing(_)
            | AppError::ValueOutOfRange(_)
            | AppError::InvalidFormat(_)
            | AppError::MaxLengthExceeded(_)
            | AppError::MissingIdempotencyKey => StatusCode::UNPROCESSABLE_ENTITY,
            AppError::StepSequenceViolation
            | AppError::EvidenceGate
            | AppError::SopNotPublished
            | AppError::CalibrationExpired
            | AppError::SopFrozen
            | AppError::VersionAlreadyPublished
            | AppError::CapaAlreadyClosed
            | AppError::CaseOccupied
            | AppError::ForeignKeyViolation => StatusCode::CONFLICT,
            AppError::NotFound => StatusCode::NOT_FOUND,
            AppError::RateLimited => StatusCode::TOO_MANY_REQUESTS,
            AppError::DlqOverflow => StatusCode::SERVICE_UNAVAILABLE,
            AppError::DatabaseError
            | AppError::HashChainBroken
            | AppError::OptimisticLockFailure
            | AppError::InternalServerError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    /// ログレベルを返す（ログ出力の分岐に使用する）
    fn log_level(&self) -> LogLevel {
        match self {
            AppError::HashChainBroken => LogLevel::Critical,
            AppError::DatabaseError
            | AppError::OptimisticLockFailure
            | AppError::InternalServerError
            | AppError::AccountLocked
            | AppError::DlqOverflow => LogLevel::Error,
            AppError::Unauthorized
            | AppError::PinVerificationFailed
            | AppError::Forbidden
            | AppError::SkillInsufficient
            | AppError::StepSequenceViolation
            | AppError::EvidenceGate
            | AppError::SopNotPublished
            | AppError::CalibrationExpired
            | AppError::SopFrozen
            | AppError::VersionAlreadyPublished
            | AppError::CapaAlreadyClosed
            | AppError::CaseOccupied
            | AppError::ForeignKeyViolation
            | AppError::RateLimited => LogLevel::Warn,
            AppError::RequiredFieldMissing(_)
            | AppError::ValueOutOfRange(_)
            | AppError::InvalidFormat(_)
            | AppError::MaxLengthExceeded(_)
            | AppError::MissingIdempotencyKey
            | AppError::NotFound => LogLevel::Info,
        }
    }

    /// violations を返す（ERR-VAL-* のみ）
    fn violations(&self) -> Option<&Vec<Violation>> {
        match self {
            AppError::RequiredFieldMissing(v)
            | AppError::ValueOutOfRange(v)
            | AppError::InvalidFormat(v)
            | AppError::MaxLengthExceeded(v) => v.as_ref(),
            _ => None,
        }
    }
}

/// ログレベル分類
enum LogLevel {
    Critical,
    Error,
    Warn,
    Info,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        // サーバー側でリクエスト追跡用 UUID を採番する
        let request_id = Uuid::now_v7();
        let status = self.status_code();
        let error_code = self.error_code();

        // エラーレベルに応じた構造化ログを出力する
        match self.log_level() {
            LogLevel::Critical => tracing::error!(
                error_code = %error_code,
                request_id = %request_id,
                "CRITICAL: ハッシュチェーン破断またはシステム整合性エラーを検出した"
            ),
            LogLevel::Error => tracing::error!(
                error_code = %error_code,
                request_id = %request_id,
                "サーバーエラーが発生した"
            ),
            LogLevel::Warn => tracing::warn!(
                error_code = %error_code,
                request_id = %request_id,
                "クライアントエラーまたは業務ルール違反が発生した"
            ),
            LogLevel::Info => tracing::info!(
                error_code = %error_code,
                request_id = %request_id,
                "バリデーションエラーが発生した"
            ),
        }

        // クライアントへは内部詳細を公開しないシンプルなメッセージを返す
        let detail = match &self {
            AppError::StepSequenceViolation => {
                "直前のステップが完了していません。".to_string()
            }
            AppError::EvidenceGate => {
                "必須エビデンスが添付されていません。".to_string()
            }
            AppError::CaseOccupied => {
                "このケースは別の端末で占有中です。".to_string()
            }
            AppError::AccountLocked => {
                "アカウントがロックされています。".to_string()
            }
            AppError::MissingIdempotencyKey => {
                "Idempotency-Key ヘッダが必要です。".to_string()
            }
            AppError::HashChainBroken => {
                "整合性エラーが検出されました。管理者に連絡してください。".to_string()
            }
            _ => self.to_string(),
        };

        let body = ProblemDetails {
            type_: format!("https://errors.wnav.example.com/{error_code}"),
            title: self.to_string(),
            status: status.as_u16(),
            detail,
            instance: format!("/requests/{request_id}"),
            error_id: Some(error_code.to_string()),
            violations: self.violations().cloned(),
        };

        (
            status,
            [("Content-Type", "application/problem+json")],
            Json(body),
        )
            .into_response()
    }
}
