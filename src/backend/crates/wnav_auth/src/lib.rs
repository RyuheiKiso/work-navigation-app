// wnav_auth クレート（MOD-BE-005）
//
// JWT RS256 認証・RBAC 6 ロール・bcrypt パスワード管理・Tower ミドルウェアを提供する。
// 本クレートは wnav_terminal_api と wnav_master_api の両バイナリで共有される。
//
// # aud クレームによるバイナリ種別判定
// - JwtKeyStore::new() / with_signing_key() の `expected_audience` 引数で自バイナリを宣言する
// - "terminal-api" 宛てトークンは wnav_master_api で拒否される
// - "master-api" 宛てトークンは wnav_terminal_api で拒否される
//
// # モジュール構成
// - `claims`: JwtClaims・JwtIssueCmd 構造体
// - `jwt`: JwtKeyStore（RS256 発行・検証・鍵ローテーション）
// - `rbac`: ロールマーカー型・AuthenticatedUser<R>・ロール階層評価
// - `current_user`: CurrentUser（Request Extension に注入）
// - `middleware`: auth_middleware・auth_log_middleware（Tower ミドルウェア）
// - `password`: bcrypt ハッシュ化・検証
// - `error`: AuthError（RFC 7807 Problem Details 形式で HTTP レスポンス変換）

// unsafe コードを禁止する（src/CLAUDE.md および src/backend/CLAUDE.md の必須要件）
#![forbid(unsafe_code)]
// 例外: doc コメントのリンク省略は許容（テスト補助関数等）
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
// 例外: モジュール名重複は許容（例: error::AuthError）
#![allow(clippy::module_name_repetitions)]
// 例外: must_use 警告は許容
#![allow(clippy::must_use_candidate)]

pub mod claims;
pub mod current_user;
pub mod error;
pub mod jwt;
pub mod middleware;
pub mod password;
pub mod rbac;

// 主要な型を再エクスポートして使いやすくする
pub use claims::{JwtClaims, JwtIssueCmd};
pub use current_user::CurrentUser;
pub use error::AuthError;
pub use jwt::{JwtKeyStore, validate_private_key_pem};
pub use middleware::{auth_log_middleware, auth_middleware};
pub use password::{hash_password, verify_password};
pub use rbac::{
    AdminRole, ApproverRole, AuditorRole, AuthenticatedUser, MasterEditorRole, OperatorRole, Role,
    SupervisorRole, effective_role_names, evaluate_roles,
};

// ─────────────────────────────────────────────────────────────────────────────
// ユニットテスト
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    /// テスト用の RSA-4096 鍵ペアを生成するヘルパー。
    /// 本番では環境変数から取得する。
    fn generate_test_rsa_keys() -> (String, String) {
        use std::process::Command;

        // openssl で RSA-4096 鍵ペアを生成する
        // CI 環境では openssl コマンドが必須
        let private_output = Command::new("openssl")
            .args(["genrsa", "4096"])
            .output()
            .expect("openssl genrsa failed");

        let private_pem = String::from_utf8(private_output.stdout).expect("invalid UTF-8");

        let public_output = Command::new("openssl")
            .args(["rsa", "-pubout"])
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .spawn()
            .and_then(|mut child| {
                use std::io::Write;
                child
                    .stdin
                    .as_mut()
                    .expect("stdin")
                    .write_all(private_pem.as_bytes())
                    .expect("write failed");
                child.wait_with_output()
            })
            .expect("openssl rsa -pubout failed");

        let public_pem = String::from_utf8(public_output.stdout).expect("invalid UTF-8");
        (private_pem, public_pem)
    }

    #[tokio::test]
    async fn test_jwt_issue_and_verify_success() {
        // JWT 発行・検証の正常系テスト
        let (private_pem, public_pem) = generate_test_rsa_keys();
        let kid = "test-2026-Q2";
        let audience = "terminal-api";

        let key_store = JwtKeyStore::with_signing_key(&private_pem, &public_pem, kid, audience);

        let cmd = JwtIssueCmd {
            user_id: Uuid::now_v7(),
            roles: vec!["operator".to_string()],
            factory_id: Uuid::now_v7(),
            device_id: Some(Uuid::now_v7()),
            kid: kid.to_string(),
            audience: audience.to_string(),
        };

        // JWT を発行して検証する
        let token = key_store.issue(cmd, 28800).expect("JWT issue failed");
        let claims = key_store.verify(&token).await.expect("JWT verify failed");

        // Claims の内容を確認する
        assert_eq!(claims.aud, "terminal-api");
        assert_eq!(claims.roles, vec!["operator".to_string()]);
        assert_eq!(claims.kid, kid);
        assert!(claims.device_id.is_some());
    }

    #[tokio::test]
    async fn test_jwt_audience_mismatch() {
        // aud ミスマッチテスト: terminal-api トークンを master-api で検証すると拒否される
        let (private_pem, public_pem) = generate_test_rsa_keys();
        let kid = "test-2026-Q2";

        // terminal-api 向けにトークンを発行する
        let issuer_store =
            JwtKeyStore::with_signing_key(&private_pem, &public_pem, kid, "terminal-api");

        let cmd = JwtIssueCmd {
            user_id: Uuid::now_v7(),
            roles: vec!["operator".to_string()],
            factory_id: Uuid::now_v7(),
            device_id: None,
            kid: kid.to_string(),
            audience: "terminal-api".to_string(),
        };

        let token = issuer_store.issue(cmd, 28800).expect("JWT issue failed");

        // master-api として検証すると InvalidAudience エラーになることを確認する
        let verifier_store = JwtKeyStore::new(&public_pem, kid, "master-api");
        let result = verifier_store.verify(&token).await;

        assert!(
            matches!(result, Err(AuthError::InvalidAudience)),
            "expected InvalidAudience, got {result:?}"
        );
    }

    #[tokio::test]
    async fn test_jwt_expired() {
        // 期限切れ JWT のテスト: TTL を -1 にして即座に期限切れにする
        let (private_pem, public_pem) = generate_test_rsa_keys();
        let kid = "test-2026-Q2";
        let audience = "terminal-api";

        let key_store = JwtKeyStore::with_signing_key(&private_pem, &public_pem, kid, audience);

        let cmd = JwtIssueCmd {
            user_id: Uuid::now_v7(),
            roles: vec!["operator".to_string()],
            factory_id: Uuid::now_v7(),
            device_id: None,
            kid: kid.to_string(),
            audience: audience.to_string(),
        };

        // TTL を -3600 秒（1 時間前に期限切れ）で発行する
        let token = key_store.issue(cmd, -3600).expect("JWT issue failed");
        let result = key_store.verify(&token).await;

        assert!(
            matches!(result, Err(AuthError::TokenExpired)),
            "expected TokenExpired, got {result:?}"
        );
    }

    #[test]
    fn test_rbac_role_hierarchy() {
        // ロール階層テスト: system_admin は全ロール権限を包含する
        assert!(evaluate_roles(&["system_admin".to_string()], "operator"));
        assert!(evaluate_roles(&["system_admin".to_string()], "supervisor"));
        assert!(evaluate_roles(
            &["system_admin".to_string()],
            "quality_admin"
        ));
        assert!(evaluate_roles(&["system_admin".to_string()], "executive"));

        // supervisor は operator を包含するが quality_admin は含まない
        assert!(evaluate_roles(&["supervisor".to_string()], "operator"));
        assert!(!evaluate_roles(
            &["supervisor".to_string()],
            "quality_admin"
        ));

        // operator は最下位ロール（他のロールを包含しない）
        assert!(evaluate_roles(&["operator".to_string()], "operator"));
        assert!(!evaluate_roles(&["operator".to_string()], "supervisor"));
        assert!(!evaluate_roles(&["operator".to_string()], "system_admin"));
    }

    #[test]
    fn test_password_hash_and_verify() {
        // bcrypt パスワードハッシュ化・検証テスト
        let plain = "secure_password_123!";
        let hash = hash_password(plain).expect("hash failed");

        // ハッシュが bcrypt 形式であることを確認する
        assert!(
            hash.starts_with("$2b$"),
            "expected bcrypt hash, got: {hash}"
        );

        // 正しいパスワードで検証が通ることを確認する
        assert!(
            verify_password(plain, &hash).expect("verify failed"),
            "correct password should verify successfully"
        );

        // 誤ったパスワードで検証が失敗することを確認する
        assert!(
            !verify_password("wrong_password", &hash).expect("verify failed"),
            "wrong password should not verify"
        );
    }

    #[test]
    fn test_role_name_constants() {
        // ロールマーカー型の role_name() が正しい文字列を返すことを確認する
        assert_eq!(AdminRole::role_name(), "system_admin");
        assert_eq!(AuditorRole::role_name(), "executive");
        assert_eq!(ApproverRole::role_name(), "quality_admin");
        assert_eq!(MasterEditorRole::role_name(), "master_admin");
        assert_eq!(SupervisorRole::role_name(), "supervisor");
        assert_eq!(OperatorRole::role_name(), "operator");
    }
}
