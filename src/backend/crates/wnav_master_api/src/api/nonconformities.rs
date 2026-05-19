// 非適合品登録ハンドラ（補足仕様）
//
// 非適合品を管理コンソールから起票する（quality_admin が担当）。
// アンドン API 仕様書 §3 の補足仕様に準拠する。
// CAPA との連携に必要なため TBL-013（nonconformities）にレコードを挿入する。
// SQLX_OFFLINE=true 環境のため sqlx::query() 動的クエリを使用する。

use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::Utc;
use uuid::Uuid;

use crate::{
    dto::nonconformities::{NonconformityResponse, RegisterNonconformityRequest},
    error::AppError,
    state::AppState,
};
use wnav_auth::{ApproverRole, AuthenticatedUser};

/// 非適合品登録（POST /api/v1/nonconformities）。
///
/// quality_admin / system_admin のみ登録可。TBL-013（nonconformities）にレコードを挿入する。
/// CAPA 起票の前提となるエンドポイント。
#[utoipa::path(
    post,
    path = "/api/v1/nonconformities",
    tag = "nonconformities",
    security(("Bearer" = [])),
    request_body = RegisterNonconformityRequest,
    responses(
        (status = 201, description = "非適合品登録成功", body = NonconformityResponse),
        (status = 401, description = "未認証"),
        (status = 403, description = "quality_admin / system_admin 専用"),
    )
)]
pub async fn register_nonconformity(
    _user: AuthenticatedUser<ApproverRole>,
    State(state): State<AppState>,
    Json(req): Json<RegisterNonconformityRequest>,
) -> Result<impl IntoResponse, AppError> {
    let new_id = Uuid::now_v7();
    let now = Utc::now();

    // エビデンス ID 配列を JSON 形式で保存する
    let evidence_ids_json = serde_json::json!(req.evidence_ids.unwrap_or_default());

    sqlx::query(
        r#"
        INSERT INTO nonconformities
            (id, alert_id, work_execution_id, lot_id, nc_type, description,
             discovered_by, discovery_step_id, evidence_ids, status,
             created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, 'open', $10, $10)
        "#,
    )
    .bind(new_id)
    .bind(req.alert_id)
    .bind(req.work_execution_id)
    .bind(req.lot_id)
    .bind(&req.nc_type)
    .bind(&req.description)
    .bind(req.discovered_by)
    .bind(req.discovery_step_id)
    .bind(&evidence_ids_json)
    .bind(now)
    .execute(&state.write_pool)
    .await?;

    tracing::info!(
        event = "nonconformity.registered",
        nonconformity_id = %new_id,
        nc_type = %req.nc_type,
        discovered_by = %req.discovered_by,
        "非適合品を登録しました",
    );

    Ok((
        StatusCode::CREATED,
        Json(NonconformityResponse {
            nonconformity_id: new_id,
            nc_type: req.nc_type,
            status: "open".to_string(),
            alert_id: req.alert_id,
            lot_id: req.lot_id,
            discovered_by: req.discovered_by,
            created_at: now,
        }),
    ))
}
