//! work-navigation-app infrastructure 層
//!
//! 対応 §: ロードマップ §7.3 §14.2 §16
//!
//! `tokio::main` エントリ／DI／環境変数読込を提供する。

// 子モジュール
pub mod config;
// CLI サブコマンド実装（serve / seed）
pub mod commands;

// 代表型を再エクスポート
pub use config::{AppConfig, ConfigError};
pub use commands::seed::{run as run_seed, SeedPreset};
