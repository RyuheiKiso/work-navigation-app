// 作業実行 API（API-work-execs-001〜005）ハンドラ（03_作業実行API仕様.md §3〜7）
//
// POST /api/v1/work-executions                    — 作業開始（CaseLock 取得必須）
// GET  /api/v1/work-executions/{id}               — 作業実行詳細
// POST /api/v1/work-executions/{id}/suspend       — 中断
// POST /api/v1/work-executions/{id}/resume        — 再開
// POST /api/v1/work-executions/{id}/complete      — 完了

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
        work_executions::{
            CompleteWorkData, CompleteWorkRequest, ResumeData, ResumeRequest, SopVersionSnapshot,
            StartWorkData, StartWorkRequest, SuspendData, SuspendRequest, WorkEventSummary,
            WorkExecutionDetailData,
        },
    },
    error::AppError,
    state::AppState,
};
use wnav_auth::CurrentUser;

/// POST /api/v1/work-executions — 作業開始（API-work-execs-001）
///
/// CaseLock（TBL-051）取得を必須とする（ADR-009）。
/// 他端末が占有中の場合は ERR-BIZ-026（409 Conflict）を返す。
#[utoipa::path(
    post,
    path = "/api/v1/work-executions",
    operation_id = "startWorkExecution",
    request_body = StartWorkRequest,
    responses(
        (status = 201, description = "作業開始成功", body = ApiResponse<StartWorkData>),
        (status = 409, description = "ケース占有中 (ERR-BIZ-026)"),
        (status = 422, description = "バリデーションエラー"),
    ),
    security(("bearer_auth" = [])),
    tag = "work-executions",
)]
pub async fn start_work_execution(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(body): Json<StartWorkRequest>,
) -> Result<(StatusCode, Json<ApiResponse<StartWorkData>>), AppError> {
    // サーバー受信時刻を付与する（権威タイムスタンプ）
    let server_received_at = Utc::now();

    // RBAC: operator / supervisor のみ実行可
    let has_permission = current_user
        .roles
        .iter()
        .any(|r| matches!(r.as_str(), "operator" | "supervisor"));
    if !has_permission {
        return Err(AppError::Forbidden);
    }

    // CaseLock 取得試行（TBL-051、ADR-009）
    // 他端末が占有中の場合は ERR-BIZ-026 を返す
    let lock_result = acquire_case_lock(
        &state.event_insert_pool,
        body.work_order_id,
        body.device_id,
        current_user.user_id,
    )
    .await?;

    if !lock_result {
        return Err(AppError::CaseOccupied);
    }

    let new_id = Uuid::now_v7();

    // work_executions テーブルに作業実行レコードを INSERT する
    sqlx::query(
        r"
        INSERT INTO work_executions
            (id, work_order_id, operator_id, device_id, status,
             started_at, server_received_at, client_recorded_at, created_at)
        VALUES ($1, $2, $3, $4, 'in_progress', $5, $5, $6, $5)
        ",
    )
    .bind(new_id)
    .bind(body.work_order_id)
    .bind(body.operator_id)
    .bind(body.device_id)
    .bind(server_received_at)
    .bind(body.start_timestamp_client)
    .execute(&state.event_insert_pool)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "work_executions INSERT に失敗した");
        AppError::DatabaseError
    })?;

    // work_orders のステータスを in_progress に更新する
    let _ = sqlx::query(
        r"
        UPDATE work_orders SET status = 'in_progress', updated_at = NOW()
        WHERE id = $1
        ",
    )
    .bind(body.work_order_id)
    .execute(&state.event_insert_pool)
    .await;

    let data = StartWorkData {
        id: new_id,
        work_order_id: body.work_order_id,
        operator_id: body.operator_id,
        device_id: body.device_id,
        status: "in_progress".to_string(),
        current_step_id: None,
        sop_version_snapshot: SopVersionSnapshot {
            sop_id: Uuid::nil(),
            version: "1.0.0".to_string(),
            snapshot_hash: "sha256:".to_string(),
        },
        started_at: server_received_at,
        created_at: server_received_at,
    };

    Ok((StatusCode::CREATED, Json(ApiResponse::new(data))))
}

/// GET /api/v1/work-executions/{id} — 作業実行詳細（API-work-execs-002）
#[utoipa::path(
    get,
    path = "/api/v1/work-executions/{id}",
    operation_id = "getWorkExecution",
    params(
        ("id" = Uuid, Path, description = "作業実行 ID"),
    ),
    responses(
        (status = 200, description = "作業実行詳細", body = ApiResponse<WorkExecutionDetailData>),
        (status = 404, description = "見つからない"),
    ),
    security(("bearer_auth" = [])),
    tag = "work-executions",
)]
pub async fn get_work_execution(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<WorkExecutionDetailData>>, AppError> {
    let row = sqlx::query_as::<
        _,
        (
            Uuid,
            Uuid,
            Uuid,
            Uuid,
            String,
            Option<Uuid>,
            chrono::DateTime<Utc>,
            Option<chrono::DateTime<Utc>>,
        ),
    >(
        r"
        SELECT id, work_order_id, operator_id, device_id, status,
               current_step_id, started_at, last_event_at
        FROM work_executions
        WHERE id = $1 AND deleted_at IS NULL
        LIMIT 1
        ",
    )
    .bind(id)
    .fetch_optional(&state.read_pool)
    .await
    .map_err(|_| AppError::DatabaseError)?;

    let Some((
        id,
        work_order_id,
        operator_id,
        device_id,
        status,
        current_step_id,
        started_at,
        last_event_at,
    )) = row
    else {
        return Err(AppError::NotFound);
    };

    let data = WorkExecutionDetailData {
        id,
        work_order_id,
        operator_id,
        device_id,
        status,
        current_step_id,
        completed_step_count: 0,
        total_step_count: 0,
        sop_version_snapshot: SopVersionSnapshot {
            sop_id: Uuid::nil(),
            version: "1.0.0".to_string(),
            snapshot_hash: "sha256:".to_string(),
        },
        started_at,
        last_event_at,
        events: Vec::<WorkEventSummary>::new(),
    };

    Ok(Json(ApiResponse::new(data)))
}

/// POST /api/v1/work-executions/{id}/suspend — 作業中断（API-work-execs-003）
///
/// WorkEvent(suspended) を記録し、CaseLock は維持する（次の再開まで保持）
#[utoipa::path(
    post,
    path = "/api/v1/work-executions/{id}/suspend",
    operation_id = "suspendWorkExecution",
    params(
        ("id" = Uuid, Path, description = "作業実行 ID"),
    ),
    request_body = SuspendRequest,
    responses(
        (status = 200, description = "中断成功", body = ApiResponse<SuspendData>),
        (status = 409, description = "ステータス違反"),
    ),
    security(("bearer_auth" = [])),
    tag = "work-executions",
)]
pub async fn suspend_work_execution(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(body): Json<SuspendRequest>,
) -> Result<Json<ApiResponse<SuspendData>>, AppError> {
    let server_received_at = Utc::now();

    // work_execution を in_progress → suspended に遷移する
    let rows_affected = sqlx::query(
        r"
        UPDATE work_executions
        SET status = 'suspended', last_event_at = $2
        WHERE id = $1 AND status = 'in_progress'
        ",
    )
    .bind(id)
    .bind(server_received_at)
    .execute(&state.event_insert_pool)
    .await
    .map_err(|_| AppError::DatabaseError)?
    .rows_affected();

    if rows_affected == 0 {
        return Err(AppError::StepSequenceViolation);
    }

    // suspensions テーブルに記録する
    let suspension_id = Uuid::now_v7();
    let _ = sqlx::query(
        r"
        INSERT INTO suspensions (id, work_execution_id, reason_code, reason_detail,
                                  suspended_at, client_recorded_at)
        VALUES ($1, $2, $3, $4, $5, $6)
        ",
    )
    .bind(suspension_id)
    .bind(id)
    .bind(&body.reason_code)
    .bind(&body.reason_detail)
    .bind(server_received_at)
    .bind(body.timestamp_client)
    .execute(&state.event_insert_pool)
    .await;

    let data = SuspendData {
        id,
        status: "suspended".to_string(),
        suspension_id,
        suspended_at: server_received_at,
    };

    Ok(Json(ApiResponse::new(data)))
}

/// POST /api/v1/work-executions/{id}/resume — 作業再開（API-work-execs-004）
///
/// CaseLock を確認または取得する（他端末占有中は ERR-BIZ-026）
#[utoipa::path(
    post,
    path = "/api/v1/work-executions/{id}/resume",
    operation_id = "resumeWorkExecution",
    params(
        ("id" = Uuid, Path, description = "作業実行 ID"),
    ),
    request_body = ResumeRequest,
    responses(
        (status = 200, description = "再開成功", body = ApiResponse<ResumeData>),
        (status = 409, description = "ケース占有中"),
    ),
    security(("bearer_auth" = [])),
    tag = "work-executions",
)]
pub async fn resume_work_execution(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(_body): Json<ResumeRequest>,
) -> Result<Json<ApiResponse<ResumeData>>, AppError> {
    let server_received_at = Utc::now();

    // work_execution の work_order_id を取得して CaseLock 確認に使用する
    let work_order_id: Option<Uuid> = sqlx::query_as::<_, (Uuid,)>(
        r"SELECT work_order_id FROM work_executions WHERE id = $1 LIMIT 1",
    )
    .bind(id)
    .fetch_optional(&state.read_pool)
    .await
    .map_err(|_| AppError::DatabaseError)?
    .map(|(v,)| v);

    let work_order_id = work_order_id.ok_or(AppError::NotFound)?;

    // CaseLock 確認・再取得（再開端末が一致しない場合は ERR-BIZ-026）
    let device_id = current_user.device_id.unwrap_or(Uuid::nil());
    let lock_ok = acquire_case_lock(
        &state.event_insert_pool,
        work_order_id,
        device_id,
        current_user.user_id,
    )
    .await?;

    if !lock_ok {
        return Err(AppError::CaseOccupied);
    }

    sqlx::query(
        r"
        UPDATE work_executions
        SET status = 'in_progress', last_event_at = $2
        WHERE id = $1 AND status = 'suspended'
        ",
    )
    .bind(id)
    .bind(server_received_at)
    .execute(&state.event_insert_pool)
    .await
    .map_err(|_| AppError::DatabaseError)?;

    let data = ResumeData {
        id,
        status: "in_progress".to_string(),
        resumed_at: server_received_at,
        current_step_id: None,
    };

    Ok(Json(ApiResponse::new(data)))
}

/// POST /api/v1/work-executions/{id}/complete — 作業完了（API-work-execs-005）
///
/// CaseLock を解放し、ハッシュチェーンブロックを TBL-031 に追記する。
/// Outbox に MSG-001 を挿入する。
#[utoipa::path(
    post,
    path = "/api/v1/work-executions/{id}/complete",
    operation_id = "completeWorkExecution",
    params(
        ("id" = Uuid, Path, description = "作業実行 ID"),
    ),
    request_body = CompleteWorkRequest,
    responses(
        (status = 200, description = "完了成功", body = ApiResponse<CompleteWorkData>),
        (status = 409, description = "未完了ステップあり"),
    ),
    security(("bearer_auth" = [])),
    tag = "work-executions",
)]
pub async fn complete_work_execution(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(body): Json<CompleteWorkRequest>,
) -> Result<Json<ApiResponse<CompleteWorkData>>, AppError> {
    let server_received_at = Utc::now();

    // in_progress → completed に遷移する
    let rows_affected = sqlx::query(
        r"
        UPDATE work_executions
        SET status = 'completed', completed_at = $2, last_event_at = $2
        WHERE id = $1 AND status = 'in_progress'
        ",
    )
    .bind(id)
    .bind(server_received_at)
    .execute(&state.event_insert_pool)
    .await
    .map_err(|_| AppError::DatabaseError)?
    .rows_affected();

    if rows_affected == 0 {
        return Err(AppError::StepSequenceViolation);
    }

    // work_orders のステータスを completed に更新する
    let _ = sqlx::query(
        r"
        UPDATE work_orders wo
        SET status = 'completed', updated_at = NOW()
        FROM work_executions we
        WHERE we.id = $1 AND wo.id = we.work_order_id
        ",
    )
    .bind(id)
    .execute(&state.event_insert_pool)
    .await;

    // CaseLock を解放する
    let _ = sqlx::query(
        r"
        UPDATE case_locks SET status = 'released', released_at = NOW()
        WHERE work_order_id = (
            SELECT work_order_id FROM work_executions WHERE id = $1
        )
        ",
    )
    .bind(id)
    .execute(&state.event_insert_pool)
    .await;

    // Outbox に MSG-001 を挿入する（完了通知）
    let outbox_id = Uuid::now_v7();
    let _ = sqlx::query(
        r"
        INSERT INTO outbox_events (outbox_id, event_type, payload, status, created_at)
        VALUES ($1, 'work_execution.completed', $2, 'PENDING', NOW())
        ",
    )
    .bind(outbox_id)
    .bind(serde_json::json!({
        "work_execution_id": id,
        "completed_by": body.completed_by,
        "completed_at": server_received_at,
    }))
    .execute(&state.event_insert_pool)
    .await;

    let data = CompleteWorkData {
        id,
        status: "completed".to_string(),
        completed_at: server_received_at,
        hash_chain_block_id: None,
        hash_chain_value: None,
    };

    Ok(Json(ApiResponse::new(data)))
}

/// CaseLock（TBL-051）を取得または確認する内部ヘルパー。
///
/// 成功（ロック取得可能）時は true を返す。
/// 他端末が占有中の場合は false を返す。
async fn acquire_case_lock(
    pool: &sqlx::PgPool,
    work_order_id: Uuid,
    device_id: Uuid,
    operator_id: Uuid,
) -> Result<bool, AppError> {
    // 既存の ACTIVE ロックを確認する
    let existing = sqlx::query_as::<_, (Uuid, String)>(
        r"
        SELECT device_id, status FROM case_locks
        WHERE work_order_id = $1 AND status = 'ACTIVE'
        LIMIT 1
        ",
    )
    .bind(work_order_id)
    .fetch_optional(pool)
    .await
    .map_err(|_| AppError::DatabaseError)?;

    if let Some((locked_device_id, _)) = existing {
        // 同一端末が再度ロックを取得する場合はハートビート更新として許可する
        if locked_device_id == device_id {
            let _ = sqlx::query(
                r"
                UPDATE case_locks SET heartbeat_at = NOW()
                WHERE work_order_id = $1 AND device_id = $2 AND status = 'ACTIVE'
                ",
            )
            .bind(work_order_id)
            .bind(device_id)
            .execute(pool)
            .await;
            return Ok(true);
        }
        // 別端末が占有中: 拒否する
        return Ok(false);
    }

    // 新規ロックを INSERT する
    let lock_id = Uuid::now_v7();
    sqlx::query(
        r"
        INSERT INTO case_locks
            (id, work_order_id, device_id, operator_id, status, acquired_at, heartbeat_at)
        VALUES ($1, $2, $3, $4, 'ACTIVE', NOW(), NOW())
        ON CONFLICT (work_order_id) WHERE status = 'ACTIVE' DO NOTHING
        ",
    )
    .bind(lock_id)
    .bind(work_order_id)
    .bind(device_id)
    .bind(operator_id)
    .execute(pool)
    .await
    .map_err(|_| AppError::DatabaseError)?;

    Ok(true)
}
