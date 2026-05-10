//! ヘルスチェック・レディネスチェック
//!
//! 対応 §: ロードマップ §14.2 §31.1 §31.2
//!
//! - `/healthz`: プロセス生存（依存ネットワークを叩かない、Liveness 用）
//! - `/readyz`:  依存（DB）への接続可否（Kubernetes readiness probe 等で使う）

use axum::{extract::State, http::StatusCode, Json};
use serde::Serialize;
use wna_domain::PasswordHasher;

use crate::app_state::AppState;

#[derive(Serialize)]
pub struct ReadyResponse {
    pub status: &'static str,
    pub db: &'static str,
}

pub async fn healthz() -> &'static str { "ok" }

/// `/readyz`: DB へ `SELECT 1` を投げ、応答できれば 200。
/// 503 を返した場合、ロードバランサ／オーケストレータはトラフィックを止める。
pub async fn readyz<H>(
    State(state): State<AppState<H>>,
) -> Result<Json<ReadyResponse>, (StatusCode, Json<ReadyResponse>)>
where H: PasswordHasher + Send + Sync + Clone + 'static {
    match sqlx::query_scalar::<_, i32>("SELECT 1")
        .fetch_one(state.task_repo.pool())
        .await
    {
        Ok(_) => Ok(Json(ReadyResponse { status: "ok", db: "ok" })),
        Err(_) => Err((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ReadyResponse { status: "degraded", db: "unreachable" }),
        )),
    }
}
