// BAT-005: PostgreSQL バックアップ通知バッチ
//
// 日次 02:00 JST。外部通知 URL へのバックアップ完了通知 HTTP POST。
// SQLX_OFFLINE=true 環境のため sqlx::query() 動的クエリを使用する。

use sqlx::PgPool;
use std::time::Duration;

const DAILY_INTERVAL_SEC: u64 = 24 * 3600;

/// BAT-005 を常駐 tokio task として実行する。
pub async fn run(write_pool: PgPool, backup_notification_url: String) {
    tracing::info!(
        bat_id = "BAT-005",
        "PG バックアップ通知バッチを起動しました"
    );

    loop {
        tokio::time::sleep(Duration::from_secs(DAILY_INTERVAL_SEC)).await;

        tracing::info!(bat_id = "BAT-005", "バックアップ通知処理を開始します");

        match send_backup_notification(&backup_notification_url).await {
            Ok(()) => {
                tracing::info!(bat_id = "BAT-005", "バックアップ完了通知を送信しました");
            }
            Err(e) => {
                tracing::error!(bat_id = "BAT-005", error = %e, "バックアップ完了通知の送信に失敗しました");
            }
        }

        let _ = sqlx::query(
            r#"
            INSERT INTO batch_execution_logs (id, bat_id, status, executed_at)
            VALUES (gen_random_uuid(), 'BAT-005', 'completed', NOW())
            "#,
        )
        .execute(&write_pool)
        .await;
    }
}

async fn send_backup_notification(url: &str) -> Result<(), reqwest::Error> {
    if url.is_empty() {
        tracing::debug!(
            bat_id = "BAT-005",
            "通知 URL が設定されていません。スキップします"
        );
        return Ok(());
    }

    let client = reqwest::Client::new();
    let payload = serde_json::json!({
        "event": "pg_backup_completed",
        "service": "wnav_master_api",
        "timestamp": chrono::Utc::now().to_rfc3339(),
    });

    client
        .post(url)
        .timeout(Duration::from_secs(30))
        .json(&payload)
        .send()
        .await?;

    Ok(())
}
