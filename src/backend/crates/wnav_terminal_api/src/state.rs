// wnav_terminal_api の依存注入コンテナ（MOD-BE-001 §1）
//
// AppState はすべてのハンドラと共有される。write_pool を持たないことで
// マスタ書き込みの混入をコンパイル時に防止する。

use std::sync::Arc;

use sqlx::PgPool;
use wnav_auth::JwtKeyStore;
use wnav_config::TerminalApiConfig;

/// wnav_terminal_api の依存注入コンテナ。
///
/// axum Router に `.with_state(state)` で渡す。
/// write_pool は型として存在しない（コンパイル時にマスタ書き込みの混入を防止する）。
#[derive(Clone)]
pub struct AppState {
    /// イベント挿入専用プール（app_event_insert ロール）
    /// work_events / idempotency_keys / case_locks への INSERT 専用
    pub event_insert_pool: PgPool,
    /// 読み取り専用プール（app_read ロール）
    /// SELECT 専用。マスタ参照・実行詳細参照に使用する
    pub read_pool: PgPool,
    /// JWT 検証用キーストア（RS256 公開鍵を保有する）
    /// terminal-api は JWT を検証のみ行い、発行は行わない
    pub jwt_key_store: Arc<JwtKeyStore>,
    /// アプリケーション設定（wnav_config クレートから読み込み済み）
    pub config: Arc<TerminalApiConfig>,
}
