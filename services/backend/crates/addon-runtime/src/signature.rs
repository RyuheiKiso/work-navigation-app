//! アドオン署名検証
//!
//! 対応 §: ロードマップ §17.6 §19.3 §27 F-004
//!
//! `*.wnaddon` の bundle 署名検証を抽象化する。
//! 実装側で cosign verify を呼ぶ（`cosign-rs` クレート、または外部プロセス）。
//! 本モジュールは **trait と検証経路** のみを提供し、実装は将来差し替え可能。

// thiserror
use thiserror::Error;

/// 署名検証エラー
#[derive(Debug, Error, Clone)]
pub enum SignatureVerificationError {
    /// 署名が存在しない（§19.3 必須）
    #[error("署名ファイルが存在しません: {0}")]
    Missing(String),
    /// OIDC issuer 不一致
    #[error("OIDC issuer が想定と一致しません: expected={expected}, actual={actual}")]
    IssuerMismatch {
        /// 期待する OIDC issuer
        expected: String,
        /// 実際の OIDC issuer
        actual: String,
    },
    /// 証明書 identity 不一致
    #[error("証明書 identity が想定と一致しません: expected={expected}, actual={actual}")]
    IdentityMismatch {
        /// 期待する識別子
        expected: String,
        /// 実際の識別子
        actual: String,
    },
    /// 暗号学的検証失敗（HMAC／RSA／ECDSA いずれかが破綻）
    #[error("暗号学的検証に失敗しました")]
    Cryptographic,
    /// バックエンド（cosign 等）の失敗
    #[error("検証バックエンド: {0}")]
    Backend(String),
}

/// 署名検証 trait
///
/// 実装は次のいずれか:
/// - `CosignVerifier`（外部 `cosign verify` プロセス／feature `cosign`）
/// - `NoopVerifier`（テスト用）
/// - 自前 `Hs256AddonVerifier` 等
pub trait AddonSignatureVerifier: Send + Sync {
    /// `*.wnaddon` のバイト列に対する署名／証明書を検証する
    fn verify(
        &self,
        addon_bytes: &[u8],
        signature: &[u8],
        certificate: &[u8],
    ) -> Result<(), SignatureVerificationError>;
}

// =====================================================================
// NoopVerifier: テスト・開発時の許可検証
// =====================================================================

/// 常に通過する検証器（テスト・開発専用）
///
/// **本番では絶対に使用しないこと**。`production` features 等で隔離する想定。
#[derive(Debug, Default, Clone)]
pub struct NoopVerifier;

impl AddonSignatureVerifier for NoopVerifier {
    fn verify(
        &self,
        _addon_bytes: &[u8],
        _signature: &[u8],
        _certificate: &[u8],
    ) -> Result<(), SignatureVerificationError> {
        // 常に OK
        Ok(())
    }
}

// =====================================================================
// StrictPolicyVerifier: 期待 OIDC identity／issuer をチェック
// =====================================================================

/// 厳格ポリシーの基本検証器
///
/// 期待値（OIDC identity／issuer）と実際の証明書フィールドを比較する。
/// 暗号学的検証は別実装に委譲する設計。
pub struct StrictPolicyVerifier {
    /// 期待する OIDC issuer（例: `https://token.actions.githubusercontent.com`）
    expected_issuer: String,
    /// 期待する identity（例: workflow URL）
    expected_identity: String,
    /// 暗号学的検証の委譲先
    crypto: Box<dyn AddonSignatureVerifier>,
}

impl StrictPolicyVerifier {
    /// 構築
    #[must_use]
    pub fn new(
        expected_issuer: impl Into<String>,
        expected_identity: impl Into<String>,
        crypto: Box<dyn AddonSignatureVerifier>,
    ) -> Self {
        // フィールドを保持
        Self {
            expected_issuer: expected_issuer.into(),
            expected_identity: expected_identity.into(),
            crypto,
        }
    }

    /// 証明書から OIDC issuer を抽出する（簡易実装）
    ///
    /// 本実装はテスト用に証明書の文字列内に `issuer=...` 形式で埋め込まれた値を抽出する。
    /// 本番では x509-parser 等で正規の証明書解析を行うこと。
    fn extract_issuer(certificate: &[u8]) -> Option<String> {
        // バイト列を文字列化（PEM 形式想定）
        let s = std::str::from_utf8(certificate).ok()?;
        // 単純な検索: "issuer=" の後ろから改行までを取る
        let after = s.find("issuer=")?;
        let rest = &s[after + 7..];
        let end = rest.find('\n').unwrap_or(rest.len());
        Some(rest[..end].trim().to_string())
    }

    /// 証明書から identity を抽出する（簡易実装）
    fn extract_identity(certificate: &[u8]) -> Option<String> {
        // バイト列を文字列化
        let s = std::str::from_utf8(certificate).ok()?;
        // "identity=" を検索
        let after = s.find("identity=")?;
        let rest = &s[after + 9..];
        let end = rest.find('\n').unwrap_or(rest.len());
        Some(rest[..end].trim().to_string())
    }
}

impl AddonSignatureVerifier for StrictPolicyVerifier {
    fn verify(
        &self,
        addon_bytes: &[u8],
        signature: &[u8],
        certificate: &[u8],
    ) -> Result<(), SignatureVerificationError> {
        // OIDC issuer チェック
        let issuer = Self::extract_issuer(certificate).ok_or_else(|| {
            // 証明書の解析自体が失敗
            SignatureVerificationError::Backend("証明書から issuer を抽出できません".to_string())
        })?;
        if issuer != self.expected_issuer {
            return Err(SignatureVerificationError::IssuerMismatch {
                expected: self.expected_issuer.clone(),
                actual: issuer,
            });
        }
        // identity チェック
        let identity = Self::extract_identity(certificate).ok_or_else(|| {
            // 証明書の解析自体が失敗
            SignatureVerificationError::Backend(
                "証明書から identity を抽出できません".to_string(),
            )
        })?;
        if identity != self.expected_identity {
            return Err(SignatureVerificationError::IdentityMismatch {
                expected: self.expected_identity.clone(),
                actual: identity,
            });
        }
        // 暗号学的検証を委譲
        self.crypto.verify(addon_bytes, signature, certificate)
    }
}

// =====================================================================
// 単体テスト
// =====================================================================

#[cfg(test)]
mod tests {
    // テスト対象
    use super::*;

    // NoopVerifier は常に Ok
    #[test]
    fn noop_passes_anything() {
        let v = NoopVerifier;
        assert!(v.verify(b"x", b"y", b"z").is_ok());
    }

    // StrictPolicyVerifier: issuer 不一致
    #[test]
    fn strict_rejects_issuer_mismatch() {
        let v = StrictPolicyVerifier::new(
            "https://token.actions.githubusercontent.com",
            "https://github.com/RyuheiKiso/work-navigation-app/.github/workflows/release.yml@refs/tags/v1.0.0",
            Box::new(NoopVerifier),
        );
        let cert = b"issuer=https://other.example.com\nidentity=foo\n";
        let r = v.verify(b"addon", b"sig", cert);
        assert!(matches!(r, Err(SignatureVerificationError::IssuerMismatch { .. })));
    }

    // StrictPolicyVerifier: identity 不一致
    #[test]
    fn strict_rejects_identity_mismatch() {
        let v = StrictPolicyVerifier::new(
            "https://token.actions.githubusercontent.com",
            "expected-identity",
            Box::new(NoopVerifier),
        );
        let cert = b"issuer=https://token.actions.githubusercontent.com\nidentity=other\n";
        let r = v.verify(b"addon", b"sig", cert);
        assert!(matches!(r, Err(SignatureVerificationError::IdentityMismatch { .. })));
    }

    // StrictPolicyVerifier: 全部一致で OK
    #[test]
    fn strict_accepts_matching_cert() {
        let v = StrictPolicyVerifier::new(
            "https://token.actions.githubusercontent.com",
            "expected-identity",
            Box::new(NoopVerifier),
        );
        let cert = b"issuer=https://token.actions.githubusercontent.com\nidentity=expected-identity\n";
        assert!(v.verify(b"addon", b"sig", cert).is_ok());
    }
}
