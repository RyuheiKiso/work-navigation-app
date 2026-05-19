// コネクションプール設定モジュール
// wnav_config クレートが YAML + figment で読み込んだ値を受け取り、
// PgPoolOptions で PostgreSQL コネクションプールを生成する。

use sqlx::{PgPool, postgres::PgPoolOptions};
use std::time::Duration;

/// DB 接続設定。wnav_config クレートが YAML + figment で読み込んだ値を受け取る（ADR-IMPL-001）。
/// DatabaseConfig は src/infra/config/config.base.yml の `database.*` セクションに対応する。
#[derive(Debug, serde::Deserialize, Clone)]
pub struct DbConfig {
    /// 最大コネクション数（config.yml `database.max_connections`、デフォルト 20）
    pub max_connections: u32,
    /// 最小アイドルコネクション数（`database.min_connections`）
    pub min_connections: u32,
    /// コネクション取得タイムアウト秒（`database.acquire_timeout_sec`）
    pub acquire_timeout_secs: u64,
    /// アイドルタイムアウト秒（`database.idle_timeout_sec`）
    pub idle_timeout_secs: u64,
    /// コネクション最大ライフタイム秒（`database.max_lifetime_sec`）
    pub max_lifetime_secs: u64,
}

impl Default for DbConfig {
    fn default() -> Self {
        Self {
            max_connections: 20,
            min_connections: 2,
            acquire_timeout_secs: 10,
            idle_timeout_secs: 600,
            max_lifetime_secs: 3600,
        }
    }
}

/// 指定 URL とコンフィグでコネクションプールを生成して返す。
pub async fn connect(database_url: &str, cfg: &DbConfig) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(cfg.max_connections)
        .min_connections(cfg.min_connections)
        .acquire_timeout(Duration::from_secs(cfg.acquire_timeout_secs))
        .idle_timeout(Duration::from_secs(cfg.idle_timeout_secs))
        .max_lifetime(Duration::from_secs(cfg.max_lifetime_secs))
        .connect(database_url)
        .await
}
