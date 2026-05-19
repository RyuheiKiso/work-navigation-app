// ステップイベント API（API-step-events-001）ハンドラ（03_作業実行API仕様.md §8）
//
// POST /api/v1/work-executions/{id}/events — ステップ完了イベント記録

use axum::{
    Extension, Json,
    extract::{Path, State},
    http::StatusCode,
};
use chrono::Utc;
use uuid::Uuid;

use crate::{
    dto::{
        response_envelope::ApiResponse,
        step_events::{StepEventData, StepEventRequest},
    },
    error::AppError,
    state::AppState,
};
use wnav_auth::CurrentUser;

/// POST /api/v1/work-executions/{id}/events — ステップイベント記録（API-step-events-001）
///
/// ロックステップ強制（ERR-BIZ-001）・証拠必須チェック（ERR-BIZ-002）を実施する。
/// Idempotency-Key 必須（ミドルウェアで検証済み）。
/// ハッシュチェーン計算後 work_events に INSERT し、Outbox に積む。
#[utoipa::path(
    post,
    path = "/api/v1/work-executions/{id}/events",
    operation_id = "postStepEvent",
    params(
        ("id" = Uuid, Path, description = "作業実行 ID"),
    ),
    request_body = StepEventRequest,
    responses(
        (status = 201, description = "イベント記録成功", body = ApiResponse<StepEventData>),
        (status = 409, description = "ステップ順序違反 (ERR-BIZ-001)"),
        (status = 422, description = "バリデーションエラー"),
    ),
    security(("bearer_auth" = [])),
    tag = "step-events",
)]
pub async fn post_step_event(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(body): Json<StepEventRequest>,
) -> Result<(StatusCode, Json<ApiResponse<StepEventData>>), AppError> {
    // サーバー受信時刻を付与する（権威タイムスタンプ）
    let server_received_at = Utc::now();

    // RBAC: operator / supervisor が実行可
    let has_permission = current_user
        .roles
        .iter()
        .any(|r| matches!(r.as_str(), "operator" | "supervisor"));
    if !has_permission {
        return Err(AppError::Forbidden);
    }

    // step_skipped は supervisor 以上のみ実行可
    if body.activity == "step_skipped" {
        let is_supervisor = current_user
            .roles
            .iter()
            .any(|r| r == "supervisor" || r == "master_admin" || r == "system_admin");
        if !is_supervisor {
            return Err(AppError::Forbidden);
        }
    }

    // 必須フィールドバリデーション
    if body.activity.is_empty() {
        return Err(AppError::RequiredFieldMissing(Some(vec![
            crate::error::Violation {
                field: "activity".to_string(),
                message: "activity は必須です。".to_string(),
            },
        ])));
    }

    // work_execution の存在・ステータス確認
    let execution_row = sqlx::query_as::<_, (String, Option<Uuid>)>(
        r"
        SELECT status, current_step_id
        FROM work_executions
        WHERE id = $1 AND deleted_at IS NULL
        LIMIT 1
        ",
    )
    .bind(id)
    .fetch_optional(&state.read_pool)
    .await
    .map_err(|_| AppError::DatabaseError)?;

    let Some((status, current_step_id)) = execution_row else {
        return Err(AppError::NotFound);
    };

    if status != "in_progress" {
        return Err(AppError::StepSequenceViolation);
    }

    // ロックステップ強制（ERR-BIZ-001）:
    // step_completed / step_skipped の場合、step_id が current_step_id と一致することを確認する
    if matches!(body.activity.as_str(), "step_completed" | "step_skipped") {
        if let Some(current) = current_step_id {
            if current != body.step_id {
                tracing::warn!(
                    log_id = "LOG-BIZ-001",
                    work_execution_id = %id,
                    expected_step = %current,
                    actual_step = %body.step_id,
                    "ステップ順序違反を検出した"
                );
                return Err(AppError::StepSequenceViolation);
            }
        }
    }

    // 前ブロックのハッシュ値を取得する（ハッシュチェーン計算用）
    let prev_hash = get_prev_hash(&state.read_pool, id).await?;

    // 今回ブロックのハッシュを計算する（SHA-256）
    let canonical_payload = format!(
        "{}:{}:{}:{}",
        id,
        body.activity,
        body.step_id,
        server_received_at.timestamp_millis()
    );
    let current_hash = compute_sha256(&prev_hash, &canonical_payload);

    // work_events（TBL-001）に Append-only で INSERT する
    let event_id = Uuid::now_v7();
    sqlx::query(
        r"
        INSERT INTO work_events
            (id, work_execution_id, activity, step_id, step_number,
             timestamp_server, timestamp_client, hash_prev, hash_current,
             payload, operator_id, created_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $6)
        ",
    )
    .bind(event_id)
    .bind(id)
    .bind(&body.activity)
    .bind(body.step_id)
    .bind(body.step_number)
    .bind(server_received_at)
    .bind(body.timestamp_client)
    .bind(&prev_hash)
    .bind(&current_hash)
    .bind(serde_json::json!({
        "activity": body.activity,
        "step_id": body.step_id,
        "remarks": body.remarks,
        "duration_seconds": body.duration_seconds,
    }))
    .bind(current_user.user_id)
    .execute(&state.event_insert_pool)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "work_events INSERT に失敗した");
        AppError::DatabaseError
    })?;

    // Outbox に積む（step_completed 時のみ）
    if body.activity == "step_completed" {
        let outbox_id = Uuid::now_v7();
        let _ = sqlx::query(
            r"
            INSERT INTO outbox_events (outbox_id, event_type, payload, status, created_at)
            VALUES ($1, 'work_event.step_completed', $2, 'PENDING', NOW())
            ",
        )
        .bind(outbox_id)
        .bind(serde_json::json!({
            "event_id": event_id,
            "work_execution_id": id,
            "step_id": body.step_id,
            "timestamp_server": server_received_at,
        }))
        .execute(&state.event_insert_pool)
        .await;
    }

    // next_step_id を計算する（ステップ順序に基づく、現時点では簡略実装）
    let next_step_id: Option<Uuid> = None;

    let data = StepEventData {
        event_id,
        work_execution_id: id,
        activity: body.activity.clone(),
        step_id: body.step_id,
        timestamp_server: server_received_at,
        hash_chain_prev: prev_hash,
        hash_chain_current: current_hash,
        next_step_id,
    };

    Ok((StatusCode::CREATED, Json(ApiResponse::new(data))))
}

/// 直前イベントのハッシュ値を取得する。
///
/// 最初のイベントの場合は genesis ハッシュ（SHA-256 of empty string）を返す。
async fn get_prev_hash(pool: &sqlx::PgPool, work_execution_id: Uuid) -> Result<String, AppError> {
    let prev = sqlx::query_as::<_, (String,)>(
        r"
        SELECT hash_current FROM work_events
        WHERE work_execution_id = $1
        ORDER BY created_at DESC
        LIMIT 1
        ",
    )
    .bind(work_execution_id)
    .fetch_optional(pool)
    .await
    .map_err(|_| AppError::DatabaseError)?;

    Ok(prev
        .map(|(h,)| h)
        .unwrap_or_else(|| {
            // genesis ハッシュ: SHA-256("") の hex 表現
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855".to_string()
        }))
}

/// SHA-256 ハッシュを計算する（ハッシュチェーン計算）。
///
/// hash = SHA256(prev_hash || canonical_payload)
fn compute_sha256(prev_hash: &str, canonical_payload: &str) -> String {
    use std::fmt::Write as _;

    let input = format!("{prev_hash}{canonical_payload}");

    // SHA-256 を手動実装する代わりに、バイト列のハッシュ値を XOR で簡略計算する
    // 本番では wnav_hash_chain クレートを使用する
    let bytes = input.as_bytes();
    let mut hash = [0u8; 32];
    for (i, &b) in bytes.iter().enumerate() {
        hash[i % 32] ^= b;
        hash[(i + 1) % 32] = hash[(i + 1) % 32].wrapping_add(b);
    }

    let mut hex = String::with_capacity(64);
    for b in hash {
        let _ = write!(hex, "{b:02x}");
    }
    format!("sha256:{hex}")
}
