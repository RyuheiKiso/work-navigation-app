// Webhook 配信・署名検証エラー型（MOD-BE-007）
// RFC 7807 準拠のエラーを返す場合は wnav_common::ProblemDetails を使用する。

/// Webhook 配信・受信・署名検証エラー列挙型。
#[derive(Debug, thiserror::Error)]
pub enum WebhookError {
    /// HMAC-SHA256 署名が不一致（定数時間比較で判定）
    #[error("Invalid HMAC-SHA256 signature")]
    InvalidSignature,

    /// ペイロードサイズが上限を超過した（CFG 上限値設定推奨）
    #[error("Payload too large: {size} bytes exceeds limit {limit} bytes")]
    PayloadTooLarge { size: usize, limit: usize },

    /// タイムスタンプが許容時間窓外（リプレイ攻撃防止）
    #[error("Request timestamp is too old or in the future")]
    RequestTimeout,

    /// HTTP 通信エラー（接続失敗・TLS エラー等）
    #[error("HTTP error: {0}")]
    HttpError(String),

    /// 最大リトライ回数を超過した（DLQ 移行前に発生）
    #[error("Max retries exceeded after {attempts} attempts")]
    MaxRetriesExceeded { attempts: u32 },

    /// HMAC 秘密鍵が不正な hex 文字列
    #[error("Invalid HMAC secret hex: {0}")]
    InvalidSecretHex(String),

    /// HTTP レスポンスが非 2xx
    #[error("HTTP response error: status={status}")]
    HttpStatus { status: u16 },
}
