// MOD-SH-004: ApiClient（親機連携）
// OAuth 2.1 Client Credentials フローで親機 API に送信する reqwest ベースのクライアント。
// IF-002 アウトバウンド: 実績データを親機 Inbound API に送信する。

use reqwest::Client;
use serde::Deserialize;

/// 親機 API クライアントの設定。
#[derive(Debug, Clone, Deserialize)]
pub struct ParentApiConfig {
    /// 親機 API のベース URL（例: "https://parent-system.example.com"）
    pub base_url: String,
    /// OAuth 2.1 Client Credentials の client_id
    pub client_id: String,
    /// OAuth 2.1 Client Credentials の client_secret
    pub client_secret: String,
    /// トークンエンドポイント URL（例: "https://auth.example.com/oauth/token"）
    pub token_endpoint: String,
    /// HTTP リクエストのタイムアウト（秒）
    pub timeout_secs: u64,
}

/// ApiClient のエラー型。
#[derive(Debug, thiserror::Error)]
pub enum ApiClientError {
    /// HTTP リクエストエラー（接続失敗・タイムアウト等）
    #[error("HTTP エラーが発生しました: {0}")]
    Http(String),

    /// レスポンスのデシリアライズエラー
    #[error("レスポンスのデシリアライズに失敗しました: {0}")]
    Deserialize(String),

    /// 認証エラー（トークン取得失敗）
    #[error("OAuth 2.1 トークンの取得に失敗しました: {0}")]
    Auth(String),
}

/// OAuth 2.1 Client Credentials フローのトークンレスポンス型（内部使用）。
#[derive(Debug, Deserialize)]
struct TokenResponse {
    /// アクセストークン（Bearer）
    access_token: String,
    /// トークン種別（通常 "Bearer"）
    #[allow(dead_code)]
    token_type: String,
}

/// 親機 API クライアント（IF-002 アウトバウンド）。
///
/// OAuth 2.1 Client Credentials フローでアクセストークンを取得・キャッシュし、
/// 親機の Inbound API に実績データを送信する。
///
/// # スレッドセーフ
/// `RwLock` を使用してトークンキャッシュをスレッドセーフに管理する。
pub struct ParentSystemApiClient {
    /// reqwest HTTP クライアント（タイムアウト設定済み）
    client: Client,
    /// 親機 API のベース URL
    base_url: String,
    /// OAuth 2.1 アクセストークンのキャッシュ（None = 未取得または期限切れ）
    access_token: tokio::sync::RwLock<Option<String>>,
    /// OAuth 2.1 client_id
    client_id: String,
    /// OAuth 2.1 client_secret
    client_secret: String,
    /// トークンエンドポイント URL
    token_endpoint: String,
}

impl ParentSystemApiClient {
    /// 設定から親機 API クライアントを初期化する。
    pub fn new(config: &ParentApiConfig) -> Self {
        // タイムアウト設定付きで reqwest クライアントを構築する
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(config.timeout_secs))
            .build()
            .unwrap_or_else(|_| Client::new());

        Self {
            client,
            base_url: config.base_url.clone(),
            access_token: tokio::sync::RwLock::new(None),
            client_id: config.client_id.clone(),
            client_secret: config.client_secret.clone(),
            token_endpoint: config.token_endpoint.clone(),
        }
    }

    /// OAuth 2.1 Client Credentials でアクセストークンを取得・キャッシュする。
    ///
    /// トークンがキャッシュ済みの場合はそのまま返す。
    /// キャッシュが空の場合はトークンエンドポイントにリクエストしてキャッシュに保存する。
    ///
    /// # 注意
    /// 本実装ではトークンの期限切れを検出しない（シンプル実装）。
    /// 本番では expires_in フィールドを確認してキャッシュ期限を管理することを推奨する。
    async fn ensure_token(&self) -> Result<String, ApiClientError> {
        // キャッシュからトークンを読み込む（読み取りロックを使用する）
        {
            let read_guard = self.access_token.read().await;
            if let Some(token) = read_guard.as_ref() {
                return Ok(token.clone());
            }
        }

        // トークンエンドポイントに Client Credentials リクエストを送信する
        let response = self
            .client
            .post(&self.token_endpoint)
            .form(&[
                ("grant_type", "client_credentials"),
                ("scope", "wnav.outbox.write"),
                ("client_id", &self.client_id),
                ("client_secret", &self.client_secret),
            ])
            .send()
            .await
            .map_err(|e| ApiClientError::Http(e.to_string()))?;

        // トークンレスポンスをデシリアライズする
        let token_response: TokenResponse = response
            .json()
            .await
            .map_err(|e| ApiClientError::Deserialize(e.to_string()))?;

        // トークンをキャッシュに保存する（書き込みロックを使用する）
        let new_token = token_response.access_token.clone();
        *self.access_token.write().await = Some(new_token.clone());

        Ok(new_token)
    }

    /// アウトバウンド実績データを親機 Inbound API に送信する。
    ///
    /// # 引数
    /// - `payload`: 送信するデータの JSON
    /// - `idempotency_key`: 冪等性キー（UUID v7 文字列）—— リプレイ防止に使用する
    ///
    /// # 送信先
    /// `POST {base_url}/api/v1/sync/outbox/inbound`
    pub async fn post_outbox_inbound(
        &self,
        payload: &serde_json::Value,
        idempotency_key: &str,
    ) -> Result<(), ApiClientError> {
        // アクセストークンを取得（キャッシュがあればキャッシュを使用する）
        let token = self.ensure_token().await?;

        // 親機 Inbound API にリクエストを送信する
        self.client
            .post(format!("{}/api/v1/sync/outbox/inbound", self.base_url))
            .bearer_auth(&token)
            .header("Idempotency-Key", idempotency_key)
            .json(payload)
            .send()
            .await
            .map_err(|e| ApiClientError::Http(e.to_string()))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parent_api_config_construction() {
        // ParentApiConfig の構築が正しいことを確認する
        let config = ParentApiConfig {
            base_url: "https://parent.example.com".to_string(),
            client_id: "test-client".to_string(),
            client_secret: "test-secret".to_string(),
            token_endpoint: "https://auth.example.com/token".to_string(),
            timeout_secs: 30,
        };

        assert_eq!(config.base_url, "https://parent.example.com");
        assert_eq!(config.timeout_secs, 30);
    }

    #[test]
    fn test_api_client_error_display() {
        // ApiClientError の Display 実装が正しいことを確認する
        let err = ApiClientError::Http("Connection refused".to_string());
        let msg = err.to_string();
        assert!(msg.contains("HTTP エラーが発生しました"));
        assert!(msg.contains("Connection refused"));

        let err = ApiClientError::Auth("Invalid client credentials".to_string());
        let msg = err.to_string();
        assert!(msg.contains("OAuth 2.1 トークンの取得に失敗しました"));
    }

    #[test]
    fn test_parent_api_client_new() {
        // ParentSystemApiClient の初期化が正しいことを確認する
        let config = ParentApiConfig {
            base_url: "https://parent.example.com".to_string(),
            client_id: "test-client".to_string(),
            client_secret: "test-secret".to_string(),
            token_endpoint: "https://auth.example.com/token".to_string(),
            timeout_secs: 10,
        };

        // パニックなく初期化できることを確認する
        let _client = ParentSystemApiClient::new(&config);
    }
}
