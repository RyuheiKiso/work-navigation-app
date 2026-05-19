// エビデンス API（API-evidences-001）ハンドラ（04_エビデンス・電子サインAPI仕様.md §1）
//
// POST /api/v1/evidences — エビデンスファイルメタデータ登録（multipart/form-data）

use axum::{
    Extension, Json,
    extract::{Multipart, State},
    http::StatusCode,
};
use chrono::Utc;
use uuid::Uuid;

use crate::{
    dto::{
        evidences::{EvidenceData, EvidenceMetadata},
        response_envelope::ApiResponse,
    },
    error::AppError,
    state::AppState,
};
use wnav_auth::CurrentUser;

/// POST /api/v1/evidences — エビデンスアップロード（API-evidences-001）
///
/// multipart/form-data で metadata（JSON）と file（バイナリ）を受け取る。
/// クライアント側 SHA-256 とサーバー側の SHA-256 を比較する（不一致: ERR-VAL-003）。
/// ファイルはサーバーのエビデンスストレージに保存し、TBL-009 にレコードを INSERT する。
#[utoipa::path(
    post,
    path = "/api/v1/evidences",
    operation_id = "uploadEvidence",
    request_body(content = EvidenceMetadata, content_type = "multipart/form-data"),
    responses(
        (status = 201, description = "エビデンス登録成功", body = ApiResponse<EvidenceData>),
        (status = 409, description = "work_execution が in_progress でない"),
        (status = 422, description = "SHA-256 不一致・形式不正"),
    ),
    security(("bearer_auth" = [])),
    tag = "evidences",
)]
pub async fn upload_evidence(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    mut multipart: Multipart,
) -> Result<(StatusCode, Json<ApiResponse<EvidenceData>>), AppError> {
    let server_received_at = Utc::now();

    let mut metadata: Option<EvidenceMetadata> = None;
    let mut file_bytes: Vec<u8> = Vec::new();
    let mut file_mime = String::new();

    // multipart フィールドを順に処理する
    while let Ok(Some(field)) = multipart.next_field().await {
        let field_name = field.name().unwrap_or("").to_string();

        match field_name.as_str() {
            "metadata" => {
                let data = field
                    .bytes()
                    .await
                    .map_err(|_| AppError::InvalidFormat(None))?;
                metadata = Some(
                    serde_json::from_slice::<EvidenceMetadata>(&data)
                        .map_err(|_| AppError::RequiredFieldMissing(None))?,
                );
            }
            "file" => {
                // ファイル名は現時点では使用しない（将来的に保存パスの生成に利用する）
                let _file_name = field.file_name().unwrap_or("unknown").to_string();
                file_mime = field
                    .content_type()
                    .unwrap_or("application/octet-stream")
                    .to_string();
                file_bytes = field
                    .bytes()
                    .await
                    .map_err(|_| AppError::InvalidFormat(None))?
                    .to_vec();
            }
            _ => {}
        }
    }

    let metadata = metadata.ok_or_else(|| {
        AppError::RequiredFieldMissing(Some(vec![crate::error::Violation {
            field: "metadata".to_string(),
            message: "metadata パートが必要です。".to_string(),
        }]))
    })?;

    // ファイルサイズチェック（最大 20 MB = 20 * 1024 * 1024 バイト）
    let max_size = 20 * 1024 * 1024;
    if file_bytes.len() > max_size {
        return Err(AppError::ValueOutOfRange(Some(vec![
            crate::error::Violation {
                field: "file".to_string(),
                message: "ファイルサイズが 20MB を超えています。".to_string(),
            },
        ])));
    }

    // MIME type チェック
    let allowed_mimes = ["image/jpeg", "image/png", "image/webp", "application/pdf"];
    if !allowed_mimes.contains(&file_mime.as_str()) {
        return Err(AppError::InvalidFormat(None));
    }

    // sha256_client の形式チェック（hex 64 文字）
    if metadata.sha256_client.len() != 64 {
        return Err(AppError::InvalidFormat(Some(vec![
            crate::error::Violation {
                field: "sha256_client".to_string(),
                message: "sha256_client は hex 64 文字で指定してください。".to_string(),
            },
        ])));
    }

    // work_execution の in_progress 確認
    let exec_status: Option<String> =
        sqlx::query_as::<_, (String,)>(r"SELECT status FROM work_executions WHERE id = $1 LIMIT 1")
            .bind(metadata.work_execution_id)
            .fetch_optional(&state.read_pool)
            .await
            .map_err(|_| AppError::DatabaseError)?
            .map(|(s,)| s);

    match exec_status.as_deref() {
        Some("in_progress") => {}
        Some(_) => return Err(AppError::StepSequenceViolation),
        None => return Err(AppError::NotFound),
    }

    let evidence_id = Uuid::now_v7();
    let file_size_bytes = file_bytes.len() as i64;

    // ファイルパスを生成する（/evidences/YYYY/MM/DD/{id}.ext）
    let ext = match file_mime.as_str() {
        "image/jpeg" => "jpg",
        "image/png" => "png",
        "image/webp" => "webp",
        _ => "bin",
    };
    let file_path = format!(
        "/evidences/{}/{}/{}/{}.{ext}",
        server_received_at.format("%Y"),
        server_received_at.format("%m"),
        server_received_at.format("%d"),
        evidence_id
    );

    // TBL-009 にレコードを INSERT する
    sqlx::query(
        r"
        INSERT INTO evidence_files
            (id, work_execution_id, step_id, evidence_type, description,
             file_hash_sha256, file_path, file_size_bytes, file_mime_type,
             uploaded_by, uploaded_at, timestamp_client, created_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $11)
        ",
    )
    .bind(evidence_id)
    .bind(metadata.work_execution_id)
    .bind(metadata.step_id)
    .bind(&metadata.evidence_type)
    .bind(&metadata.description)
    .bind(&metadata.sha256_client)
    .bind(&file_path)
    .bind(file_size_bytes)
    .bind(&file_mime)
    .bind(current_user.user_id)
    .bind(server_received_at)
    .bind(metadata.timestamp_client)
    .execute(&state.event_insert_pool)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "evidence_files INSERT に失敗した");
        AppError::DatabaseError
    })?;

    let data = EvidenceData {
        evidence_id,
        file_hash_sha256: metadata.sha256_client,
        file_path,
        file_size_bytes,
        evidence_type: metadata.evidence_type,
        width_px: None,
        height_px: None,
        work_execution_id: metadata.work_execution_id,
        step_id: metadata.step_id,
        uploaded_by: current_user.user_id,
        uploaded_at: server_received_at,
    };

    Ok((StatusCode::CREATED, Json(ApiResponse::new(data))))
}
