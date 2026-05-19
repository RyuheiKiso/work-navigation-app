// 同期 API（API-sync-001〜002）ハンドラ（07_運用・監視API仕様.md §5〜6）
//
// GET  /api/v1/sync/master          — マスタ差分同期
// POST /api/v1/sync/outbox/inbound  — ローカル Outbox 送信
//
// SQLX_OFFLINE=true 環境のため sqlx::query() を使用する。cargo sqlx prepare 後に sqlx::query! に切り替えること。

use axum::{
    Extension, Json,
    extract::{Query, State},
    http::StatusCode,
};
use chrono::Utc;

use crate::{
    dto::{
        response_envelope::ApiResponse,
        sync::{MasterSyncData, MasterSyncQuery, OutboxInboundData, OutboxInboundRequest},
    },
    error::AppError,
    state::AppState,
};
use wnav_auth::CurrentUser;

/// GET /api/v1/sync/master — マスタ差分同期（API-sync-001）
///
/// 子機がマスタキャッシュを取得するエンドポイント。
/// `since` パラメータ以降に更新されたデータのみを返す（差分同期）。
#[utoipa::path(
    get,
    path = "/api/v1/sync/master",
    operation_id = "syncMaster",
    params(
        ("since" = Option<chrono::DateTime<chrono::Utc>>, Query, description = "差分同期基点時刻"),
        ("resource_types" = Option<String>, Query, description = "カンマ区切りのリソース種別"),
    ),
    responses(
        (status = 200, description = "マスタ差分取得成功", body = ApiResponse<MasterSyncData>),
    ),
    security(("bearer_auth" = [])),
    tag = "sync",
)]
pub async fn sync_master(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(query): Query<MasterSyncQuery>,
) -> Result<Json<ApiResponse<MasterSyncData>>, AppError> {
    let since = query.since.unwrap_or_else(|| {
        // デフォルトは 24 時間前
        Utc::now() - chrono::Duration::hours(24)
    });

    // SOP データを取得する（差分同期）
    let sops: Vec<serde_json::Value> =
        sqlx::query_as::<_, (uuid::Uuid, String, String, chrono::DateTime<Utc>)>(
            r"
        SELECT id, name_json::text, version, updated_at
        FROM sops
        WHERE factory_id = $1 AND updated_at > $2 AND deleted_at IS NULL
        ORDER BY updated_at ASC
        LIMIT 1000
        ",
        )
        .bind(current_user.factory_id)
        .bind(since)
        .fetch_all(&state.read_pool)
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|(id, name_json, version, updated_at)| {
            serde_json::json!({
                "id": id,
                "name_json": name_json,
                "version": version,
                "updated_at": updated_at,
            })
        })
        .collect();

    let data = MasterSyncData {
        sync_timestamp: Utc::now(),
        sops,
        processes: Vec::new(),
        users: Vec::new(),
        has_more: false,
    };

    Ok(Json(ApiResponse::new(data)))
}

/// POST /api/v1/sync/outbox/inbound — ローカル Outbox 送信（API-sync-002）
///
/// 子機バックエンドから親機へ Outbox イベントを一括送信する（最大 100 件）。
/// 各 outbox_event_id で Idempotency チェックを行う。
#[utoipa::path(
    post,
    path = "/api/v1/sync/outbox/inbound",
    operation_id = "syncOutboxInbound",
    request_body = OutboxInboundRequest,
    responses(
        (status = 200, description = "Outbox 送信成功", body = ApiResponse<OutboxInboundData>),
        (status = 422, description = "バリデーションエラー"),
    ),
    security(("bearer_auth" = [])),
    tag = "sync",
)]
pub async fn sync_outbox_inbound(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Json(body): Json<OutboxInboundRequest>,
) -> Result<(StatusCode, Json<ApiResponse<OutboxInboundData>>), AppError> {
    // 最大 100 件チェック
    if body.events.len() > 100 {
        return Err(AppError::ValueOutOfRange(None));
    }

    let mut accepted_count = 0i32;
    let mut skipped_count = 0i32;

    for event in &body.events {
        // Idempotency チェック（既存の outbox_event_id は無視する）
        let exists: bool = sqlx::query_as::<_, (bool,)>(
            r"SELECT EXISTS(SELECT 1 FROM outbox_events WHERE outbox_id = $1)",
        )
        .bind(event.outbox_event_id)
        .fetch_one(&state.read_pool)
        .await
        .map(|(e,)| e)
        .unwrap_or(false);

        if exists {
            skipped_count += 1;
            continue;
        }

        // 新規イベントを outbox_events に INSERT する
        let _ = sqlx::query(
            r"
            INSERT INTO outbox_events
                (outbox_id, event_type, payload, status, source_factory_id, created_at)
            VALUES ($1, $2, $3, 'PENDING', $4, $5)
            ON CONFLICT (outbox_id) DO NOTHING
            ",
        )
        .bind(event.outbox_event_id)
        .bind(&event.event_type)
        .bind(&event.payload)
        .bind(body.source_factory_id)
        .bind(event.occurred_at)
        .execute(&state.event_insert_pool)
        .await;

        accepted_count += 1;
    }

    let data = OutboxInboundData {
        accepted_count,
        skipped_count,
    };

    Ok((StatusCode::OK, Json(ApiResponse::new(data))))
}
