//! 環境変数からアプリケーション構成を読み込む
//!
//! 対応 §: ロードマップ §14.2 §11.4 §20.1

// 標準ライブラリ
use std::env;
use std::net::SocketAddr;
// 境界エラー
use anyhow::{Context, Result};

/// アプリケーション構成
#[derive(Debug, Clone)]
pub struct AppConfig {
    /// PostgreSQL 接続 URL（DATABASE_URL）
    pub database_url: String,
    /// バインドアドレス（WNA_LISTEN_ADDR、デフォルト 0.0.0.0:8080）
    pub listen_addr: SocketAddr,
}

/// 構成エラー
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    /// 必須環境変数の欠落
    #[error("必須環境変数 {0} が未設定です")]
    Missing(&'static str),
    /// バインドアドレスのパースエラー
    #[error("WNA_LISTEN_ADDR のパースに失敗しました: {0}")]
    InvalidListenAddr(String),
}

impl AppConfig {
    /// 環境変数から構成を読み込む
    ///
    /// # Errors
    /// 必須環境変数の欠落／パース失敗時にエラーを返す。
    pub fn from_env() -> Result<Self> {
        // DATABASE_URL は必須
        let database_url = env::var("DATABASE_URL")
            .map_err(|_| ConfigError::Missing("DATABASE_URL"))
            .context("DATABASE_URL の取得")?;

        // WNA_LISTEN_ADDR はデフォルトあり
        let listen_addr_raw = env::var("WNA_LISTEN_ADDR")
            .unwrap_or_else(|_| "0.0.0.0:8080".to_string());
        // パース
        let listen_addr: SocketAddr = listen_addr_raw
            .parse()
            .map_err(|_| ConfigError::InvalidListenAddr(listen_addr_raw.clone()))
            .context("WNA_LISTEN_ADDR のパース")?;

        // 完成した構成
        Ok(Self {
            database_url,
            listen_addr,
        })
    }
}
