// wnav_webhook クレート（MOD-BE-007）
//
// Webhook 配信サービス・HMAC-SHA256 ペイロード署名・受信側での署名検証を提供する。
// 本クレートは wnav_terminal_api バイナリのみが依存する（wnav_master_api は使用しない）。
//
// # 設計方針
// - 本クレートは 1 回の HTTP POST と署名生成に専念する（単一責任の原則）
// - リトライ・バックオフは wnav_outbox（MOD-BE-006）に委譲する
// - HMAC-SHA256 署名は "sha256={hex}" 形式で X-WNav-Signature ヘッダに付与する
// - 受信側は subtle::ConstantTimeEq で定数時間比較（タイミング攻撃防止）を行う
//
// # モジュール構成
// - `signature`: HMAC-SHA256 署名生成・検証・タイムスタンプ検証
// - `sender`: WebhookSender（HTTP POST + 署名ヘッダ付与）
// - `receiver`: WebhookReceiver（署名検証 + タイムスタンプ検証）
// - `error`: WebhookError 列挙型

// unsafe コードを禁止する（src/CLAUDE.md および src/backend/CLAUDE.md の必須要件）
#![forbid(unsafe_code)]
// Clippy の全 lint を有効化する（ワークスペース設定で deny 済みだが明示する）
#![deny(clippy::all, clippy::pedantic)]
// 例外: doc コメントのリンク省略は許容
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
// 例外: モジュール名重複は許容
#![allow(clippy::module_name_repetitions)]
// 例外: must_use 警告は許容
#![allow(clippy::must_use_candidate)]

pub mod error;
pub mod receiver;
pub mod sender;
pub mod signature;

// 主要な型を再エクスポートして使いやすくする
pub use error::WebhookError;
pub use receiver::WebhookReceiver;
pub use sender::{WebhookSender, WebhookSenderConfig};
pub use signature::{sign_payload, verify_signature, verify_timestamp};

// ─────────────────────────────────────────────────────────────────────────────
// ユニットテスト
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sign_payload_format() {
        // sign_payload() が "sha256={hex}" 形式で返ることを確認する
        let payload = b"test payload";
        let secret = "test_secret";
        let signature = sign_payload(payload, secret);

        assert!(
            signature.starts_with("sha256="),
            "expected 'sha256=' prefix, got: {}",
            signature
        );
        // hex 部分は 64 文字（SHA-256 = 256bit = 32byte = 64 hex chars）
        let hex_part = signature.strip_prefix("sha256=").unwrap();
        assert_eq!(hex_part.len(), 64, "expected 64 hex chars, got: {}", hex_part.len());
    }

    #[test]
    fn test_sign_payload_deterministic() {
        // 同じ payload + secret で常に同じ署名が生成されることを確認する
        let payload = b"deterministic test";
        let secret = "secret_key";

        let sig1 = sign_payload(payload, secret);
        let sig2 = sign_payload(payload, secret);
        assert_eq!(sig1, sig2, "same input should produce same signature");
    }

    #[test]
    fn test_verify_signature_success() {
        // 正しい秘密鍵・ペイロードで署名検証が成功することを確認する
        let payload = b"webhook payload";
        let secret = "correct_secret";

        let signature = sign_payload(payload, secret);
        let result = verify_signature(payload, secret, &signature);
        assert!(result.is_ok(), "valid signature should verify successfully");
    }

    #[test]
    fn test_verify_signature_wrong_secret() {
        // 異なる秘密鍵では署名検証が失敗することを確認する（定数時間比較）
        let payload = b"webhook payload";
        let correct_secret = "correct_secret";
        let wrong_secret = "wrong_secret";

        let signature = sign_payload(payload, correct_secret);
        let result = verify_signature(payload, wrong_secret, &signature);
        assert!(
            matches!(result, Err(WebhookError::InvalidSignature)),
            "wrong secret should cause InvalidSignature"
        );
    }

    #[test]
    fn test_verify_signature_tampered_payload() {
        // ペイロードが改竄された場合に署名検証が失敗することを確認する
        let original_payload = b"original payload";
        let tampered_payload = b"tampered payload";
        let secret = "secret";

        let signature = sign_payload(original_payload, secret);
        let result = verify_signature(tampered_payload, secret, &signature);
        assert!(
            matches!(result, Err(WebhookError::InvalidSignature)),
            "tampered payload should cause InvalidSignature"
        );
    }

    #[test]
    fn test_verify_signature_without_prefix() {
        // "sha256=" プレフィックスなしの hex 値でも検証できることを確認する
        let payload = b"test";
        let secret = "secret";

        let full_signature = sign_payload(payload, secret);
        let hex_only = full_signature.strip_prefix("sha256=").unwrap().to_string();

        // プレフィックスなしでも verify_signature は処理できる
        let result = verify_signature(payload, secret, &hex_only);
        assert!(result.is_ok(), "hex without prefix should also verify");
    }

    #[test]
    fn test_verify_timestamp_valid() {
        // 現在時刻付近のタイムスタンプは検証を通過することを確認する
        let now = chrono::Utc::now().timestamp();
        let result = signature::verify_timestamp(&now.to_string(), 300);
        assert!(result.is_ok(), "current timestamp should be valid");
    }

    #[test]
    fn test_verify_timestamp_too_old() {
        // 古いタイムスタンプ（6 分前）は拒否されることを確認する（リプレイ攻撃防止）
        let old_ts = chrono::Utc::now().timestamp() - 360; // 6 分前
        let result = signature::verify_timestamp(&old_ts.to_string(), 300);
        assert!(
            matches!(result, Err(WebhookError::RequestTimeout)),
            "old timestamp should cause RequestTimeout"
        );
    }

    #[test]
    fn test_verify_timestamp_future() {
        // 未来のタイムスタンプ（6 分後）は拒否されることを確認する
        let future_ts = chrono::Utc::now().timestamp() + 360; // 6 分後
        let result = signature::verify_timestamp(&future_ts.to_string(), 300);
        assert!(
            matches!(result, Err(WebhookError::RequestTimeout)),
            "future timestamp should cause RequestTimeout"
        );
    }

    #[test]
    fn test_webhook_receiver_full_verification() {
        // WebhookReceiver による署名・タイムスタンプ一括検証テスト
        let secret = "receiver_secret";
        let receiver = WebhookReceiver::new(secret.to_string());

        let payload = b"receiver test payload";
        let signature = sign_payload(payload, secret);
        let timestamp = chrono::Utc::now().timestamp().to_string();

        let result = receiver.verify_request(payload, &signature, Some(&timestamp));
        assert!(result.is_ok(), "valid signature and timestamp should pass");
    }

    #[test]
    fn test_webhook_receiver_invalid_signature() {
        // WebhookReceiver が不正な署名を拒否することを確認する
        let secret = "receiver_secret";
        let receiver = WebhookReceiver::new(secret.to_string());

        let payload = b"receiver test payload";
        let bad_signature = "sha256=000000000000000000000000000000000000000000000000000000000000bad1";

        let result = receiver.verify_request(payload, bad_signature, None);
        assert!(
            matches!(result, Err(WebhookError::InvalidSignature)),
            "invalid signature should be rejected"
        );
    }
}
