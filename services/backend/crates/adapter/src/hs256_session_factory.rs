//! HMAC-SHA256 セッションファクトリ
//!
//! 対応 §: ロードマップ §10.5 §10.3.1 §11.4.1 ADR-0010
//!
//! 短寿命の不透明トークンを **本物の HMAC-SHA256** で生成する。
//! トークン形式は `<base64url(payload)>.<base64url(hmac)>`。
//! payload は `<user_id>.<unix_ts>`。

// ドメイン
use wna_domain::{SessionToken, User};
// usecase trait
use wna_usecase::SessionFactory;
// HMAC + SHA-256
use hmac::{Hmac, Mac};
use sha2::Sha256;
// Base64URL（パディングなし、JWT 慣行）
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
// 標準
use std::time::{SystemTime, UNIX_EPOCH};

// HMAC-SHA256 の型エイリアス
type HmacSha256 = Hmac<Sha256>;

/// HMAC-SHA256 セッションファクトリ
#[derive(Clone)]
pub struct Hs256SessionFactory {
    /// 秘密鍵（環境変数で注入する想定、§11.4 STRIDE Information Disclosure 対策で
    /// メモリダンプを取られても秘密鍵が漏れないよう、利用者は OS Keystore／Vault から
    /// 取り出した値を渡すこと）
    secret: Vec<u8>,
}

/// セッション発行時のエラー
#[derive(Debug, thiserror::Error)]
pub enum Hs256Error {
    /// 時刻取得失敗（システムクロックが UNIX_EPOCH より前）
    #[error("時刻取得に失敗しました")]
    Clock,
    /// HMAC 鍵長エラー（HMAC は任意鍵長を受け取れるため通常発生しない）
    #[error("HMAC 鍵長エラー")]
    InvalidKey,
    /// トークン値オブジェクト構築失敗
    #[error("トークン構築に失敗しました")]
    Token,
}

impl Hs256SessionFactory {
    /// 秘密鍵から構築する
    #[must_use]
    pub fn new(secret: impl Into<Vec<u8>>) -> Self {
        // 鍵を保持する
        Self {
            secret: secret.into(),
        }
    }

    /// 内部署名計算（HMAC-SHA256 のバイト列を返す）
    fn sign(&self, payload: &[u8]) -> Result<Vec<u8>, Hs256Error> {
        // HMAC を秘密鍵で初期化
        let mut mac = <HmacSha256 as Mac>::new_from_slice(&self.secret)
            .map_err(|_| Hs256Error::InvalidKey)?;
        // payload を投入
        mac.update(payload);
        // ダイジェストを取得
        Ok(mac.finalize().into_bytes().to_vec())
    }

    /// 署名検証（同じ payload で計算した HMAC と一致するか）
    ///
    /// **注意**: 本メソッドは将来「セッショントークン検証ミドルウェア」で使う想定。
    pub fn verify_signature(&self, payload: &[u8], signature: &[u8]) -> bool {
        // 期待する署名
        let Ok(expected) = self.sign(payload) else {
            // 署名計算失敗は不一致扱い
            return false;
        };
        // 定数時間比較
        constant_time_eq(&expected, signature)
    }
}

/// 定数時間比較（タイミング攻撃対策、§11.4.1 STRIDE Spoofing）
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    // 長さが違えば不一致
    if a.len() != b.len() {
        return false;
    }
    // XOR を OR で蓄積し、最後に 0 か判定
    let mut acc: u8 = 0;
    for (x, y) in a.iter().zip(b.iter()) {
        acc |= x ^ y;
    }
    // 0 であれば一致
    acc == 0
}

impl SessionFactory for Hs256SessionFactory {
    type Error = Hs256Error;

    async fn issue(&self, user: &User) -> Result<SessionToken, Self::Error> {
        // 現在時刻（unix epoch 秒）
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| Hs256Error::Clock)?
            .as_secs();
        // payload 文字列（不透明トークン）
        let payload_str = format!("{}.{}", user.id().as_str(), now);
        // 署名
        let sig = self.sign(payload_str.as_bytes())?;
        // base64url で文字列化（パディングなし）
        let payload_b64 = URL_SAFE_NO_PAD.encode(payload_str.as_bytes());
        let sig_b64 = URL_SAFE_NO_PAD.encode(sig);
        // 連結
        let token = format!("{payload_b64}.{sig_b64}");
        // 値オブジェクトに射影
        SessionToken::new(token).map_err(|_| Hs256Error::Token)
    }
}

// =====================================================================
// 単体テスト
// =====================================================================

#[cfg(test)]
mod tests {
    // テスト対象
    use super::*;
    // ドメイン
    use wna_domain::UserId;

    // 同じ秘密鍵・同じ payload なら署名は再現可能
    #[test]
    fn sign_is_deterministic() {
        // ファクトリ
        let f = Hs256SessionFactory::new(b"unit-test-secret".to_vec());
        // payload
        let p = b"hello";
        // 2 回計算
        let s1 = f.sign(p).expect("ok");
        let s2 = f.sign(p).expect("ok");
        // 一致
        assert_eq!(s1, s2);
        // SHA-256 出力サイズは 32 bytes
        assert_eq!(s1.len(), 32);
    }

    // 異なる payload の署名は異なる
    #[test]
    fn sign_changes_with_payload() {
        let f = Hs256SessionFactory::new(b"key".to_vec());
        let s1 = f.sign(b"payload-1").expect("ok");
        let s2 = f.sign(b"payload-2").expect("ok");
        assert_ne!(s1, s2);
    }

    // verify_signature: 自分の署名は検証成功
    #[test]
    fn verify_signature_round_trips() {
        let f = Hs256SessionFactory::new(b"key".to_vec());
        let p = b"opaque-payload";
        let s = f.sign(p).expect("ok");
        assert!(f.verify_signature(p, &s));
    }

    // verify_signature: 改ざんされた署名は失敗
    #[test]
    fn verify_signature_fails_on_tamper() {
        let f = Hs256SessionFactory::new(b"key".to_vec());
        let p = b"x";
        let mut s = f.sign(p).expect("ok");
        // 1 byte だけ反転
        s[0] ^= 0x01;
        assert!(!f.verify_signature(p, &s));
    }

    // 異なる鍵で発行したトークンの検証は失敗
    #[test]
    fn different_keys_dont_match() {
        let fa = Hs256SessionFactory::new(b"key-A".to_vec());
        let fb = Hs256SessionFactory::new(b"key-B".to_vec());
        let p = b"x";
        let sig_a = fa.sign(p).expect("ok");
        assert!(!fb.verify_signature(p, &sig_a));
    }

    // issue: トークンが 2 部に分かれる（payload.signature）
    #[tokio::test]
    async fn issue_produces_two_part_token() {
        let f = Hs256SessionFactory::new(b"unit-test-secret".to_vec());
        let id = UserId::new("op-1").expect("valid");
        let user = User::new(id, "オペレータ A").expect("valid");
        let token = f.issue(&user).await.expect("ok");
        let parts: Vec<&str> = token.as_str().split('.').collect();
        assert_eq!(parts.len(), 2);
        // 先頭は base64url（user_id.timestamp の payload）
        let decoded = URL_SAFE_NO_PAD.decode(parts[0]).expect("decode");
        let s = String::from_utf8(decoded).expect("utf8");
        assert!(s.starts_with("op-1."));
    }
}
