// BAT-013: CaseLock Reaper（MOD-BE-001）
//
// 60 秒ごとに 5 分（300 秒）超過の ACTIVE case_lock を EXPIRED に更新する。
// wnav_terminal_api の main.rs で tokio::spawn して起動する。
// event_insert_pool を使用する（case_locks は INSERT/UPDATE/DELETE 許可の例外テーブル）。

use sqlx::PgPool;
use std::time::Duration;

/// BAT-013: CaseLock Reaper を起動する。
///
/// shutdown シグナル受信時に安全に終了する。
pub async fn run_case_lock_reaper(
    pool: PgPool,
    mut shutdown: tokio::sync::broadcast::Receiver<()>,
) {
    tracing::info!(
        log_id = "LOG-BAT-013",
        event = "case_lock_reaper.started",
        "CaseLock Reaper を起動した（60 秒間隔）"
    );

    loop {
        tokio::select! {
            _ = shutdown.recv() => {
                tracing::info!(
                    log_id = "LOG-BAT-013",
                    event = "case_lock_reaper.shutdown",
                    "CaseLock Reaper がシャットダウンシグナルを受信した"
                );
                break;
            }
            _ = tokio::time::sleep(Duration::from_secs(60)) => {
                // 60 秒間隔で実行する
            }
        }

        match expire_stale_locks(&pool).await {
            Ok(expired_count) => {
                if expired_count > 0 {
                    tracing::info!(
                        log_id = "LOG-BAT-013",
                        event = "case_lock.expired",
                        count = expired_count,
                        "ハートビートタイムアウトの case_lock を EXPIRED に更新した"
                    );
                }
            }
            Err(e) => {
                tracing::error!(
                    log_id = "LOG-BAT-013",
                    event = "case_lock_reaper.error",
                    error = %e,
                    "case_lock の期限切れ処理に失敗した"
                );
            }
        }
    }
}

/// ハートビートが 5 分（300 秒）超過した ACTIVE case_lock を EXPIRED に更新する。
///
/// 5 分（CFG の閾値）以上ハートビートが更新されていない case_lock を期限切れにする。
async fn expire_stale_locks(pool: &PgPool) -> Result<u64, sqlx::Error> {
    let result = sqlx::query(
        r"
        UPDATE case_locks
        SET status = 'EXPIRED'
        WHERE status = 'ACTIVE'
          AND heartbeat_at < NOW() - INTERVAL '5 minutes'
        ",
    )
    .execute(pool)
    .await?;

    Ok(result.rows_affected())
}
