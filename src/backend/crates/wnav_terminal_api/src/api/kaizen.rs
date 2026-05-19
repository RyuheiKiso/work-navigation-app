// Kaizen 改善提案 API（API-kaizen-001）ハンドラ（06_アンドン・CAPA・KaizenAPI仕様.md §6）
//
// POST /api/v1/kaizen-proposals — 改善提案起票（terminal-api 担当）

use axum::{Extension, Json, extract::State, http::StatusCode};
use chrono::Utc;
use uuid::Uuid;

use crate::{
    dto::{
        kaizen::{KaizenData, KaizenRequest},
        response_envelope::ApiResponse,
    },
    error::AppError,
    state::AppState,
};
use wnav_auth::CurrentUser;

/// POST /api/v1/kaizen-proposals — 改善提案起票（API-kaizen-001）
///
/// 全ロール（operator を含む）が提案可能。
/// 最大 10 件のエビデンス添付を許容する。
#[utoipa::path(
    post,
    path = "/api/v1/kaizen-proposals",
    operation_id = "createKaizenProposal",
    request_body = KaizenRequest,
    responses(
        (status = 201, description = "改善提案起票成功", body = ApiResponse<KaizenData>),
        (status = 401, description = "認証エラー"),
        (status = 422, description = "バリデーションエラー"),
    ),
    security(("bearer_auth" = [])),
    tag = "kaizen",
)]
pub async fn create_kaizen_proposal(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Json(body): Json<KaizenRequest>,
) -> Result<(StatusCode, Json<ApiResponse<KaizenData>>), AppError> {
    let server_received_at = Utc::now();

    // 必須フィールドバリデーション
    if body.title.is_empty() || body.title.len() > 200 {
        return Err(AppError::RequiredFieldMissing(None));
    }
    if body.proposal_detail.len() > 5000 {
        return Err(AppError::MaxLengthExceeded(None));
    }

    // evidence_ids は最大 10 件
    if let Some(ref ids) = body.evidence_ids {
        if ids.len() > 10 {
            return Err(AppError::ValueOutOfRange(Some(vec![
                crate::error::Violation {
                    field: "evidence_ids".to_string(),
                    message: "evidence_ids は最大 10 件です。".to_string(),
                },
            ])));
        }
    }

    let proposal_id = Uuid::now_v7();

    // TBL-015 にレコードを INSERT する
    sqlx::query(
        r"
        INSERT INTO kaizen_proposals
            (id, proposer_id, process_id, category, title, current_situation,
             proposal_detail, expected_benefit, related_sop_id, status,
             created_at, timestamp_client)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, 'submitted', $10, $11)
        ",
    )
    .bind(proposal_id)
    .bind(body.proposer_id)
    .bind(body.process_id)
    .bind(&body.category)
    .bind(&body.title)
    .bind(&body.current_situation)
    .bind(&body.proposal_detail)
    .bind(&body.expected_benefit)
    .bind(body.related_sop_id)
    .bind(server_received_at)
    .bind(body.timestamp_client)
    .execute(&state.event_insert_pool)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "kaizen_proposals INSERT に失敗した");
        AppError::DatabaseError
    })?;

    let data = KaizenData {
        proposal_id,
        status: "submitted".to_string(),
        title: body.title,
        proposer_id: body.proposer_id,
        created_at: server_received_at,
    };

    Ok((StatusCode::CREATED, Json(ApiResponse::new(data))))
}
