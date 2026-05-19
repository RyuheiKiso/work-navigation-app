// BAT-014: SSE Retry（MOD-BE-001）
//
// 1 分ごとに TBL-053（sse_dispatch_log）の failed レコードを再送試行する。
// dispatch_retry_max（CFG-030）回試行後も配信できなかった場合は failed のままにする。

use sqlx::PgPool;
use std::time::Duration;

/// BAT-014: SSE Retry タスクを起動する。
///
/// shutdown シグナル受信時に安全に終了する。
pub async fn run_sse_retry(pool: PgPool, mut shutdown: tokio::sync::broadcast::Receiver<()>) {
    tracing::info!(
        log_id = "LOG-BAT-014",
        event = "sse_retry.started",
        "SSE Retry タスクを起動した（60 秒間隔）"
    );

    loop {
        tokio::select! {
            _ = shutdown.recv() => {
                tracing::info!(
                    log_id = "LOG-BAT-014",
                    event = "sse_retry.shutdown",
                    "SSE Retry タスクがシャットダウンシグナルを受信した"
                );
                break;
            }
            _ = tokio::time::sleep(Duration::from_secs(60)) => {
                // 1 分間隔で実行する
            }
        }

        match retry_failed_dispatches(&pool).await {
            Ok(retried_count) => {
                if retried_count > 0 {
                    tracing::info!(
                        log_id = "LOG-BAT-014",
                        event = "sse_dispatch.retried",
                        count = retried_count,
                        "failed sse_dispatch_log レコードの再送を試行した"
                    );
                }
            }
            Err(e) => {
                tracing::error!(
                    log_id = "LOG-BAT-014",
                    event = "sse_retry.error",
                    error = %e,
                    "SSE 再送処理に失敗した"
                );
            }
        }
    }
}

/// TBL-053 の failed レコードを queued に戻して再送キューに積む。
///
/// dispatch_retry_max（CFG-030 = 5 回）未満のレコードのみ対象とする。
async fn retry_failed_dispatches(pool: &PgPool) -> Result<u64, sqlx::Error> {
    // 最大リトライ回数は 5 回（CFG-030）
    let max_retry = 5i32;

    let result = sqlx::query(
        r"
        UPDATE sse_dispatch_log
        SET status = 'queued', retry_count = retry_count + 1, updated_at = NOW()
        WHERE status = 'failed'
          AND retry_count < $1
        ",
    )
    .bind(max_retry)
    .execute(pool)
    .await?;

    Ok(result.rows_affected())
}
