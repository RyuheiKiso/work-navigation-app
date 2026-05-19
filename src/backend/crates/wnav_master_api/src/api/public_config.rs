// 公開設定ハンドラ（ADR-IMPL-002）
//
// GET /api/v1/public/config — 認証不要の SPA 起動時設定取得 API。

use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};

use crate::{dto::public_config::PublicConfigResponse, error::AppError, state::AppState};

/// マスタ SPA 起動時設定取得（GET /api/v1/public/config）。
///
/// 認証不要。SPA が起動直後に呼び出してサーバー側設定を取得する。
/// 非機密情報のみ返す（DB URL・秘密鍵等は絶対に含めない）。
#[utoipa::path(
    get,
    path = "/api/v1/public/config",
    tag = "public",
    responses(
        (status = 200, description = "公開設定", body = PublicConfigResponse),
    )
)]
pub async fn get_public_config(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let fe = &state.config.frontend_master;

    // フロントエンド向け公開設定を返す（非機密情報のみ）
    Ok((
        StatusCode::OK,
        Json(PublicConfigResponse {
            api_base_url: fe.api_base_url.clone(),
            openapi_url: fe.openapi_url.clone(),
            session_timeout_min: fe.session_timeout_min,
            polling_interval_ms: fe.polling_interval_ms,
        }),
    ))
}
