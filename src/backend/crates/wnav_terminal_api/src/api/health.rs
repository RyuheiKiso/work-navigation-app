// ヘルスチェック API（API-system-001〜002）ハンドラ（07_運用・監視API仕様.md §9〜10）
//
// /healthz: liveness probe（DB 疎通に依存しない）
// /readyz: readiness probe（DB・Outbox・LDAP の状態を返す）

use axum::{Json, extract::State, http::StatusCode};
use chrono::Utc;

use crate::{
    dto::system::{HealthResponse, ReadyzChecks, ReadyzResponse},
    state::AppState,
};

/// GET /healthz — liveness probe（API-system-001）
///
/// バックエンドプロセスが起動中であれば常に HTTP 200 を返す。
/// DB 接続状態には依存しない。
#[utoipa::path(
    get,
    path = "/healthz",
    operation_id = "healthz",
    responses(
        (status = 200, description = "ヘルスチェック成功", body = HealthResponse),
    ),
    tag = "system",
)]
pub async fn healthz() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        timestamp: Utc::now(),
    })
}

/// GET /api/v1/readyz — readiness probe（API-system-002）
///
/// DB・Outbox Consumer・LDAP の状態を返す。
/// DB が error の場合は HTTP 503 を返す。
#[utoipa::path(
    get,
    path = "/api/v1/readyz",
    operation_id = "readyz",
    responses(
        (status = 200, description = "Ready", body = ReadyzResponse),
        (status = 503, description = "Not ready"),
    ),
    tag = "system",
)]
pub async fn readyz(
    State(state): State<AppState>,
) -> (StatusCode, Json<ReadyzResponse>) {
    // event_insert_pool への疎通確認
    let db_status = sqlx::query("SELECT 1")
        .execute(&state.event_insert_pool)
        .await
        .map(|_| "ok".to_string())
        .unwrap_or_else(|_| "error".to_string());

    let is_ready = db_status == "ok";
    let status = if is_ready {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    let response = ReadyzResponse {
        status: if is_ready {
            "ready".to_string()
        } else {
            "not_ready".to_string()
        },
        checks: ReadyzChecks {
            database: db_status,
            // Outbox Consumer の稼働確認は将来実装する
            outbox_consumer: "ok".to_string(),
            // LDAP 接続確認は将来実装する（現時点では "degraded" とする）
            ldap: "degraded".to_string(),
        },
        timestamp: Utc::now(),
    };

    (status, Json(response))
}
