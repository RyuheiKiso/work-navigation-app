//! # `wnav_config`
//!
//! 作業ナビゲーションシステムのバックエンド設定管理クレート。
//! YAML ファイル + 環境変数 + `secret_ref` 解決を組み合わせて
//! バイナリ別に型安全な設定構造体を返す。
//!
//! ## 使用例
//!
//! ```no_run
//! // wnav_terminal_api の起動時に呼び出す
//! let config = wnav_config::load_terminal_api().unwrap_or_else(|e| {
//!     eprintln!("FATAL: configuration load failed: {e}");
//!     std::process::exit(78);
//! });
//! ```

// unsafe コードはネイティブライブラリとの FFI が必要な場合のみ許可する（ADR に根拠必須）
#![forbid(unsafe_code)]
// コード品質を一貫して高水準に保つため clippy の全警告をエラーとして扱う
#![deny(clippy::all, clippy::pedantic)]
// 日本語コメント内の英語識別子は意図的にバッククォートなしで記載する
#![allow(clippy::doc_markdown)]
// ConfigError は figment::Error の Box 化で可能な限りサイズを抑えている
#![allow(clippy::result_large_err)]
// missing_errors_doc は公開 API の doc コメントで個別に対処する
#![allow(clippy::missing_errors_doc)]
// must_use は expose() など意図的に省略する場合がある
#![allow(clippy::must_use_candidate)]
// workspace Cargo.toml の lint_groups_priority は既存の workspace 設定の問題であり
// wnav_config クレート自体の責任ではない
#![allow(clippy::lint_groups_priority)]

pub mod error;
pub mod profile;
pub mod redact;
pub mod schema;
pub mod secret_ref;
pub mod sources;
pub mod validation;

// 利用者が impl ブロックの中まで辿る必要がないよう最上位から再エクスポートする
pub use error::ConfigError;
pub use schema::{MasterApiConfig, TerminalApiConfig};

/// `wnav_terminal_api` 起動時に呼び出す設定ロード関数
///
/// YAML ファイル読込 → `secret_ref` 解決 → バリデーション を順に実行する。
/// `WNAV_PROFILE` 未設定・ファイル欠損・`schema_version` 不一致・`secret_ref` 解決失敗の
/// いずれかが発生した場合は `ConfigError` を返す（fail-fast 設計）。
///
/// # Errors
///
/// - [`ConfigError::ProfileNotSet`] — `WNAV_PROFILE` 環境変数が未設定
/// - [`ConfigError::FileNotFound`] — YAML ファイルが存在しない
/// - [`ConfigError::SchemaVersionMismatch`] — `schema_version` が 1 以外
/// - [`ConfigError::Extract`] — figment のデシリアライズ失敗
/// - [`ConfigError::SecretRefResolution`] — `secret_ref` の解決失敗
/// - [`ConfigError::InvalidValue`] — バリデーション失敗
pub fn load_terminal_api() -> Result<TerminalApiConfig, ConfigError> {
    // figment を構築してから型安全なデシリアライズを行い、バリデーションへ渡す
    // ? 演算子が figment::Error → ConfigError の From 変換を自動的に行う
    let cfg = sources::build_figment()?.extract()?;
    validation::validate_terminal(cfg)
}

/// wnav_master_api 起動時に呼び出す設定ロード関数
///
/// YAML ファイル読込 → `secret_ref` 解決 → バリデーション を順に実行する。
///
/// # Errors
///
/// - [`ConfigError::ProfileNotSet`] — `WNAV_PROFILE` 環境変数が未設定
/// - [`ConfigError::FileNotFound`] — YAML ファイルが存在しない
/// - [`ConfigError::SchemaVersionMismatch`] — `schema_version` が 1 以外
/// - [`ConfigError::Extract`] — figment のデシリアライズ失敗
/// - [`ConfigError::SecretRefResolution`] — `secret_ref` の解決失敗
/// - [`ConfigError::InvalidValue`] — バリデーション失敗
pub fn load_master_api() -> Result<MasterApiConfig, ConfigError> {
    // figment を構築してから型安全なデシリアライズを行い、バリデーションへ渡す
    // ? 演算子が figment::Error → ConfigError の From 変換を自動的に行う
    let cfg = sources::build_figment()?.extract()?;
    validation::validate_master(cfg)
}
