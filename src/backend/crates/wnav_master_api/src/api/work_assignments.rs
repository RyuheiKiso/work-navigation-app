// 作業指示 Push 受信ハンドラ（API-sync-003 / MOD-BE-011）
//
// 外部システムからの作業指示 Push を受信する。
// HMAC-SHA256 署名検証（X-Signature-256 ヘッダ）と idempotency_key による重複チェックを実施する。
// SQLX_OFFLINE=true 環境のため sqlx::query() 動的クエリを使用する。

use axum::{
    body::to_bytes,
    extract::{Request, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::Utc;
use sqlx::Row as _;
use uuid::Uuid;

use crate::{
    dto::work_assignments::{WorkAssignmentPushRequest, WorkAssignmentPushResponse},
    error::AppError,
    state::AppState,
};

/// 作業指示 Push 受信（POST /api/v1/work-assignments）。
///
/// 外部システムが作業指示を Push する。
/// X-Signature-256 ヘッダで HMAC-SHA256 署名を検証する。
/// 同一 idempotency_key の再送は 200 を返して無視する（Idempotent 設計）。
#[utoipa::path(
    post,
    path = "/api/v1/work-assignments",
    tag = "work_assignments",
    request_body = WorkAssignmentPushRequest,
    params(
        ("X-Signature-256" = String, Header, description = "HMAC-SHA256 署名 (sha256=<hex>)"),
    ),
    responses(
        (status = 201, description = "新規登録成功", body = WorkAssignmentPushResponse),
        (status = 200, description = "重複受信（idempotency_key で判定）", body = WorkAssignmentPushResponse),
        (status = 422, description = "署名検証失敗"),
    )
)]
pub async fn receive_work_assignment(
    State(state): State<AppState>,
    req: Request,
) -> Result<impl IntoResponse, AppError> {
    // X-Signature-256 ヘッダを取得する
    let signature = req
        .headers()
        .get("X-Signature-256")
        .and_then(|v| v.to_str().ok())
        .ok_or(AppError::InvalidSignature)?
        .to_string();

    // ボディを取得する
    let (_, body) = req.into_parts();
    let body_bytes = to_bytes(body, 10 * 1024 * 1024)
        .await
        .map_err(|_| AppError::Validation("リクエストボディが大きすぎます".to_string()))?;

    // HMAC 秘密鍵を取得して署名を検証する
    let hmac_key = fetch_webhook_hmac_key(&state).await?;
    verify_hmac_signature(&body_bytes, &signature, &hmac_key)?;

    // JSON デシリアライズ
    let payload: WorkAssignmentPushRequest = serde_json::from_slice(&body_bytes)
        .map_err(|e| AppError::Validation(format!("JSON パースエラー: {e}")))?;

    let server_received_at = Utc::now().timestamp_millis();

    // idempotency_key で重複チェック
    let existing = sqlx::query(
        r#"SELECT id FROM work_assignments WHERE idempotency_key = $1"#,
    )
    .bind(payload.idempotency_key)
    .fetch_optional(&state.read_pool)
    .await?;

    if let Some(existing_row) = existing {
        let existing_id: Uuid = existing_row.get("id");
        tracing::info!(
            event = "work_assignment.duplicate",
            idempotency_key = %payload.idempotency_key,
            "重複した作業指示 Push を受信しました",
        );
        return Ok((
            StatusCode::OK,
            Json(WorkAssignmentPushResponse {
                assignment_id: existing_id,
                idempotency_key: payload.idempotency_key,
                is_duplicate: true,
                server_received_at,
            }),
        ));
    }

    // 新規登録
    let new_id = Uuid::now_v7();

    sqlx::query(
        r#"
        INSERT INTO work_assignments
            (id, idempotency_key, external_assignment_id, factory_id, worker_id,
             process_id, scheduled_start_at, scheduled_end_at, payload,
             server_received_at, created_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, NOW())
        "#,
    )
    .bind(new_id)
    .bind(payload.idempotency_key)
    .bind(&payload.external_assignment_id)
    .bind(payload.factory_id)
    .bind(payload.worker_id)
    .bind(&payload.process_id)
    .bind(payload.scheduled_start_at)
    .bind(payload.scheduled_end_at)
    .bind(&payload.payload)
    .bind(server_received_at)
    .execute(&state.write_pool)
    .await?;

    // SSE ディスパッチログに記録する（terminal-api への配信トリガー）
    sqlx::query(
        r#"
        INSERT INTO sse_dispatch_log
            (id, event_type, assignment_id, factory_id, created_at)
        VALUES (gen_random_uuid(), 'work_assignment_push', $1, $2, NOW())
        "#,
    )
    .bind(new_id)
    .bind(payload.factory_id)
    .execute(&state.write_pool)
    .await?;

    tracing::info!(
        event = "work_assignment.received",
        assignment_id = %new_id,
        idempotency_key = %payload.idempotency_key,
        "作業指示 Push を新規登録しました",
    );

    Ok((
        StatusCode::CREATED,
        Json(WorkAssignmentPushResponse {
            assignment_id: new_id,
            idempotency_key: payload.idempotency_key,
            is_duplicate: false,
            server_received_at,
        }),
    ))
}

/// DB の webhook_secrets テーブルから HMAC 秘密鍵を取得する。
async fn fetch_webhook_hmac_key(state: &AppState) -> Result<String, AppError> {
    let row = sqlx::query(
        r#"
        SELECT secret_value FROM webhook_secrets
        WHERE purpose = 'work_assignment_push' AND is_active = true
        ORDER BY created_at DESC
        LIMIT 1
        "#,
    )
    .fetch_optional(&state.read_pool)
    .await?;

    row.map(|r| {
        let v: String = r.get("secret_value");
        v
    })
    .ok_or_else(|| AppError::Internal("HMAC 秘密鍵が設定されていません".to_string()))
}

/// HMAC-SHA256 署名を検証するヘルパー（定数時間比較でタイミング攻撃を防止する）。
fn verify_hmac_signature(
    body: &[u8],
    signature: &str,
    secret: &str,
) -> Result<(), AppError> {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;
    type HmacSha256 = Hmac<Sha256>;

    let hex_sig = signature.strip_prefix("sha256=").unwrap_or(signature);
    let expected_bytes =
        hex::decode(hex_sig).map_err(|_| AppError::InvalidSignature)?;

    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
        .map_err(|_| AppError::Internal("HMAC 初期化エラー".to_string()))?;
    mac.update(body);
    let computed = mac.finalize().into_bytes();

    // subtle::ConstantTimeEq で定数時間比較する
    use subtle::ConstantTimeEq;
    if computed.ct_eq(&expected_bytes).into() {
        Ok(())
    } else {
        Err(AppError::InvalidSignature)
    }
}
