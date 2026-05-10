//! セッショントークンと認証成功後のセッション
//!
//! 対応 §: ロードマップ §10.5
//!
//! トークン検証は境界層に委譲する。本ドメインでは「不透明な文字列」として扱う。

use super::credential::CredentialError;
use super::user::User;

/// セッショントークン値オブジェクト
///
/// JWT 文字列を保持する。検証は境界層に委譲する。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionToken(String);

impl SessionToken {
    /// 文字列から `SessionToken` を構築する
    ///
    /// # Errors
    /// 空または 4096 文字超は不正。
    pub fn new(value: impl Into<String>) -> Result<Self, CredentialError> {
        let v: String = value.into();
        if v.is_empty() {
            return Err(CredentialError::InvalidToken);
        }
        if v.len() > 4096 {
            return Err(CredentialError::InvalidToken);
        }
        Ok(Self(v))
    }

    /// 内部 &str を取得する
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// セッション情報
#[derive(Debug, Clone)]
pub struct Session {
    user: User,
    token: SessionToken,
}

impl Session {
    /// セッションを構築する
    #[must_use]
    pub const fn new(user: User, token: SessionToken) -> Self {
        Self { user, token }
    }

    /// 利用者を取得する
    #[must_use]
    pub const fn user(&self) -> &User {
        &self.user
    }

    /// トークンを取得する
    #[must_use]
    pub const fn token(&self) -> &SessionToken {
        &self.token
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_token_rejects_empty() {
        let r = SessionToken::new("");
        assert!(matches!(r, Err(CredentialError::InvalidToken)));
    }

    #[test]
    fn session_token_as_str_preserves_value() {
        let t = SessionToken::new("opaque-token").expect("valid");
        assert_eq!(t.as_str(), "opaque-token");
    }

    #[test]
    fn session_token_boundary_4096_ok_4097_reject() {
        let ok = "a".repeat(4096);
        assert!(SessionToken::new(ok).is_ok());
        let ng = "a".repeat(4097);
        assert!(matches!(SessionToken::new(ng), Err(CredentialError::InvalidToken)));
    }
}
