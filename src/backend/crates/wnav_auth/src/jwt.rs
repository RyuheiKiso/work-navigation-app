// JWT RS256 発行・検証モジュール（MOD-BE-005 §2 / FNC-BE-014）
// JwtKeyStore は公開鍵の複数管理（90 日鍵ローテーション対応）と JWT 発行・検証を提供する。
// aud クレームによるバイナリ種別判定（terminal-api / master-api）を実施する。

use std::collections::HashMap;
use tokio::sync::RwLock;

use jsonwebtoken::{
    Algorithm, DecodingKey, EncodingKey, Header, TokenData, Validation, decode, decode_header,
    encode,
};

use crate::{
    claims::{JwtClaims, JwtIssueCmd},
    error::AuthError,
};

/// JWT 公開鍵ストアおよびオプションの署名鍵（発行用）。
///
/// ## 鍵ローテーション対応（90 日周期、24 時間 Grace Period）
/// - Grace Period 中は `keys` に旧鍵・新鍵の両方を格納し、どちらの kid でも検証可能にする。
/// - `add_key` / `remove_key` で動的に鍵を追加・削除する（BAT-010 連携）。
///
/// ## バイナリ種別判定
/// - 起動時に `expected_audience` を設定することで、自バイナリ宛て以外のトークンを拒否する。
/// - terminal-api: `expected_audience = "terminal-api"`
/// - master-api: `expected_audience = "master-api"`
pub struct JwtKeyStore {
    /// kid → 公開鍵 PEM のマップ（Grace Period 中は 2 エントリ）
    keys: RwLock<HashMap<String, String>>,
    /// 署名用秘密鍵（発行専用。master-api のみ保有、terminal-api は None）
    signing_key_pem: Option<String>,
    /// 発行時に使用する kid（将来の動的鍵更新 API 用に保持。現在は issue 呼び出し元が cmd.kid で指定する）
    _current_kid: Option<String>,
    /// 自バイナリの audience 文字列（検証時に aud クレームと照合する）
    expected_audience: String,
}

/// RSA 秘密鍵 PEM の構文が正しいかを確認する（鍵ローテーション前の事前検証用）。
///
/// 正しい PEM であれば `Ok(())`、不正な場合は `Err(AuthError::InvalidPrivateKey)` を返す。
pub fn validate_private_key_pem(pem: &str) -> Result<(), AuthError> {
    EncodingKey::from_rsa_pem(pem.as_bytes()).map_err(|_| AuthError::InvalidPrivateKey)?;
    Ok(())
}

impl JwtKeyStore {
    /// 検証専用の JwtKeyStore を生成する（terminal-api / master-api 共通）。
    ///
    /// # 引数
    /// - `public_key_pem`: RSA-4096 公開鍵（PEM 形式）
    /// - `kid`: 鍵 ID（例: "2026-Q2"）
    /// - `expected_audience`: 自バイナリの audience 文字列
    pub fn new(public_key_pem: &str, kid: &str, expected_audience: &str) -> Self {
        let mut map = HashMap::new();
        map.insert(kid.to_string(), public_key_pem.to_string());

        Self {
            keys: RwLock::new(map),
            signing_key_pem: None,
            _current_kid: None,
            expected_audience: expected_audience.to_string(),
        }
    }

    /// 発行・検証両用の JwtKeyStore を生成する（master-api のみ）。
    ///
    /// # 引数
    /// - `private_key_pem`: RSA-4096 秘密鍵（PEM 形式）
    /// - `public_key_pem`: RSA-4096 公開鍵（PEM 形式）
    /// - `kid`: 鍵 ID（例: "2026-Q2"）
    /// - `audience`: 自バイナリの audience 文字列
    pub fn with_signing_key(
        private_key_pem: &str,
        public_key_pem: &str,
        kid: &str,
        audience: &str,
    ) -> Self {
        let mut map = HashMap::new();
        map.insert(kid.to_string(), public_key_pem.to_string());

        Self {
            keys: RwLock::new(map),
            signing_key_pem: Some(private_key_pem.to_string()),
            _current_kid: Some(kid.to_string()),
            expected_audience: audience.to_string(),
        }
    }

    /// Grace Period 用: 追加の公開鍵を鍵ストアに登録する（BAT-010 連携）。
    pub async fn add_key(&self, kid: String, public_key_pem: String) {
        // 新しい鍵を追加し、Grace Period 中の二重検証を可能にする
        self.keys.write().await.insert(kid, public_key_pem);
    }

    /// 旧鍵を鍵ストアから削除する（Grace Period 終了後）。
    pub async fn remove_key(&self, kid: &str) {
        // 旧鍵を削除して Grace Period を終了する
        self.keys.write().await.remove(kid);
    }

    /// JWT を RS256 で検証し、Claims を返す（FNC-BE-014）。
    ///
    /// 検証内容:
    /// - RS256 署名
    /// - iss: "wnav.factory.example"
    /// - aud: self.expected_audience と一致すること
    /// - exp: 期限切れでないこと
    /// - kid: 鍵ストアに存在すること
    #[tracing::instrument(skip(self, token), err)]
    pub async fn verify(&self, token: &str) -> Result<JwtClaims, AuthError> {
        // JWT ヘッダから kid を取得して対応する公開鍵を選択する
        let header = decode_header(token)
            .map_err(|_| AuthError::InvalidToken("failed to decode JWT header".to_string()))?;

        let kid = header
            .kid
            .ok_or(AuthError::MissingKid)?;

        // 鍵ストアから kid に対応する公開鍵 PEM を取得する
        let keys = self.keys.read().await;
        let pem = keys
            .get(&kid)
            .ok_or_else(|| AuthError::UnknownKid(kid.clone()))?;

        let decoding_key = DecodingKey::from_rsa_pem(pem.as_bytes())
            .map_err(|_| AuthError::InvalidPublicKey)?;

        // aud・iss・exp を検証する
        let mut validation = Validation::new(Algorithm::RS256);
        validation.set_issuer(&["wnav.factory.example"]);
        validation.set_audience(&[&self.expected_audience]);
        validation.validate_exp = true;

        let token_data: TokenData<JwtClaims> =
            decode(token, &decoding_key, &validation).map_err(|e| {
                match e.kind() {
                    jsonwebtoken::errors::ErrorKind::ExpiredSignature => AuthError::TokenExpired,
                    jsonwebtoken::errors::ErrorKind::InvalidSignature => {
                        AuthError::InvalidSignature
                    }
                    jsonwebtoken::errors::ErrorKind::InvalidAudience => AuthError::InvalidAudience,
                    _ => AuthError::InvalidToken(e.to_string()),
                }
            })?;

        Ok(token_data.claims)
    }

    /// JWT を RS256 で発行する（ログインハンドラから呼び出し）。
    ///
    /// # エラー
    /// - 秘密鍵が設定されていない場合（検証専用インスタンスで呼び出した場合）は `InvalidPrivateKey`
    #[tracing::instrument(skip(self, cmd), err)]
    pub fn issue(&self, cmd: JwtIssueCmd, ttl_sec: i64) -> Result<String, AuthError> {
        // 秘密鍵が設定されていない場合は発行不可
        let private_pem = self
            .signing_key_pem
            .as_deref()
            .ok_or(AuthError::InvalidPrivateKey)?;

        let encoding_key = EncodingKey::from_rsa_pem(private_pem.as_bytes())
            .map_err(|_| AuthError::InvalidPrivateKey)?;

        // 現在時刻を基準に exp を計算する
        let now = chrono::Utc::now().timestamp();
        let jti = uuid::Uuid::now_v7();

        let claims = JwtClaims {
            sub: cmd.user_id,
            iss: "wnav.factory.example".to_string(),
            aud: cmd.audience,
            iat: now,
            exp: now + ttl_sec,
            roles: cmd.roles,
            factory_id: cmd.factory_id,
            device_id: cmd.device_id,
            jti,
            kid: cmd.kid.clone(),
        };

        let header = Header {
            alg: Algorithm::RS256,
            // 鍵ローテーション識別のため kid を JWT ヘッダに付与する
            kid: Some(cmd.kid),
            ..Default::default()
        };

        encode(&header, &claims, &encoding_key)
            .map_err(|e| AuthError::JwtEncodeError(e.to_string()))
    }
}
