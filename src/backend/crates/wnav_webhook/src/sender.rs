// Webhook 配信サービス（MOD-BE-007 §3 / BAT-008）
// 1 回の HTTP POST と HMAC-SHA256 署名を担う（リトライは wnav_outbox に委譲する）。
// 単一責任の原則: 本クレートはリトライロジックを持たない。

use uuid::Uuid;

use crate::{error::WebhookError, signature::sign_payload};

/// Webhook 配信設定。
/// `wnav_config` の `WebhookConfig` からデシリアライズして使用する。
#[derive(Debug, Clone, serde::Deserialize)]
pub struct WebhookSenderConfig {
    /// HMAC-SHA256 署名秘密鍵（UTF-8 文字列として使用）
    pub hmac_secret: String,
    /// HTTP タイムアウト（ミリ秒）
    pub timeout_ms: u64,
}

impl Default for WebhookSenderConfig {
    fn default() -> Self {
        Self {
            hmac_secret: String::new(),
            // デフォルト 5000ms（CFG-028）
            timeout_ms: 5_000,
        }
    }
}

/// Webhook 配信サービス。
///
/// Outbox Consumer（MOD-BE-006）から呼び出されて HTTP POST を実行する。
/// リトライ・バックオフは `wnav_outbox` が担当し、本クレートは 1 回の POST に専念する。
pub struct WebhookSender {
    /// reqwest HTTP クライアント（タイムアウト・TLS 設定済み）
    client: reqwest::Client,
    /// HMAC 秘密鍵文字列
    hmac_secret: String,
}

impl WebhookSender {
    /// `WebhookSender` を生成する。
    ///
    /// # 引数
    /// - `config`: Webhook 配信設定
    pub fn new(config: WebhookSenderConfig) -> Result<Self, WebhookError> {
        // reqwest クライアントを設定する（タイムアウト・native-tls）
        let client = reqwest::ClientBuilder::new()
            .timeout(std::time::Duration::from_millis(config.timeout_ms))
            .build()
            .map_err(|e| WebhookError::HttpError(e.to_string()))?;

        Ok(Self {
            client,
            hmac_secret: config.hmac_secret,
        })
    }

    /// Webhook エンドポイントにペイロードを 1 回 POST する。
    ///
    /// ヘッダ:
    /// - `Content-Type: application/json`
    /// - `X-WNav-Signature: sha256={hex}`（HMAC-SHA256 署名）
    /// - `X-WNav-Event: {event_type}`（イベント種別）
    /// - `X-WNav-Delivery-Id: {idempotency_key}`（配信 ID）
    /// - `Idempotency-Key: {idempotency_key}`（冪等キー）
    /// - `X-WNav-Timestamp: {unix_epoch_secs}`（タイムスタンプ。受信側でリプレイ防止に使用）
    ///
    /// # 引数
    /// - `endpoint_url`: 配信先 URL（親機 Inbound Webhook）
    /// - `event_type`: イベント種別文字列（X-WNav-Event ヘッダ値）
    /// - `payload`: JSON ペイロード
    /// - `idempotency_key`: 配信 ID（元イベントの UUID を伝播させてリプレイ防止を可能にする）
    ///
    /// # エラー
    /// - `WebhookError::HttpError`: HTTP 通信エラー
    /// - `WebhookError::HttpStatus`: 非 2xx レスポンス
    #[tracing::instrument(skip(self, payload), fields(event_type = %event_type, idempotency_key = %idempotency_key), err)]
    pub async fn send(
        &self,
        endpoint_url: &str,
        event_type: &str,
        payload: &serde_json::Value,
        idempotency_key: Uuid,
    ) -> Result<(), WebhookError> {
        // ペイロードを JSON バイト列にシリアライズする
        let payload_bytes =
            serde_json::to_vec(payload).map_err(|e| WebhookError::HttpError(e.to_string()))?;

        // HMAC-SHA256 署名を計算する（"sha256={hex}" 形式）
        let signature = sign_payload(&payload_bytes, &self.hmac_secret);

        // 現在時刻（Unix エポック秒）をタイムスタンプとして付与する
        let timestamp = chrono::Utc::now().timestamp().to_string();

        // HTTP POST を実行する
        let response = self
            .client
            .post(endpoint_url)
            .header("Content-Type", "application/json")
            .header("X-WNav-Signature", &signature)
            .header("X-WNav-Event", event_type)
            .header("X-WNav-Delivery-Id", idempotency_key.to_string())
            .header("Idempotency-Key", idempotency_key.to_string())
            .header("X-WNav-Timestamp", &timestamp)
            .body(payload_bytes)
            .send()
            .await
            .map_err(|e| WebhookError::HttpError(e.to_string()))?;

        // 2xx 以外は配信失敗としてエラーを返す
        let status = response.status();
        if status.is_success() {
            tracing::info!(
                log_id = "LOG-WH-001",
                event = "webhook.delivered",
                event_type = %event_type,
                idempotency_key = %idempotency_key,
                status = status.as_u16(),
            );
            Ok(())
        } else {
            tracing::warn!(
                log_id = "LOG-WH-002",
                event = "webhook.delivery_failed",
                event_type = %event_type,
                idempotency_key = %idempotency_key,
                status = status.as_u16(),
            );
            Err(WebhookError::HttpStatus {
                status: status.as_u16(),
            })
        }
    }
}
