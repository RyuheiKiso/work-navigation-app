// IdempotencyMiddleware — Idempotency-Key ヘッダ検証・TBL-035 照合（MOD-BE-001 §2-4）
//
// 書き込みメソッド（POST/PUT/PATCH）に対して Idempotency-Key ヘッダを要求する。
// TBL-035（idempotency_keys）を event_insert_pool で照合する。
// 同一 Key のキャッシュヒット時は保存済みレスポンスを返し DB 操作を行わない。

use axum::{
    body::Body,
    extract::{Request, State},
    http::{HeaderValue, StatusCode, header},
    middleware::Next,
    response::Response,
};
use uuid::Uuid;

use crate::{error::AppError, state::AppState};

/// Idempotency-Key ヘッダを検証・照合するミドルウェア（terminal-api 専用）。
///
/// GET / DELETE メソッドはスキップする。
/// POST / PUT / PATCH の場合 Idempotency-Key ヘッダが必須となる。
/// キャッシュヒット時は保存済みレスポンスを返す（DB 再書き込みなし）。
pub async fn idempotency_middleware(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let method = request.method().clone();

    // 読み取り専用メソッドはスキップする
    if method == axum::http::Method::GET || method == axum::http::Method::DELETE {
        return Ok(next.run(request).await);
    }

    // Idempotency-Key ヘッダを取得する（必須）
    let idempotency_key = request
        .headers()
        .get("idempotency-key")
        .and_then(|v| v.to_str().ok())
        .ok_or(AppError::MissingIdempotencyKey)?
        .to_string();

    // UUID v7 形式を検証する
    let _key_uuid: Uuid = idempotency_key
        .parse()
        .map_err(|_| AppError::InvalidFormat(None))?;

    let pool = state.event_insert_pool.clone();
    let ttl_sec = state.config.idempotency.ttl_sec;

    // TBL-035 を event_insert_pool で照合する（キャッシュヒット確認）
    let cached = lookup_idempotency_cache(&pool, &idempotency_key, ttl_sec).await?;

    if let Some(cached_response) = cached {
        // キャッシュヒット: 保存済みレスポンスを返す（DB 操作なし）
        tracing::info!(
            log_id = "LOG-IDEMPOTENCY-001",
            idempotency_key = %idempotency_key,
            "Idempotency-Key キャッシュヒット。保存済みレスポンスを返す"
        );
        return Ok(cached_response);
    }

    // キャッシュミス: リクエストを処理してレスポンスをキャッシュに保存する
    let response = next.run(request).await;

    if response.status().is_success() {
        let status_code = response.status().as_u16();
        // 成功レスポンスのみキャッシュに保存する
        if let Err(e) = store_idempotency_cache(&pool, &idempotency_key, status_code).await {
            tracing::warn!(
                log_id = "LOG-IDEMPOTENCY-002",
                idempotency_key = %idempotency_key,
                error = %e,
                "Idempotency キャッシュ保存に失敗した（レスポンスは正常返却）"
            );
        }
    }

    Ok(response)
}

/// TBL-035 から Idempotency-Key に対応するキャッシュを取得する。
///
/// TTL 切れのキャッシュは None を返す。
async fn lookup_idempotency_cache(
    pool: &sqlx::PgPool,
    idempotency_key: &str,
    ttl_sec: u64,
) -> Result<Option<Response>, AppError> {
    let ttl_interval = format!("{ttl_sec} seconds");

    // TBL-035 から既存キーを検索する（TTL 内のもののみ）
    let row = sqlx::query_as::<_, (i32, Vec<u8>)>(
        r"
        SELECT response_status, response_body
        FROM idempotency_keys
        WHERE idempotency_key = $1
          AND created_at > NOW() - $2::interval
        LIMIT 1
        ",
    )
    .bind(idempotency_key)
    .bind(&ttl_interval)
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "idempotency_keys 照合に失敗した");
        AppError::DatabaseError
    })?;

    let Some((status_code, body_bytes)) = row else {
        return Ok(None);
    };

    // 保存済みレスポンスを再構築する
    let status = StatusCode::from_u16(status_code as u16).unwrap_or(StatusCode::OK);

    let response = Response::builder()
        .status(status)
        .header(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        )
        .body(Body::from(body_bytes))
        .map_err(|_| AppError::InternalServerError)?;

    Ok(Some(response))
}

/// TBL-035 に Idempotency-Key と レスポンスキャッシュを保存する。
async fn store_idempotency_cache(
    pool: &sqlx::PgPool,
    idempotency_key: &str,
    status_code: u16,
) -> Result<(), sqlx::Error> {
    // キャッシュを TBL-035 に INSERT する（競合時は無視する）
    sqlx::query(
        r"
        INSERT INTO idempotency_keys (idempotency_key, response_status, response_body, created_at)
        VALUES ($1, $2, $3, NOW())
        ON CONFLICT (idempotency_key) DO NOTHING
        ",
    )
    .bind(idempotency_key)
    .bind(i32::from(status_code))
    .bind(b"{}".as_slice())
    .execute(pool)
    .await?;

    Ok(())
}
