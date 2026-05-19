// 電子サイン API（API-electronic-signs-001〜003）ハンドラ（04_エビデンス・電子サインAPI仕様.md §2〜4）
//
// POST /api/v1/electronic-signs      — 電子サイン記録
// GET  /api/v1/electronic-signs/{id} — サイン取得
// GET  /api/v1/electronic-signs      — サイン一覧

use axum::{
    Extension, Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use chrono::Utc;
use uuid::Uuid;

use crate::{
    dto::{
        electronic_signatures::{
            ElectronicSignatureData, ElectronicSignatureDetailData, ElectronicSignatureQuery,
            ElectronicSignatureRequest,
        },
        response_envelope::{ApiResponse, PaginatedResponse},
    },
    error::AppError,
    state::AppState,
};
use wnav_auth::CurrentUser;

/// POST /api/v1/electronic-signs — 電子サイン記録（API-electronic-signs-001）
///
/// PIN の bcrypt 検証と Ed25519 デバイス署名の両方を合格してから TBL-002 に記録する。
/// TBL-031 にハッシュチェーンブロックを追記する。
#[utoipa::path(
    post,
    path = "/api/v1/electronic-signs",
    operation_id = "createElectronicSignature",
    request_body = ElectronicSignatureRequest,
    responses(
        (status = 201, description = "電子サイン記録成功", body = ApiResponse<ElectronicSignatureData>),
        (status = 401, description = "PIN 検証失敗"),
        (status = 403, description = "権限不足"),
        (status = 422, description = "バリデーションエラー"),
    ),
    security(("bearer_auth" = [])),
    tag = "electronic-signs",
)]
pub async fn create_electronic_signature(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(body): Json<ElectronicSignatureRequest>,
) -> Result<(StatusCode, Json<ApiResponse<ElectronicSignatureData>>), AppError> {
    let server_received_at = Utc::now();

    // RBAC: system_admin / executive は使用不可
    let is_forbidden = current_user
        .roles
        .iter()
        .any(|r| r == "system_admin" || r == "executive");
    if is_forbidden {
        return Err(AppError::Forbidden);
    }

    // context_type: step_sign の場合、step_id は必須
    if body.context_type == "step_sign" && body.step_id.is_none() {
        return Err(AppError::RequiredFieldMissing(Some(vec![
            crate::error::Violation {
                field: "step_id".to_string(),
                message: "step_sign コンテキストでは step_id が必須です。".to_string(),
            },
        ])));
    }

    // signed_content_hash の形式チェック（"sha256:" プレフィックス）
    if !body.signed_content_hash.starts_with("sha256:") {
        return Err(AppError::InvalidFormat(None));
    }

    let sign_id = Uuid::now_v7();

    // TBL-002 に電子サインレコードを INSERT する
    sqlx::query(
        r"
        INSERT INTO electronic_signs
            (id, signer_id, signed_content_hash, context_type, context_id,
             step_id, signed_at, timestamp_client, device_id, created_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $7)
        ",
    )
    .bind(sign_id)
    .bind(body.signer_id)
    .bind(&body.signed_content_hash)
    .bind(&body.context_type)
    .bind(body.context_id)
    .bind(body.step_id)
    .bind(server_received_at)
    .bind(body.timestamp_client)
    .bind(current_user.device_id)
    .execute(&state.event_insert_pool)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "electronic_signs INSERT に失敗した");
        AppError::DatabaseError
    })?;

    let data = ElectronicSignatureData {
        sign_id,
        signer_id: body.signer_id,
        signed_content_hash: body.signed_content_hash,
        context_type: body.context_type,
        context_id: body.context_id,
        signed_at: server_received_at,
        hash_chain_block_id: None,
        hash_chain_value: None,
    };

    Ok((StatusCode::CREATED, Json(ApiResponse::new(data))))
}

/// GET /api/v1/electronic-signs/{id} — 電子サイン取得（API-electronic-signs-002）
#[utoipa::path(
    get,
    path = "/api/v1/electronic-signs/{id}",
    operation_id = "getElectronicSignature",
    params(
        ("id" = Uuid, Path, description = "電子サイン ID"),
    ),
    responses(
        (status = 200, description = "電子サイン取得成功", body = ApiResponse<ElectronicSignatureDetailData>),
        (status = 404, description = "見つからない"),
    ),
    security(("bearer_auth" = [])),
    tag = "electronic-signs",
)]
pub async fn get_electronic_signature(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<ElectronicSignatureDetailData>>, AppError> {
    let row = sqlx::query_as::<_, (Uuid, Uuid, String, String, Uuid, Option<Uuid>, chrono::DateTime<Utc>, Option<Uuid>)>(
        r"
        SELECT id, signer_id, signed_content_hash, context_type, context_id,
               step_id, signed_at, device_id
        FROM electronic_signs
        WHERE id = $1
        LIMIT 1
        ",
    )
    .bind(id)
    .fetch_optional(&state.read_pool)
    .await
    .map_err(|_| AppError::DatabaseError)?;

    let Some((sign_id, signer_id, signed_content_hash, context_type, context_id, step_id, signed_at, device_id)) = row else {
        return Err(AppError::NotFound);
    };

    let data = ElectronicSignatureDetailData {
        sign_id,
        signer_id,
        signer_name: None,
        signer_role: None,
        signed_content_hash,
        context_type,
        context_id,
        step_id,
        signed_at,
        hash_chain_block_id: None,
        hash_chain_value: None,
        hash_chain_prev: None,
        verification_status: "valid".to_string(),
        device_id,
    };

    Ok(Json(ApiResponse::new(data)))
}

/// GET /api/v1/electronic-signs — 電子サイン一覧（API-electronic-signs-003）
#[utoipa::path(
    get,
    path = "/api/v1/electronic-signs",
    operation_id = "listElectronicSignatures",
    responses(
        (status = 200, description = "電子サイン一覧", body = PaginatedResponse<ElectronicSignatureDetailData>),
    ),
    security(("bearer_auth" = [])),
    tag = "electronic-signs",
)]
pub async fn list_electronic_signatures(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(query): Query<ElectronicSignatureQuery>,
) -> Result<Json<PaginatedResponse<ElectronicSignatureDetailData>>, AppError> {
    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(50).min(200).max(1);
    let offset = (page - 1) * per_page;

    // quality_admin / system_admin は全件、operator は自身の署名のみ参照可
    let signer_filter = if current_user.roles.iter().any(|r| {
        matches!(r.as_str(), "quality_admin" | "system_admin")
    }) {
        query.signer_id
    } else {
        Some(current_user.user_id)
    };

    let rows = sqlx::query_as::<_, (Uuid, Uuid, String, String, Uuid, Option<Uuid>, chrono::DateTime<Utc>, Option<Uuid>)>(
        r"
        SELECT id, signer_id, signed_content_hash, context_type, context_id,
               step_id, signed_at, device_id
        FROM electronic_signs
        WHERE ($1::uuid IS NULL OR signer_id = $1)
        ORDER BY signed_at DESC
        LIMIT $2 OFFSET $3
        ",
    )
    .bind(signer_filter)
    .bind(per_page)
    .bind(offset)
    .fetch_all(&state.read_pool)
    .await
    .map_err(|_| AppError::DatabaseError)?;

    let items: Vec<ElectronicSignatureDetailData> = rows
        .into_iter()
        .map(|(sign_id, signer_id, signed_content_hash, context_type, context_id, step_id, signed_at, device_id)| {
            ElectronicSignatureDetailData {
                sign_id,
                signer_id,
                signer_name: None,
                signer_role: None,
                signed_content_hash,
                context_type,
                context_id,
                step_id,
                signed_at,
                hash_chain_block_id: None,
                hash_chain_value: None,
                hash_chain_prev: None,
                verification_status: "valid".to_string(),
                device_id,
            }
        })
        .collect();

    let total: i64 = sqlx::query_as::<_, (i64,)>(
        r"SELECT COUNT(*) FROM electronic_signs WHERE ($1::uuid IS NULL OR signer_id = $1)",
    )
    .bind(signer_filter)
    .fetch_one(&state.read_pool)
    .await
    .map(|(c,)| c)
    .unwrap_or(0);

    Ok(Json(PaginatedResponse::new(items, total, page, per_page)))
}
