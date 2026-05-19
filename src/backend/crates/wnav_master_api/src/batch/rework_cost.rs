// BAT-011: リワーク・コスト集計バッチ
//
// 日次。kaizen_reports テーブルへの集計 INSERT。
// SQLX_OFFLINE=true 環境のため sqlx::query() 動的クエリを使用する。

use sqlx::PgPool;
use std::time::Duration;

const DAILY_INTERVAL_SEC: u64 = 24 * 3600;

/// BAT-011 を常駐 tokio task として実行する。
pub async fn run(write_pool: PgPool, read_pool: PgPool) {
    tracing::info!(bat_id = "BAT-011", "リワーク・コスト集計バッチを起動しました");

    loop {
        tokio::time::sleep(Duration::from_secs(DAILY_INTERVAL_SEC)).await;

        tracing::info!(bat_id = "BAT-011", "リワーク・コスト集計処理を開始します");

        match run_aggregation(&write_pool, &read_pool).await {
            Ok(inserted) => {
                tracing::info!(bat_id = "BAT-011", inserted_count = inserted, "リワーク・コスト集計処理が完了しました");
            }
            Err(e) => {
                tracing::error!(bat_id = "BAT-011", error = %e, "リワーク・コスト集計処理に失敗しました");
            }
        }

        let _ = sqlx::query(
            r#"
            INSERT INTO batch_execution_logs (id, bat_id, status, executed_at)
            VALUES (gen_random_uuid(), 'BAT-011', 'completed', NOW())
            "#,
        )
        .execute(&write_pool)
        .await;
    }
}

async fn run_aggregation(write_pool: &PgPool, read_pool: &PgPool) -> Result<u64, sqlx::Error> {
    use sqlx::Row as _;

    let today = chrono::Utc::now().date_naive();
    let yesterday = today - chrono::TimeDelta::days(1);
    let yesterday_start = chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(
        yesterday.and_hms_opt(0, 0, 0).unwrap_or_default(),
        chrono::Utc,
    );
    let yesterday_end = chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(
        yesterday.and_hms_opt(23, 59, 59).unwrap_or_default(),
        chrono::Utc,
    );

    let stats = sqlx::query(
        r#"
        SELECT
            COUNT(*) AS rework_count,
            COALESCE(SUM(planned_hours), 0) AS total_hours,
            reason_code
        FROM reworks
        WHERE created_at BETWEEN $1 AND $2
        GROUP BY reason_code
        "#,
    )
    .bind(yesterday_start)
    .bind(yesterday_end)
    .fetch_all(read_pool)
    .await?;

    let mut inserted: u64 = 0;

    for stat in &stats {
        let rework_count: i64 = stat.get("rework_count");
        let total_hours: f64 = stat.get("total_hours");
        let reason_code: String = stat.get("reason_code");

        sqlx::query(
            r#"
            INSERT INTO kaizen_reports
                (id, report_date, rework_count, total_hours, reason_code, created_at)
            VALUES (gen_random_uuid(), $1, $2, $3, $4, NOW())
            ON CONFLICT (report_date, reason_code)
            DO UPDATE SET rework_count = EXCLUDED.rework_count, total_hours = EXCLUDED.total_hours
            "#,
        )
        .bind(yesterday)
        .bind(rework_count as i32)
        .bind(total_hours)
        .bind(&reason_code)
        .execute(write_pool)
        .await?;
        inserted += 1;
    }

    Ok(inserted)
}
