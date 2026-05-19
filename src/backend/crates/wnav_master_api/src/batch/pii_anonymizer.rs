// BAT-004: PII 匿名化バッチ
//
// 月次実行。inactive ユーザーの個人情報を ANONYMIZED に置換（GDPR 対応）。
// SQLX_OFFLINE=true 環境のため sqlx::query() 動的クエリを使用する。

use sqlx::PgPool;
use std::time::Duration;

const MONTHLY_INTERVAL_SEC: u64 = 30 * 24 * 3600;
const ANONYMIZE_AFTER_DAYS: i64 = 90;

/// BAT-004 を常駐 tokio task として実行する。
pub async fn run(write_pool: PgPool) {
    tracing::info!(bat_id = "BAT-004", "PII 匿名化バッチを起動しました");

    loop {
        tokio::time::sleep(Duration::from_secs(MONTHLY_INTERVAL_SEC)).await;

        tracing::info!(bat_id = "BAT-004", "PII 匿名化処理を開始します");

        match run_anonymization(&write_pool).await {
            Ok(count) => {
                tracing::info!(bat_id = "BAT-004", anonymized_count = count, "PII 匿名化処理が完了しました");
            }
            Err(e) => {
                tracing::error!(bat_id = "BAT-004", error = %e, "PII 匿名化処理に失敗しました");
            }
        }

        let _ = sqlx::query(
            r#"
            INSERT INTO batch_execution_logs (id, bat_id, status, executed_at)
            VALUES (gen_random_uuid(), 'BAT-004', 'completed', NOW())
            "#,
        )
        .execute(&write_pool)
        .await;
    }
}

async fn run_anonymization(write_pool: &PgPool) -> Result<u64, sqlx::Error> {
    // 90 日以上前に非活性化されたユーザーを匿名化する
    let cutoff = chrono::Utc::now() - chrono::TimeDelta::days(ANONYMIZE_AFTER_DAYS);

    let result = sqlx::query(
        r#"
        UPDATE users
        SET
            login_id = 'ANONYMIZED_' || id::text,
            display_name = 'ANONYMIZED',
            email = 'ANONYMIZED_' || id::text || '@example.com',
            anonymized_at = NOW()
        WHERE
            is_active = false
            AND deactivated_at IS NOT NULL
            AND deactivated_at < $1
            AND anonymized_at IS NULL
        "#,
    )
    .bind(cutoff)
    .execute(write_pool)
    .await?;

    Ok(result.rows_affected())
}
