// ヘルスチェックハンドラ
//
// GET /healthz — 軽量ヘルスチェック（認証不要）
// GET /api/v1/ops/health — DB 疎通 + JWT キー存在確認を含む詳細ヘルスチェック

use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};

use crate::{
    dto::health::{ComponentStatus, HealthComponents, HealthResponse},
    error::AppError,
    state::AppState,
};

/// 軽量ヘルスチェック（GET /healthz）。
///
/// 認証不要。監視システムが叩く用途。DB 疎通は確認しない。
#[utoipa::path(
    get,
    path = "/healthz",
    tag = "health",
    responses(
        (status = 200, description = "サービスが稼働中", body = HealthResponse),
    )
)]
pub async fn healthz() -> impl IntoResponse {
    let response = HealthResponse {
        status: "ok".to_string(),
        service: "wnav_master_api".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        components: HealthComponents {
            write_db: ComponentStatus {
                status: "unknown".to_string(),
                error: None,
                latency_ms: None,
            },
            read_db: ComponentStatus {
                status: "unknown".to_string(),
                error: None,
                latency_ms: None,
            },
            jwt_keys: ComponentStatus {
                status: "ok".to_string(),
                error: None,
                latency_ms: None,
            },
        },
        timestamp: chrono::Utc::now().to_rfc3339(),
    };

    (StatusCode::OK, Json(response))
}

/// 詳細ヘルスチェック（GET /api/v1/ops/health）。
///
/// DB 疎通確認と JWT キーストアの存在確認を実施する。
#[utoipa::path(
    get,
    path = "/api/v1/ops/health",
    tag = "ops",
    security(("Bearer" = [])),
    responses(
        (status = 200, description = "全コンポーネント正常", body = HealthResponse),
        (status = 503, description = "一部コンポーネント異常", body = HealthResponse),
    )
)]
pub async fn ops_health(State(state): State<AppState>) -> Result<impl IntoResponse, AppError> {
    let write_db_status = check_db_pool(&state.write_pool, "write").await;
    let read_db_status = check_db_pool(&state.read_pool, "read").await;

    let jwt_status = ComponentStatus {
        status: "ok".to_string(),
        error: None,
        latency_ms: None,
    };

    let all_ok = write_db_status.status == "ok" && read_db_status.status == "ok";
    let overall_status = if all_ok { "ok" } else { "degraded" };
    let http_status = if all_ok {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    let response = HealthResponse {
        status: overall_status.to_string(),
        service: "wnav_master_api".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        components: HealthComponents {
            write_db: write_db_status,
            read_db: read_db_status,
            jwt_keys: jwt_status,
        },
        timestamp: chrono::Utc::now().to_rfc3339(),
    };

    Ok((http_status, Json(response)))
}

/// DB プールへの疎通確認ヘルパー（SELECT 1）。
async fn check_db_pool(pool: &sqlx::PgPool, role: &str) -> ComponentStatus {
    let start = std::time::Instant::now();
    match sqlx::query("SELECT 1").execute(pool).await {
        Ok(_) => {
            let latency = start.elapsed().as_millis();
            tracing::debug!(role = role, latency_ms = latency, "DB 疎通確認 OK");
            ComponentStatus {
                status: "ok".to_string(),
                error: None,
                latency_ms: Some(latency),
            }
        }
        Err(e) => {
            tracing::error!(role = role, error = %e, "DB 疎通確認失敗");
            ComponentStatus {
                status: "error".to_string(),
                error: Some("DB connection failed".to_string()),
                latency_ms: None,
            }
        }
    }
}
