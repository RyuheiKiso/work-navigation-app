//! パスワードハッシュと認証情報集約
//!
//! 対応 §: ロードマップ §10.5.1 §11.4.1
//!
//! 平文パスワードは [`PasswordHash`] では扱わない（§11.4.1 STRIDE Information Disclosure）。
//! ハッシュ生成・検証は [`PasswordHasher`] trait を経由し、実装は adapter 層に置く。

use core::fmt;

use super::user::UserId;

/// パスワードハッシュ値オブジェクト
///
/// Argon2id の PHC エンコード文字列（`$argon2id$v=...`）を保持する。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PasswordHash(String);

impl PasswordHash {
    /// PHC エンコード文字列から `PasswordHash` を構築する
    ///
    /// # Errors
    /// `$argon2id$` で始まらない／200 文字を超える場合は不正。
    pub fn from_phc(phc: impl Into<String>) -> Result<Self, CredentialError> {
        let s: String = phc.into();
        if !s.starts_with("$argon2id$") {
            return Err(CredentialError::InvalidHashFormat);
        }
        if s.len() > 200 {
            return Err(CredentialError::InvalidHashFormat);
        }
        Ok(Self(s))
    }

    /// 内部 PHC 文字列を取得する
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// パスワードハッシュ生成・検証 trait
///
/// 実装は adapter 層（`argon2` crate を使う）。ドメイン層は本 trait のみ参照する。
pub trait PasswordHasher: Send + Sync {
    /// 平文パスワードからハッシュを生成する
    ///
    /// # Errors
    /// 実装エラー（リソース不足・乱数失敗）を `CredentialError::HasherFailure` として返す。
    fn hash(&self, plaintext: &str) -> Result<PasswordHash, CredentialError>;

    /// ハッシュと平文の一致を検証する
    fn verify(&self, hash: &PasswordHash, plaintext: &str) -> Result<bool, CredentialError>;
}

/// 認証情報（Aggregate ルート）
///
/// `UserId` と `PasswordHash` の組。
/// 認証時の照合は境界層で `PasswordHasher::verify` を呼ぶ。
#[derive(Debug, Clone)]
pub struct Credential {
    user_id: UserId,
    password_hash: PasswordHash,
}

impl Credential {
    /// 新しい Credential を生成する
    #[must_use]
    pub const fn new(user_id: UserId, password_hash: PasswordHash) -> Self {
        Self {
            user_id,
            password_hash,
        }
    }

    /// 利用者 ID を取得する
    #[must_use]
    pub const fn user_id(&self) -> &UserId {
        &self.user_id
    }

    /// パスワードハッシュを取得する
    #[must_use]
    pub const fn password_hash(&self) -> &PasswordHash {
        &self.password_hash
    }
}

/// 認証関連のエラー
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum CredentialError {
    /// パスワードハッシュ形式が不正
    InvalidHashFormat,
    /// パスワード照合失敗
    PasswordMismatch,
    /// トークン形式が不正
    InvalidToken,
    /// アカウントが無効化されている
    AccountDisabled,
    /// ハッシャ実装の失敗
    HasherFailure(&'static str),
}

impl fmt::Display for CredentialError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // §20.1「人を責めない」表現
        match self {
            CredentialError::InvalidHashFormat => write!(f, "ハッシュ形式が不正です"),
            CredentialError::PasswordMismatch => {
                write!(f, "ユーザ ID またはパスワードが一致しません")
            }
            CredentialError::InvalidToken => write!(f, "トークン形式が不正です"),
            CredentialError::AccountDisabled => write!(f, "アカウントが無効化されています"),
            CredentialError::HasherFailure(msg) => write!(f, "ハッシュ処理に失敗しました: {msg}"),
        }
    }
}

impl std::error::Error for CredentialError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::DomainError;

    #[test]
    fn password_hash_rejects_non_phc() {
        let r = PasswordHash::from_phc("plain");
        assert!(matches!(r, Err(CredentialError::InvalidHashFormat)));
    }

    #[test]
    fn password_hash_accepts_phc() {
        let phc = "$argon2id$v=19$m=4096,t=3,p=1$c2FsdHl0$Z3aXAo";
        assert!(PasswordHash::from_phc(phc).is_ok());
    }

    #[test]
    fn password_hash_boundary_200_ok_201_reject() {
        let prefix = "$argon2id$";
        let ok = format!("{prefix}{}", "a".repeat(200 - prefix.len()));
        assert_eq!(ok.len(), 200);
        assert!(PasswordHash::from_phc(ok).is_ok());
        let ng = format!("{prefix}{}", "a".repeat(201 - prefix.len()));
        assert_eq!(ng.len(), 201);
        assert!(matches!(
            PasswordHash::from_phc(ng),
            Err(CredentialError::InvalidHashFormat)
        ));
    }

    #[test]
    fn password_hash_as_str_preserves_value() {
        let phc = "$argon2id$v=19$m=4096,t=3,p=1$c2FsdHk$ZGVtbw";
        let h = PasswordHash::from_phc(phc).expect("valid");
        assert_eq!(h.as_str(), phc);
    }

    #[test]
    fn credential_error_display_renders() {
        let e1 = CredentialError::PasswordMismatch;
        assert!(format!("{e1}").contains("一致"));
        let e2 = CredentialError::AccountDisabled;
        assert!(format!("{e2}").contains("無効化"));
    }

    #[test]
    fn domain_error_display_renders() {
        let e = DomainError::PreconditionNotSatisfied;
        assert!(format!("{e}").contains("条件"));
    }
}
