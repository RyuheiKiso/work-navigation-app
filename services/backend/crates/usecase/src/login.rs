//! ログインユースケース
//!
//! 対応 §: ロードマップ §10.5 §10.5.0 §10.5.1 §11.4.1 §27 F-006
//!
//! ID＋パスワード認証（ADR-0007）。
//! 副作用は `CredentialRepository` ／`SessionFactory` の trait 経由で adapter 層へ委譲する。

// ドメイン依存
use wna_domain::{
    Credential, CredentialError, PasswordHasher, Session, SessionToken, User, UserId,
};

// =====================================================================
// CredentialRepository（trait）
// =====================================================================

/// 認証情報リポジトリ
pub trait CredentialRepository: Send + Sync {
    /// 実装固有エラー
    type Error: std::error::Error + Send + Sync + 'static;

    /// `UserId` で `Credential` を取得する
    fn find_credential(
        &self,
        user_id: &UserId,
    ) -> impl std::future::Future<Output = Result<Option<Credential>, Self::Error>> + Send;

    /// `UserId` で `User` を取得する
    fn find_user(
        &self,
        user_id: &UserId,
    ) -> impl std::future::Future<Output = Result<Option<User>, Self::Error>> + Send;
}

// =====================================================================
// SessionFactory（trait）
// =====================================================================

/// セッショントークン発行
///
/// JWT 等の発行は実装依存。短寿命設計（§10.3.1）の責務は実装側にある。
pub trait SessionFactory: Send + Sync {
    /// 実装固有エラー
    type Error: std::error::Error + Send + Sync + 'static;

    /// `User` から `SessionToken` を発行する
    fn issue(
        &self,
        user: &User,
    ) -> impl std::future::Future<Output = Result<SessionToken, Self::Error>> + Send;
}

// =====================================================================
// LoginCommand
// =====================================================================

/// ログインコマンド
#[derive(Debug, Clone)]
pub struct LoginCommand {
    /// 利用者 ID
    pub user_id: UserId,
    /// 平文パスワード（usecase の境界以降では保持しない）
    pub plaintext_password: String,
}

// =====================================================================
// LoginError
// =====================================================================

/// ログインユースケースのエラー
#[derive(Debug)]
pub enum LoginError<C, S>
where
    C: std::error::Error + Send + Sync + 'static,
    S: std::error::Error + Send + Sync + 'static,
{
    /// 利用者が存在しない
    UserNotFound,
    /// 認証情報が存在しない
    CredentialNotFound,
    /// アカウント無効化（§10.5.1）
    AccountDisabled,
    /// パスワード照合失敗
    PasswordMismatch,
    /// 認証情報リポジトリエラー
    CredentialRepository(C),
    /// セッション発行エラー
    SessionFactory(S),
    /// ハッシャ実装エラー
    Hasher(CredentialError),
}

// Display 実装（境界層でのログ用）
impl<C, S> std::fmt::Display for LoginError<C, S>
where
    C: std::error::Error + Send + Sync + 'static,
    S: std::error::Error + Send + Sync + 'static,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // バリアントごとに分岐
        match self {
            LoginError::UserNotFound => write!(f, "利用者が見つかりません"),
            LoginError::CredentialNotFound => write!(f, "認証情報が見つかりません"),
            LoginError::AccountDisabled => write!(f, "アカウントが無効化されています"),
            LoginError::PasswordMismatch => {
                write!(f, "ユーザ ID またはパスワードが一致しません")
            }
            LoginError::CredentialRepository(e) => write!(f, "認証情報リポジトリ: {e}"),
            LoginError::SessionFactory(e) => write!(f, "セッション発行: {e}"),
            LoginError::Hasher(e) => write!(f, "ハッシュ処理: {e}"),
        }
    }
}

// Error 実装
impl<C, S> std::error::Error for LoginError<C, S>
where
    C: std::error::Error + Send + Sync + 'static,
    S: std::error::Error + Send + Sync + 'static,
{
}

// =====================================================================
// LoginUseCase 実装
// =====================================================================

/// ログインユースケース
pub struct LoginUseCase<R, F, H>
where
    R: CredentialRepository,
    F: SessionFactory,
    H: PasswordHasher,
{
    /// 認証情報リポジトリ
    repository: R,
    /// セッション発行
    session_factory: F,
    /// パスワードハッシャ
    hasher: H,
}

impl<R, F, H> LoginUseCase<R, F, H>
where
    R: CredentialRepository,
    F: SessionFactory,
    H: PasswordHasher,
{
    /// コンストラクタ（DI）
    pub const fn new(repository: R, session_factory: F, hasher: H) -> Self {
        // フィールドを保持する
        Self {
            repository,
            session_factory,
            hasher,
        }
    }

    /// ログインを実行する
    ///
    /// # Errors
    /// 利用者未存在／無効化／パスワード不一致／リポジトリ・発行エラー。
    pub async fn execute(
        &self,
        cmd: LoginCommand,
    ) -> Result<Session, LoginError<R::Error, F::Error>> {
        // 利用者プロフィールを取得
        let user = self
            .repository
            .find_user(&cmd.user_id)
            .await
            .map_err(LoginError::CredentialRepository)?
            .ok_or(LoginError::UserNotFound)?;

        // 無効化チェック（§10.5.1）
        if !user.is_enabled() {
            return Err(LoginError::AccountDisabled);
        }

        // 認証情報を取得
        let credential = self
            .repository
            .find_credential(&cmd.user_id)
            .await
            .map_err(LoginError::CredentialRepository)?
            .ok_or(LoginError::CredentialNotFound)?;

        // パスワード検証
        let matched = self
            .hasher
            .verify(credential.password_hash(), &cmd.plaintext_password)
            .map_err(LoginError::Hasher)?;

        // 一致しない場合はエラー（§20.1: 「人を責めない」表現にしない；具体性は保持）
        if !matched {
            return Err(LoginError::PasswordMismatch);
        }

        // セッション発行
        let token = self
            .session_factory
            .issue(&user)
            .await
            .map_err(LoginError::SessionFactory)?;

        // セッションを返す
        Ok(Session::new(user, token))
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
    use wna_domain::{PasswordHash, User};

    // メモリ Repository
    struct MemRepo {
        // 1 利用者
        user: User,
        // 認証情報
        credential: Credential,
    }

    // Repository 用エラー
    #[derive(Debug)]
    struct E;
    impl std::fmt::Display for E {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            // 表示
            write!(f, "mem error")
        }
    }
    impl std::error::Error for E {}

    impl CredentialRepository for MemRepo {
        type Error = E;
        async fn find_credential(
            &self,
            user_id: &UserId,
        ) -> Result<Option<Credential>, Self::Error> {
            // ID 一致時のみ返す
            if user_id == self.credential.user_id() {
                Ok(Some(self.credential.clone()))
            } else {
                Ok(None)
            }
        }
        async fn find_user(&self, user_id: &UserId) -> Result<Option<User>, Self::Error> {
            // ID 一致時のみ
            if user_id == self.user.id() {
                Ok(Some(self.user.clone()))
            } else {
                Ok(None)
            }
        }
    }

    // セッションファクトリ
    struct MemSession;
    impl SessionFactory for MemSession {
        type Error = E;
        async fn issue(&self, user: &User) -> Result<SessionToken, Self::Error> {
            // 簡易トークン
            SessionToken::new(format!("session-{}", user.id())).map_err(|_| E)
        }
    }

    // ハッシャ（決め打ち）
    struct StaticHasher {
        // 想定平文
        plaintext: String,
    }

    impl PasswordHasher for StaticHasher {
        fn hash(&self, _plaintext: &str) -> Result<PasswordHash, CredentialError> {
            // PHC らしい文字列を返す
            PasswordHash::from_phc("$argon2id$v=19$m=4096,t=3,p=1$c2FsdHk$ZGVtbw")
        }
        fn verify(
            &self,
            _hash: &PasswordHash,
            plaintext: &str,
        ) -> Result<bool, CredentialError> {
            // 想定平文と一致
            Ok(plaintext == self.plaintext)
        }
    }

    // テスト: 妥当な認証
    #[tokio::test]
    async fn login_succeeds_with_valid_credentials() {
        // ID
        let id = UserId::new("op-1").expect("valid");
        // ユーザ
        let user = User::new(id.clone(), "オペレータ A").expect("valid");
        // 認証情報
        let hash = PasswordHash::from_phc("$argon2id$v=19$m=4096,t=3,p=1$c2FsdHk$ZGVtbw")
            .expect("valid");
        let cred = Credential::new(id.clone(), hash);
        // メモリリポジトリ
        let repo = MemRepo { user, credential: cred };
        // セッションファクトリ
        let sf = MemSession;
        // ハッシャ（want = "secret"）
        let h = StaticHasher { plaintext: "secret".to_string() };
        // ユースケース
        let uc = LoginUseCase::new(repo, sf, h);
        // コマンド
        let cmd = LoginCommand {
            user_id: id,
            plaintext_password: "secret".to_string(),
        };
        // 実行
        let session = uc.execute(cmd).await.expect("ok");
        // トークンが発行されている
        assert!(session.token().as_str().starts_with("session-"));
    }

    // テスト: パスワード不一致
    #[tokio::test]
    async fn login_rejects_wrong_password() {
        // 共通セットアップ
        let id = UserId::new("op-1").expect("valid");
        let user = User::new(id.clone(), "オペレータ A").expect("valid");
        let hash = PasswordHash::from_phc("$argon2id$v=19$m=4096,t=3,p=1$c2FsdHk$ZGVtbw")
            .expect("valid");
        let cred = Credential::new(id.clone(), hash);
        let repo = MemRepo { user, credential: cred };
        let sf = MemSession;
        // 想定平文と異なるものを渡す
        let h = StaticHasher { plaintext: "secret".to_string() };
        let uc = LoginUseCase::new(repo, sf, h);
        // 不一致パスワード
        let cmd = LoginCommand {
            user_id: id,
            plaintext_password: "WRONG".to_string(),
        };
        // 実行
        let r = uc.execute(cmd).await;
        // パスワード不一致エラー
        assert!(matches!(r, Err(LoginError::PasswordMismatch)));
    }

    // テスト: アカウント無効化
    #[tokio::test]
    async fn login_rejects_disabled_account() {
        // ユーザを無効化
        let id = UserId::new("op-1").expect("valid");
        let mut user = User::new(id.clone(), "オペレータ A").expect("valid");
        user.disable();
        let hash = PasswordHash::from_phc("$argon2id$v=19$m=4096,t=3,p=1$c2FsdHk$ZGVtbw")
            .expect("valid");
        let cred = Credential::new(id.clone(), hash);
        let repo = MemRepo { user, credential: cred };
        let sf = MemSession;
        let h = StaticHasher { plaintext: "secret".to_string() };
        let uc = LoginUseCase::new(repo, sf, h);
        let cmd = LoginCommand {
            user_id: id,
            plaintext_password: "secret".to_string(),
        };
        // 実行
        let r = uc.execute(cmd).await;
        // 無効化エラー
        assert!(matches!(r, Err(LoginError::AccountDisabled)));
    }
}
