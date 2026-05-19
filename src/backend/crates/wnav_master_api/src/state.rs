// wnav_master_api の依存注入コンテナ（AppState）
//
// `AppState` は write_pool + read_pool の 2 プールのみを保持する。
// event_insert_pool は型として存在しない。
// これにより DB ロール混入をコンパイル時に防止する（src/backend/CLAUDE.md §3）。

use std::sync::Arc;

use sqlx::PgPool;
use wnav_auth::JwtKeyStore;
use wnav_config::MasterApiConfig;

/// マスタメンテ・管理コンソール向け API の依存コンテナ。
///
/// axum::Router に `.with_state(state)` で渡す。
/// event_insert_pool は持たない（コンパイル時にイベント挿入の混入を防止）。
///
/// Clone は Arc でラップされているため低コスト。
#[derive(Clone)]
pub struct AppState {
    /// マスタ書き込みプール（DBロール: app_write）
    /// SOP・マスタ・ユーザー等への INSERT / UPDATE
    pub write_pool: PgPool,
    /// 読み取り専用プール（DBロール: app_read）
    /// Audit Trail 照会・ダッシュボード等の SELECT
    pub read_pool: PgPool,
    /// JWT 検証・発行キーストア（aud = "master-api" 専用）
    pub key_store: Arc<JwtKeyStore>,
    /// アプリケーション設定
    pub config: Arc<MasterApiConfig>,
}
