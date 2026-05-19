// 運用・監視ハンドラ（API-ops-001〜002）
//
// DLQ 照会・再キュー・ハッシュチェーン手動検証・マスタ同期トリガー。
// SQLX_OFFLINE=true 環境のため sqlx::query() 動的クエリを使用する。

use axum::{
    extract::{Path, State},
    http::{HeaderValue, StatusCode, header::CONTENT_TYPE},
    response::IntoResponse,
    Json,
};
use chrono::Utc;
use sqlx::Row as _;
use uuid::Uuid;

use crate::{
    dto::{
        metrics::MetricsResponse,
        ops::{
            DlqEntry, DlqListResponse, HashChainVerifyRequest, HashChainVerifyResponse,
            MasterSyncResponse, RequeueRequest, RequeueResponse,
        },
    },
    error::AppError,
    state::AppState,
};
use wnav_auth::{AdminRole, AuthenticatedUser, AuditorRole};

/// DLQ 一覧取得（GET /api/v1/ops/outbox/dlq）。
///
/// AuditorRole 以上が必要。Dead Letter Queue のエントリを返す。
#[utoipa::path(
    get,
    path = "/api/v1/ops/outbox/dlq",
    tag = "ops",
    security(("Bearer" = [])),
    responses(
        (status = 200, description = "DLQ 一覧", body = DlqListResponse),
        (status = 401, description = "未認証"),
        (status = 403, description = "権限不足"),
    )
)]
pub async fn list_dlq(
    _user: AuthenticatedUser<AuditorRole>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let rows = sqlx::query(
        r#"
        SELECT id, event_id, event_type, last_error, retry_count, dead_lettered_at
        FROM outbox_dead_letters
        WHERE deleted_at IS NULL
        ORDER BY dead_lettered_at DESC
        LIMIT 100
        "#,
    )
    .fetch_all(&state.read_pool)
    .await?;

    let items: Vec<DlqEntry> = rows
        .iter()
        .map(|r| DlqEntry {
            id: r.get("id"),
            event_id: r.get("event_id"),
            event_type: r.get("event_type"),
            last_error: r.get("last_error"),
            retry_count: r.get("retry_count"),
            dead_lettered_at: r.get("dead_lettered_at"),
        })
        .collect();

    let total = items.len() as i64;

    Ok((StatusCode::OK, Json(DlqListResponse { items, total })))
}

/// DLQ エントリの再キュー（POST /api/v1/ops/outbox/{id}/requeue）。
///
/// AdminRole 必須。
#[utoipa::path(
    post,
    path = "/api/v1/ops/outbox/{id}/requeue",
    tag = "ops",
    security(("Bearer" = [])),
    params(
        ("id" = Uuid, Path, description = "DLQ エントリ ID"),
    ),
    request_body = RequeueRequest,
    responses(
        (status = 200, description = "再キュー成功", body = RequeueResponse),
        (status = 401, description = "未認証"),
        (status = 403, description = "AdminRole 必須"),
        (status = 404, description = "エントリが見つからない"),
    )
)]
pub async fn requeue(
    _user: AuthenticatedUser<AdminRole>,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<RequeueRequest>,
) -> Result<impl IntoResponse, AppError> {
    let row = sqlx::query(
        r#"
        SELECT id, event_id FROM outbox_dead_letters
        WHERE id = $1 AND deleted_at IS NULL
        "#,
    )
    .bind(id)
    .fetch_optional(&state.read_pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("dlq_entry:{id}")))?;

    let event_id: Uuid = row.get("event_id");

    sqlx::query(
        r#"
        INSERT INTO outbox_events (id, event_id, event_type, status, retry_count, created_at)
        SELECT gen_random_uuid(), event_id, event_type, 'pending', 0, NOW()
        FROM outbox_dead_letters
        WHERE id = $1
        "#,
    )
    .bind(id)
    .execute(&state.write_pool)
    .await?;

    sqlx::query(
        r#"
        UPDATE outbox_dead_letters SET deleted_at = NOW(), requeue_reason = $1
        WHERE id = $2
        "#,
    )
    .bind(&req.reason)
    .bind(id)
    .execute(&state.write_pool)
    .await?;

    tracing::info!(
        event = "ops.dlq.requeued",
        dlq_entry_id = %id,
        event_id = %event_id,
        "DLQ エントリを再キューしました",
    );

    Ok((
        StatusCode::OK,
        Json(RequeueResponse {
            event_id,
            message: "Event has been requeued successfully.".to_string(),
        }),
    ))
}

/// ハッシュチェーン手動検証（POST /api/v1/ops/hash-chain/verify）。
///
/// AdminRole 必須。
#[utoipa::path(
    post,
    path = "/api/v1/ops/hash-chain/verify",
    tag = "ops",
    security(("Bearer" = [])),
    request_body = HashChainVerifyRequest,
    responses(
        (status = 200, description = "検証結果", body = HashChainVerifyResponse),
        (status = 401, description = "未認証"),
        (status = 403, description = "AdminRole 必須"),
    )
)]
pub async fn verify_hash_chain(
    _user: AuthenticatedUser<AdminRole>,
    State(state): State<AppState>,
    Json(req): Json<HashChainVerifyRequest>,
) -> Result<impl IntoResponse, AppError> {
    use wnav_hash_chain::{ChainBlock, verify_chain};

    let limit = req.limit.unwrap_or(10000);

    let rows = if let Some(case_id) = req.case_id {
        sqlx::query(
            r#"
            SELECT id, case_id, sequence_number, prev_block_hash, content_hash, block_hash, created_at
            FROM work_event_blocks
            WHERE case_id = $1
            ORDER BY sequence_number ASC
            LIMIT $2
            "#,
        )
        .bind(case_id)
        .bind(limit)
        .fetch_all(&state.read_pool)
        .await?
    } else {
        sqlx::query(
            r#"
            SELECT id, case_id, sequence_number, prev_block_hash, content_hash, block_hash, created_at
            FROM work_event_blocks
            ORDER BY case_id, sequence_number ASC
            LIMIT $1
            "#,
        )
        .bind(limit)
        .fetch_all(&state.read_pool)
        .await?
    };

    let mut verified_count: i64 = 0;
    let mut broken_count: i64 = 0;
    let mut broken_case_ids: Vec<Uuid> = Vec::new();

    let mut groups: std::collections::HashMap<Uuid, Vec<ChainBlock>> =
        std::collections::HashMap::new();

    for row in &rows {
        verified_count += 1;
        let block_id: Uuid = row.get("id");
        let case_id: Uuid = row.get("case_id");
        let sequence_number: i64 = row.get("sequence_number");
        let created_at: chrono::DateTime<Utc> = row.get("created_at");

        let prev_block_hash_vec: Vec<u8> = row.get("prev_block_hash");
        let content_hash_vec: Vec<u8> = row.get("content_hash");
        let block_hash_vec: Vec<u8> = row.get("block_hash");

        let prev_block_hash: [u8; 32] = prev_block_hash_vec.try_into().unwrap_or([0u8; 32]);
        let content_hash: [u8; 32] = content_hash_vec.try_into().unwrap_or([0u8; 32]);
        let block_hash: [u8; 32] = block_hash_vec.try_into().unwrap_or([0u8; 32]);

        groups.entry(case_id).or_default().push(ChainBlock {
            id: block_id,
            case_id,
            sequence_number,
            prev_block_hash,
            content_hash,
            block_hash,
            created_at,
        });
    }

    for (case_id, blocks) in &groups {
        if let Err(e) = verify_chain(blocks) {
            broken_count += 1;
            broken_case_ids.push(*case_id);
            tracing::error!(
                bat_id = "manual_verify",
                case_id = %case_id,
                error = %e,
                "ハッシュチェーン破断を検知しました",
            );
        }
    }

    Ok((
        StatusCode::OK,
        Json(HashChainVerifyResponse {
            verified_count,
            broken_count,
            broken_case_ids,
            verified_at: Utc::now(),
        }),
    ))
}

/// マスタ同期トリガー（POST /api/v1/ops/master-sync）。
///
/// AdminRole 必須。
#[utoipa::path(
    post,
    path = "/api/v1/ops/master-sync",
    tag = "ops",
    security(("Bearer" = [])),
    responses(
        (status = 202, description = "同期開始", body = MasterSyncResponse),
        (status = 401, description = "未認証"),
        (status = 403, description = "AdminRole 必須"),
    )
)]
pub async fn trigger_master_sync(
    _user: AuthenticatedUser<AdminRole>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    sqlx::query(
        r#"
        INSERT INTO outbox_events (id, event_id, event_type, status, retry_count, created_at)
        VALUES (gen_random_uuid(), gen_random_uuid(), 'master_sync_trigger', 'pending', 0, NOW())
        "#,
    )
    .execute(&state.write_pool)
    .await?;

    tracing::info!(event = "ops.master_sync.triggered", "マスタ同期をトリガーしました");

    Ok((
        StatusCode::ACCEPTED,
        Json(MasterSyncResponse {
            message: "Master sync has been triggered.".to_string(),
            started_at: Utc::now(),
        }),
    ))
}

/// Prometheus 互換メトリクス出力（GET /api/v1/ops/metrics）。
///
/// AuditorRole 以上が必要。
#[utoipa::path(
    get,
    path = "/api/v1/ops/metrics",
    tag = "ops",
    security(("Bearer" = [])),
    responses(
        (status = 200, description = "Prometheus テキスト形式メトリクス"),
        (status = 401, description = "未認証"),
        (status = 403, description = "権限不足"),
    )
)]
pub async fn metrics(
    _user: AuthenticatedUser<AuditorRole>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let write_pool_size = state.write_pool.size();
    let read_pool_size = state.read_pool.size();

    // MetricsResponse に格納してから metrics_text を取り出し、Prometheus 形式で返す
    let metrics = MetricsResponse {
        metrics_text: format!(
            r#"# HELP wnav_master_api_db_write_pool_size write connection pool size
# TYPE wnav_master_api_db_write_pool_size gauge
wnav_master_api_db_write_pool_size {write_pool_size}

# HELP wnav_master_api_db_read_pool_size read connection pool size
# TYPE wnav_master_api_db_read_pool_size gauge
wnav_master_api_db_read_pool_size {read_pool_size}

# HELP wnav_master_api_up master API uptime indicator
# TYPE wnav_master_api_up gauge
wnav_master_api_up 1
"#
        ),
    };

    let mut response = (StatusCode::OK, metrics.metrics_text).into_response();
    response.headers_mut().insert(
        CONTENT_TYPE,
        HeaderValue::from_static("text/plain; version=0.0.4"),
    );
    Ok(response)
}
