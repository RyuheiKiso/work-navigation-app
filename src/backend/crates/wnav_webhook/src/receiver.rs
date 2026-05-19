// Webhook 受信側の署名検証（MOD-BE-007 §2）
// X-WNav-Signature ヘッダを HMAC-SHA256 + 定数時間比較で検証する。
// X-WNav-Timestamp ヘッダでリプレイ攻撃を防止する。

use crate::{
    error::WebhookError,
    signature::{verify_signature, verify_timestamp},
};

/// タイムスタンプの許容時間窓（秒）: デフォルト 300 秒（5 分）
const DEFAULT_TIMESTAMP_TOLERANCE_SECS: i64 = 300;

/// Webhook 受信側の署名検証器。
///
/// axum ミドルウェアまたはハンドラから呼び出して、
/// 受信リクエストの正当性を確認する。
pub struct WebhookReceiver {
    /// HMAC 秘密鍵文字列（配信側と同一の値）
    hmac_secret: String,
    /// タイムスタンプ許容時間窓（秒）
    timestamp_tolerance_secs: i64,
}

impl WebhookReceiver {
    /// WebhookReceiver を生成する。
    ///
    /// # 引数
    /// - `hmac_secret`: HMAC 秘密鍵文字列（配信側と同一）
    pub fn new(hmac_secret: String) -> Self {
        Self {
            hmac_secret,
            timestamp_tolerance_secs: DEFAULT_TIMESTAMP_TOLERANCE_SECS,
        }
    }

    /// タイムスタンプ許容時間窓を変更するビルダーメソッド。
    pub fn with_timestamp_tolerance(mut self, secs: i64) -> Self {
        self.timestamp_tolerance_secs = secs;
        self
    }

    /// 受信リクエストの署名とタイムスタンプを検証する。
    ///
    /// 検証手順:
    /// 1. `X-WNav-Signature` ヘッダから hex digest を取り出す
    /// 2. 同じ secret で HMAC-SHA256(secret, body) を計算する
    /// 3. 定数時間比較で一致を確認する（タイミング攻撃防止）
    /// 4. `X-WNav-Timestamp` ヘッダが許容時間窓内か確認する（リプレイ攻撃防止）
    ///
    /// # 引数
    /// - `body`: 受信したリクエストボディのバイト列
    /// - `signature_header`: `X-WNav-Signature` ヘッダの値（"sha256={hex}" 形式）
    /// - `timestamp_header`: `X-WNav-Timestamp` ヘッダの値（Unix エポック秒文字列、任意）
    ///
    /// # エラー
    /// - `WebhookError::InvalidSignature`: 署名が不一致
    /// - `WebhookError::RequestTimeout`: タイムスタンプが許容範囲外
    #[tracing::instrument(skip(self, body), err)]
    pub fn verify_request(
        &self,
        body: &[u8],
        signature_header: &str,
        timestamp_header: Option<&str>,
    ) -> Result<(), WebhookError> {
        // タイムスタンプが提供されている場合はリプレイ攻撃防止チェックを行う
        if let Some(ts) = timestamp_header {
            verify_timestamp(ts, self.timestamp_tolerance_secs)?;
        }

        // HMAC-SHA256 署名を定数時間比較で検証する
        verify_signature(body, &self.hmac_secret, signature_header)?;

        tracing::debug!(
            event = "webhook.signature_verified",
            "Webhook signature verification succeeded",
        );

        Ok(())
    }
}
