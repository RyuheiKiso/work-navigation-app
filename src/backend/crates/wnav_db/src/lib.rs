// wnav_db クレート — PostgreSQL Infrastructure 実装（MOD-BE-004）
//
// wnav_domain の Repository Trait を sqlx + PostgreSQL で実装する。
// 依存性逆転の原則（DIP）に従い、Domain 層が Infrastructure 詳細に依存しない設計を保証する。
//
// # SQLX_PREPARE_REQUIRED
// sqlx::query_as を使用しているため、コンパイル前に以下を実行する必要がある:
//   cargo sqlx prepare --database-url $DATABASE_URL
// SQLX_OFFLINE=true の場合は .sqlx/ キャッシュを使用する。
//
// # コネクションプール設計（src/backend/CLAUDE.md DB ロール 3 分離）
// - `app_event_insert`: INSERT 専用（wnav_terminal_api のみ保有）
// - `app_write`: SELECT/INSERT/UPDATE（wnav_master_api のみ保有）
// - `app_read`: SELECT 専用（両バイナリが保有）

// unsafe コードを禁止する（src/CLAUDE.md および src/backend/CLAUDE.md の必須要件）
#![forbid(unsafe_code)]
// 例外: doc コメントのリンク省略は許容
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
// 例外: モジュール名重複は許容
#![allow(clippy::module_name_repetitions)]
// 例外: Into 変換の冗長な From 実装は許容
#![allow(clippy::from_over_into)]
// 例外: must_use 警告は許容
#![allow(clippy::must_use_candidate)]
// 例外: cast_possible_truncation（u32→i32 変換等）は各所で unwrap_or で処理済み
#![allow(clippy::cast_possible_truncation)]

pub mod error;
pub mod pool;
pub mod repository;
pub mod row_types;
pub mod transaction;

use sqlx::PgPool;

pub use error::DbError;
pub use pool::{DbConfig, connect};

/// (DB ロール分離) `wnav_terminal_api` 向けの 2 プール初期化のパラメータ。
pub struct TerminalPoolParams<'a> {
    /// 接続先 PostgreSQL ホスト
    pub host: &'a str,
    /// 接続先 PostgreSQL ポート
    pub port: u16,
    /// 接続先データベース名
    pub db_name: &'a str,
    /// `app_event_insert` ロールのユーザー名
    pub event_insert_user: &'a str,
    /// `app_event_insert` ロールのパスワード
    pub event_insert_password: &'a str,
    /// `app_read` ロールのユーザー名
    pub read_user: &'a str,
    /// `app_read` ロールのパスワード
    pub read_password: &'a str,
}

/// (DB ロール分離) `wnav_terminal_api` 向けの 2 プール初期化。
///
/// - `event_insert_pool`: `app_event_insert` ロール（INSERT 専用）
/// - `read_pool`: `app_read` ロール（SELECT 専用）
///
/// # 引数
/// - `params`: 接続先・クレデンシャルをまとめた `TerminalPoolParams`
/// - `cfg`: コネクションプール設定
///
/// # Errors
/// 接続確立に失敗した場合は `DbError::Connection` を返す。
pub async fn init_terminal_pools(
    params: TerminalPoolParams<'_>,
    cfg: &DbConfig,
) -> Result<(PgPool, PgPool), DbError> {
    // app_event_insert プール: INSERT 専用（Append-only テーブルへの書き込み）
    let event_insert_url = format!(
        "postgres://{}:{}@{}:{}/{}",
        params.event_insert_user,
        params.event_insert_password,
        params.host,
        params.port,
        params.db_name
    );
    let event_insert_pool = connect(&event_insert_url, cfg)
        .await
        .map_err(|e| DbError::Connection(format!("app_event_insert プール接続失敗: {e}")))?;

    // app_read プール: SELECT 専用（全テーブルの読み取り）
    let read_url = format!(
        "postgres://{}:{}@{}:{}/{}",
        params.read_user, params.read_password, params.host, params.port, params.db_name
    );
    let read_pool = connect(&read_url, cfg)
        .await
        .map_err(|e| DbError::Connection(format!("app_read プール接続失敗: {e}")))?;

    Ok((event_insert_pool, read_pool))
}

/// (DB ロール分離) `wnav_master_api` 向けの 2 プール初期化のパラメータ。
pub struct MasterPoolParams<'a> {
    /// 接続先 PostgreSQL ホスト
    pub host: &'a str,
    /// 接続先 PostgreSQL ポート
    pub port: u16,
    /// 接続先データベース名
    pub db_name: &'a str,
    /// `app_write` ロールのユーザー名
    pub write_user: &'a str,
    /// `app_write` ロールのパスワード
    pub write_password: &'a str,
    /// `app_read` ロールのユーザー名
    pub read_user: &'a str,
    /// `app_read` ロールのパスワード
    pub read_password: &'a str,
}

/// (DB ロール分離) `wnav_master_api` 向けの 2 プール初期化。
///
/// - `write_pool`: `app_write` ロール（SELECT/INSERT/UPDATE。マスタ CRUD）
/// - `read_pool`: `app_read` ロール（SELECT 専用）
///
/// # 引数
/// - `params`: 接続先・クレデンシャルをまとめた `MasterPoolParams`
/// - `cfg`: コネクションプール設定
///
/// # Errors
/// 接続確立に失敗した場合は `DbError::Connection` を返す。
pub async fn init_master_pools(
    params: MasterPoolParams<'_>,
    cfg: &DbConfig,
) -> Result<(PgPool, PgPool), DbError> {
    // app_write プール: SELECT/INSERT/UPDATE（マスタデータ CRUD）
    let write_url = format!(
        "postgres://{}:{}@{}:{}/{}",
        params.write_user, params.write_password, params.host, params.port, params.db_name
    );
    let write_pool = connect(&write_url, cfg)
        .await
        .map_err(|e| DbError::Connection(format!("app_write プール接続失敗: {e}")))?;

    // app_read プール: SELECT 専用（Audit Trail 照会・ダッシュボード）
    let read_url = format!(
        "postgres://{}:{}@{}:{}/{}",
        params.read_user, params.read_password, params.host, params.port, params.db_name
    );
    let read_pool = connect(&read_url, cfg)
        .await
        .map_err(|e| DbError::Connection(format!("app_read プール接続失敗: {e}")))?;

    Ok((write_pool, read_pool))
}
