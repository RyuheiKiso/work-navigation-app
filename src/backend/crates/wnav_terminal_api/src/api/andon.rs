// アンドン API（API-andon-001）ハンドラ（06_アンドン・CAPA・KaizenAPI仕様.md §1）
//
// POST /api/v1/alerts — アンドン発報（terminal-api 担当）

use axum::{Extension, Json, extract::State, http::StatusCode};
use chrono::Utc;
use uuid::Uuid;

use crate::{
    dto::{
        andon::{AndonData, AndonRequest},
        response_envelope::ApiResponse,
    },
    error::AppError,
    state::AppState,
};
use wnav_auth::CurrentUser;

/// POST /api/v1/alerts — アンドン発報（API-andon-001）
///
/// 全ロール（operator を含む）がアラートを起票可能。
/// severity: critical の場合は Outbox に MSG-005 を挿入して全 supervisor/quality_admin に通知する。
#[utoipa::path(
    post,
    path = "/api/v1/alerts",
    operation_id = "createAndonAlert",
    request_body = AndonRequest,
    responses(
        (status = 201, description = "アンドン発報成功", body = ApiResponse<AndonData>),
        (status = 401, description = "認証エラー"),
        (status = 422, description = "バリデーションエラー"),
    ),
    security(("bearer_auth" = [])),
    tag = "andon",
)]
pub async fn create_andon_alert(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Json(body): Json<AndonRequest>,
) -> Result<(StatusCode, Json<ApiResponse<AndonData>>), AppError> {
    let server_received_at = Utc::now();

    // 必須フィールドバリデーション
    if body.title.is_empty() || body.title.len() > 200 {
        return Err(AppError::RequiredFieldMissing(None));
    }
    if body.description.is_empty() || body.description.len() > 2000 {
        return Err(AppError::MaxLengthExceeded(None));
    }

    let alert_id = Uuid::now_v7();

    // TBL-012 にアラートレコードを INSERT する
    sqlx::query(
        r"
        INSERT INTO alerts
            (id, alert_type, severity, status, work_execution_id, step_id,
             raised_by, title, description, raised_at, timestamp_client, created_at)
        VALUES ($1, $2, $3, 'open', $4, $5, $6, $7, $8, $9, $10, $9)
        ",
    )
    .bind(alert_id)
    .bind(&body.alert_type)
    .bind(&body.severity)
    .bind(body.work_execution_id)
    .bind(body.step_id)
    .bind(body.raised_by)
    .bind(&body.title)
    .bind(&body.description)
    .bind(server_received_at)
    .bind(body.timestamp_client)
    .execute(&state.event_insert_pool)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "alerts INSERT に失敗した");
        AppError::DatabaseError
    })?;

    // severity: critical の場合は Outbox に MSG-005 を挿入する
    let notification_sent = if body.severity == "critical" {
        let outbox_id = Uuid::now_v7();
        sqlx::query(
            r"
            INSERT INTO outbox_events (outbox_id, event_type, payload, status, created_at)
            VALUES ($1, 'internal.alert_triggered', $2, 'PENDING', NOW())
            ",
        )
        .bind(outbox_id)
        .bind(serde_json::json!({
            "alert_id": alert_id,
            "severity": &body.severity,
            "title": &body.title,
        }))
        .execute(&state.event_insert_pool)
        .await
        .is_ok()
    } else {
        false
    };

    let data = AndonData {
        alert_id,
        alert_type: body.alert_type,
        severity: body.severity,
        status: "open".to_string(),
        work_execution_id: body.work_execution_id,
        raised_by: body.raised_by,
        title: body.title,
        raised_at: server_received_at,
        notification_sent,
    };

    Ok((StatusCode::CREATED, Json(ApiResponse::new(data))))
}
