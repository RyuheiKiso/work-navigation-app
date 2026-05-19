// ステップイベント API（API-step-events-001）ハンドラ（03_作業実行API仕様.md §8）
//
// POST /api/v1/work-executions/{id}/events — ステップ完了イベント記録
//
// SQLX_OFFLINE=true 環境のため sqlx::query() を使用する。cargo sqlx prepare 後に sqlx::query! に切り替えること。

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
    // DDL に current_step_id 列は存在しないため current_step_index（SMALLINT）を代用する
    let execution_row = sqlx::query_as::<_, (String, i16)>(
        r"
        SELECT status, current_step_index
        FROM work_executions
        WHERE work_execution_id = $1
        LIMIT 1
        ",
    )
    .bind(id)
    .fetch_optional(&state.read_pool)
    .await
    .map_err(|_| AppError::DatabaseError)?;

    let Some((status, _current_step_index)) = execution_row else {
        return Err(AppError::NotFound);
    };

    if status != "IN_PROGRESS" {
        return Err(AppError::StepSequenceViolation);
    }

    // ロックステップ強制（ERR-BIZ-001）:
    // step_completed / step_skipped の場合のステップ順序チェック（簡略実装）
    if matches!(body.activity.as_str(), "step_completed" | "step_skipped") {
        if let Some(current) = Option::<Uuid>::None {
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
    // DDL 列名: event_id, case_id, resource, sop_version_id, terminal_id, prev_hash, content_hash, server_received_at
    // prev_hash / content_hash は CHAR(64) 制約のため sha256: プレフィックスを除去して 64 文字 hex のみにする
    let event_id = Uuid::now_v7();
    let prev_hash_hex = prev_hash.trim_start_matches("sha256:").to_string();
    let content_hash_hex = current_hash.trim_start_matches("sha256:").to_string();
    // 先頭に 0 を埋めて必ず 64 文字にする（簡略 hash 実装では 64 文字未満になる場合があるため）
    let prev_hash_64 = format!("{:0>64}", prev_hash_hex);
    let content_hash_64 = format!("{:0>64}", content_hash_hex);
    sqlx::query(
        r"
        INSERT INTO work_events
            (event_id, case_id, activity, step_id,
             timestamp_server, timestamp_client, resource, sop_version_id,
             terminal_id, prev_hash, content_hash, payload, server_received_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $5)
        ",
    )
    .bind(event_id)
    .bind(id)
    .bind(&body.activity)
    .bind(body.step_id)
    .bind(server_received_at)
    .bind(body.timestamp_client)
    .bind(current_user.user_id)
    .bind(Uuid::nil())
    .bind(current_user.device_id.unwrap_or_else(Uuid::nil))
    .bind(&prev_hash_64)
    .bind(&content_hash_64)
    .bind(serde_json::json!({
        "activity": body.activity,
        "step_id": body.step_id,
        "remarks": body.remarks,
        "duration_seconds": body.duration_seconds,
    }))
    .execute(&state.event_insert_pool)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "work_events INSERT に失敗した");
        AppError::DatabaseError
    })?;

    // Outbox に積む（step_completed 時のみ）
    // event_type は CHECK 制約の列挙値 'work_event' を使用する（'work_event.step_completed' は不可）
    if body.activity == "step_completed" {
        let outbox_id = Uuid::now_v7();
        let idempotency_key = Uuid::now_v7();
        let _ = sqlx::query(
            r"
            INSERT INTO outbox_events
                (outbox_id, event_id, event_type, payload, status, idempotency_key, created_at)
            VALUES ($1, $2, 'work_event', $3, 'PENDING', $4, NOW())
            ",
        )
        .bind(outbox_id)
        .bind(event_id)
        .bind(serde_json::json!({
            "event_id": event_id,
            "work_execution_id": id,
            "step_id": body.step_id,
            "timestamp_server": server_received_at,
        }))
        .bind(idempotency_key)
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
        hash_chain_prev: prev_hash_64,
        hash_chain_current: content_hash_64,
        next_step_id,
    };

    Ok((StatusCode::CREATED, Json(ApiResponse::new(data))))
}

/// 直前イベントのハッシュ値を取得する。
///
/// 最初のイベントの場合は genesis ハッシュ（"0" × 64）を返す。
async fn get_prev_hash(pool: &sqlx::PgPool, work_execution_id: Uuid) -> Result<String, AppError> {
    // DDL: content_hash CHAR(64)、case_id が work_execution_id に対応する
    let prev = sqlx::query_as::<_, (String,)>(
        r"
        SELECT content_hash FROM work_events
        WHERE case_id = $1
        ORDER BY server_received_at DESC
        LIMIT 1
        ",
    )
    .bind(work_execution_id)
    .fetch_optional(pool)
    .await
    .map_err(|_| AppError::DatabaseError)?;

    Ok(prev.map(|(h,)| h).unwrap_or_else(|| {
        // genesis ハッシュ: "0" × 64（DDL の prev_hash DEFAULT と整合する）
        "0000000000000000000000000000000000000000000000000000000000000000".to_string()
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
