// Outbox Consumer エラー型（MOD-BE-006）
// BAT-002 の常駐ループが遭遇する全エラーを網羅する。

use uuid::Uuid;

/// Outbox Consumer エラー列挙型。
///
/// - `Database`: sqlx エラー（接続失敗・クエリエラー）
/// - `WebhookFailed`: Webhook 配信失敗（HTTP エラー・署名エラー等）
/// - `MaxRetriesExceeded`: 最大リトライ回数を超過（DLQ 移行前に発生）
/// - `InvalidConfig`: 設定値不正（hex デコード失敗等）
#[allow(clippy::doc_markdown)]
#[derive(Debug, thiserror::Error)]
pub enum OutboxError {
    /// sqlx データベースエラー
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    /// Webhook 配信失敗（HTTP エラー・タイムアウト等）
    #[error("Webhook delivery failed: {0}")]
    WebhookFailed(String),

    /// 最大リトライ回数超過（DLQ 移行トリガー）
    #[error("Max retries exceeded for outbox_id={outbox_id}")]
    MaxRetriesExceeded {
        /// 最大リトライを超過した Outbox イベント ID
        outbox_id: Uuid,
    },

    /// 設定値が不正（hmac_secret_hex 等）
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    /// HTTP 通信エラー（reqwest エラー）
    #[error("HTTP error: {0}")]
    Http(String),

    /// HTTP レスポンスが非 2xx
    #[error("HTTP status error: {0}")]
    HttpStatus(u16),
}
