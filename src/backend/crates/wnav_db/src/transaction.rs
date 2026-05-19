// トランザクション境界設計（FNC-BE-008）
// WorkEvent と OutboxEvent は必ず同一 PostgreSQL トランザクションに含める。
// この設計により「WorkEvent は記録されたが OutboxEvent が記録されなかった」
// という非整合状態を排除する。

use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use wnav_domain::{error::DomainError, model::work_event::WorkEvent};

/// (FNC-BE-008) WorkEvent + OutboxEvent を同一トランザクションで記録する。
///
/// # 処理順序
/// 1. Idempotency Key を TBL-035 に INSERT（重複なら冪等応答）
/// 2. 前イベントの content_hash を FOR UPDATE で取得
/// 3. wnav_hash_chain で content_hash を計算
/// 4. TBL-001 work_events に INSERT（Append-only）
/// 5. TBL-003 outbox_events に INSERT（MSG-001）
/// 6. COMMIT
///
/// # 冪等性保証
/// Idempotency Key が既に存在する場合はロールバックして冪等応答を返す。
pub async fn record_step_completed_tx(
    pool: &PgPool,
    event: WorkEvent,
    idempotency_key: Uuid,
    endpoint: &str,
) -> Result<(), DomainError> {
    let mut tx: Transaction<'_, Postgres> = pool.begin().await.map_err(crate::error::map_sqlx)?;

    // 1. Idempotency Key を TBL-035 に INSERT（重複なら CONFLICT → 冪等応答）
    let conflict = sqlx::query(
        r#"
        INSERT INTO idempotency_keys (idempotency_key, response_body, expires_at)
        VALUES ($1, '{}', NOW() + INTERVAL '86400 seconds')
        ON CONFLICT (idempotency_key) DO NOTHING
        "#,
    )
    .bind(idempotency_key)
    .execute(&mut *tx)
    .await
    .map_err(crate::error::map_sqlx)?;

    if conflict.rows_affected() == 0 {
        // 既存の Idempotency Key → 冪等応答（ERR-DB-001）
        tx.rollback().await.ok();
        return Err(DomainError::DuplicateExternalKey {
            key: idempotency_key.to_string(),
        });
    }

    // 2. 前イベントの content_hash を取得（FOR UPDATE でロック）
    let prev_hash: Option<String> = sqlx::query_scalar(
        r#"
        SELECT content_hash
        FROM work_events
        WHERE case_id = $1
        ORDER BY event_id DESC
        LIMIT 1
        FOR UPDATE
        "#,
    )
    .bind(event.case_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(crate::error::map_sqlx)?;

    // genesis の場合は 64 桁の "0" を使用する
    let prev_hash = prev_hash.unwrap_or_else(|| "0".repeat(64));

    // 3. content_hash 計算（wnav_hash_chain に委譲）
    // canonical JSON → SHA-256 の 2 ステップで計算する
    let canonical = wnav_hash_chain::canonical_json(&event.payload);
    let content_bytes = wnav_hash_chain::compute_content_hash(&canonical);
    let content_hash_hex = wnav_hash_chain::bytes32_to_hex(&content_bytes);

    // 4. WorkEvent INSERT（TBL-001、Append-only）
    sqlx::query(
        r#"
        INSERT INTO work_events (
            event_id, case_id, activity, step_id,
            timestamp_client, timestamp_server, resource,
            sop_version_id, terminal_id, payload,
            prev_hash, content_hash
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
        "#,
    )
    .bind(event.event_id)
    .bind(event.case_id)
    .bind(&event.activity)
    .bind(event.step_id)
    .bind(event.timestamp_client)
    .bind(event.timestamp_server)
    .bind(event.resource)
    .bind(event.sop_version_id)
    .bind(event.terminal_id)
    .bind(&event.payload)
    .bind(&prev_hash)
    .bind(&content_hash_hex)
    .execute(&mut *tx)
    .await
    .map_err(crate::error::map_sqlx)?;

    // 5. OutboxEvent INSERT（TBL-003、MSG-001）
    let outbox_payload = serde_json::to_value(&event)
        .map_err(|e| DomainError::Internal(format!("シリアライズ失敗: {e}")))?;

    sqlx::query(
        r#"
        INSERT INTO outbox_events (
            outbox_id, event_id, idempotency_key, event_type,
            payload, status, retry_count, last_attempted_at
        )
        VALUES ($1, $2, $3, 'outbox.work_event', $4, 'PENDING', 0, NULL)
        "#,
    )
    .bind(Uuid::now_v7())
    .bind(event.event_id)
    .bind(idempotency_key)
    .bind(outbox_payload)
    .execute(&mut *tx)
    .await
    .map_err(crate::error::map_sqlx)?;

    // endpoint は audit trail として利用可能にするが、現バージョンではログに留める
    tracing::debug!(
        idempotency_key = %idempotency_key,
        event_id = %event.event_id,
        endpoint = endpoint,
        "WorkEvent + OutboxEvent をコミットします"
    );

    tx.commit().await.map_err(crate::error::map_sqlx)?;
    Ok(())
}
