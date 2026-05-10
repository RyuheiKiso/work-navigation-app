//! 認証ドメイン
//!
//! 対応 §: ロードマップ §10.5 §10.5.0 §10.5.1 §11.4.1 §27 F-006 §29
//!
//! ユーザ ID＋パスワード認証（ADR-0007）の最小ドメインモデル。
//! Argon2id 派生鍵の **ハッシュ生成・検証** はこの層では trait（`PasswordHasher`）として
//! 抽象化し、実装は adapter 層に委譲する（§9.4 副作用は境界層に局在化）。

pub mod credential;
pub mod session;
pub mod user;

pub use credential::{Credential, CredentialError, PasswordHash, PasswordHasher};
pub use session::{Session, SessionToken};
pub use user::{User, UserId};
