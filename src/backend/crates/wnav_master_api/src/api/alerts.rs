// アンドン対応ハンドラ（API-andon-002）
//
// 管理コンソールからのアラート対応・解決を担当する。
// アンドン発報（POST /alerts）は terminal-api が担当し、
// アンドン対応（PATCH /alerts/{id}/acknowledge）は master-api が担当する。
// SQLX_OFFLINE=true 環境のため sqlx::query() 動的クエリを使用する。

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::Utc;
use uuid::Uuid;

use crate::{
    dto::alerts::{AcknowledgeAlertRequest, AlertAcknowledgedResponse},
    error::AppError,
    state::AppState,
};
use wnav_auth::{AuditorRole, AuthenticatedUser};

/// アンドン対応・解決（PATCH /api/v1/alerts/{id}/acknowledge）。
///
/// 管理コンソールからアラートを確認・解決する。AuditorRole 以上必須。
/// アンドン発報（POST /alerts）は terminal-api が担当する。
#[utoipa::path(
    patch,
    path = "/api/v1/alerts/{id}/acknowledge",
    tag = "alerts",
    security(("Bearer" = [])),
    params(
        ("id" = Uuid, Path, description = "アラート ID"),
    ),
    request_body = AcknowledgeAlertRequest,
    responses(
        (status = 200, description = "アラート対応成功", body = AlertAcknowledgedResponse),
        (status = 401, description = "未認証"),
        (status = 403, description = "AuditorRole 必須"),
        (status = 404, description = "アラートが見つからない"),
    )
)]
pub async fn acknowledge_alert(
    _user: AuthenticatedUser<AuditorRole>,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<AcknowledgeAlertRequest>,
) -> Result<impl IntoResponse, AppError> {
    let now = Utc::now();

    // resolved フラグに応じて status を変更する
    let new_status = if req.resolved { "resolved" } else { "acknowledged" };

    // アラートが存在することを確認してから更新する
    let affected = sqlx::query(
        r#"
        UPDATE alerts
        SET status = $1,
            acknowledged_by = $2,
            acknowledgement_note = $3,
            acknowledged_at = $4,
            updated_at = $4
        WHERE id = $5 AND deleted_at IS NULL
        "#,
    )
    .bind(new_status)
    .bind(req.acknowledged_by)
    .bind(req.acknowledgement_note.as_deref())
    .bind(now)
    .bind(id)
    .execute(&state.write_pool)
    .await?;

    if affected.rows_affected() == 0 {
        return Err(AppError::NotFound(format!("alert:{id}")));
    }

    tracing::info!(
        event = "alert.acknowledged",
        alert_id = %id,
        acknowledged_by = %req.acknowledged_by,
        resolved = req.resolved,
        "アラートを対応済みにしました",
    );

    Ok((
        StatusCode::OK,
        Json(AlertAcknowledgedResponse {
            alert_id: id,
            status: new_status.to_string(),
            acknowledged_by: req.acknowledged_by,
            acknowledged_at: now,
        }),
    ))
}
