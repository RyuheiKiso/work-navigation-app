//! Argon2id パスワードハッシュ実装
//!
//! 対応 §: ロードマップ §10.5.1 §11.4.2 ADR-0004 ADR-0007 §27 F-008
//!
//! `wna_domain::PasswordHasher` を `argon2` crate で実装する。
//! ソルトは `rand_core::OsRng` から取得し、PHC 文字列で永続化する。

// ドメイン
use wna_domain::{CredentialError, PasswordHash, PasswordHasher};
// argon2
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash as PhcHash, SaltString},
    Argon2, PasswordHasher as ArgonPasswordHasher, PasswordVerifier,
};

/// Argon2id 実装
#[derive(Debug, Clone, Default)]
pub struct Argon2idHasher;

impl Argon2idHasher {
    /// 新しいハッシャを返す
    #[must_use]
    pub const fn new() -> Self {
        // Default と同じ振る舞い（パラメータは argon2 のデフォルト）
        Self
    }
}

impl PasswordHasher for Argon2idHasher {
    fn hash(&self, plaintext: &str) -> Result<PasswordHash, CredentialError> {
        // ランダムソルト生成
        let salt = SaltString::generate(&mut OsRng);
        // Argon2id 既定パラメータでハッシュ
        let argon = Argon2::default();
        // PHC 文字列を取得
        let phc = argon
            .hash_password(plaintext.as_bytes(), &salt)
            .map_err(|_| CredentialError::HasherFailure("hash_password に失敗しました"))?
            .to_string();
        // ドメイン値オブジェクトに射影
        PasswordHash::from_phc(phc)
    }

    fn verify(&self, hash: &PasswordHash, plaintext: &str) -> Result<bool, CredentialError> {
        // PHC を argon2 の `PasswordHash` 型にパース
        let parsed = PhcHash::new(hash.as_str())
            .map_err(|_| CredentialError::HasherFailure("PHC のパースに失敗しました"))?;
        // 検証
        let argon = Argon2::default();
        // verify_password は `Ok(())` で一致、`Err(_)` で不一致／エラー
        match argon.verify_password(plaintext.as_bytes(), &parsed) {
            // 一致
            Ok(()) => Ok(true),
            // 不一致（password_hash::Error::Password が返る）
            Err(argon2::password_hash::Error::Password) => Ok(false),
            // それ以外はハッシャ失敗
            Err(_) => Err(CredentialError::HasherFailure(
                "verify_password に失敗しました",
            )),
        }
    }
}

// =====================================================================
// 単体テスト
// =====================================================================

#[cfg(test)]
mod tests {
    // テスト対象
    use super::*;

    // hash → verify のラウンドトリップ
    #[test]
    fn hash_and_verify_round_trips() {
        // ハッシャ
        let h = Argon2idHasher::new();
        // 平文
        let plain = "correct horse battery staple";
        // ハッシュ
        let phc = h.hash(plain).expect("hash ok");
        // 同じ平文で検証
        let r = h.verify(&phc, plain).expect("verify ok");
        // 一致
        assert!(r);
    }

    // 異なる平文は false
    #[test]
    fn verify_returns_false_on_mismatch() {
        // ハッシャ
        let h = Argon2idHasher::new();
        // 平文
        let phc = h.hash("secret").expect("hash ok");
        // 別の平文
        let r = h.verify(&phc, "other").expect("verify ok");
        // 不一致
        assert!(!r);
    }
}
