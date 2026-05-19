// BAT-006〜010: レポート生成バッチ（イベント駆動）
//
// report_jobs テーブルをポーリングして 'queued' ジョブを処理する。
// SQLX_OFFLINE=true 環境のため sqlx::query() 動的クエリを使用する。

use sqlx::PgPool;
use std::time::Duration;

const POLL_INTERVAL_SEC: u64 = 30;

/// BAT-006〜010 レポート生成バッチを常駐 tokio task として実行する。
pub async fn run(write_pool: PgPool, read_pool: PgPool) {
    tracing::info!(bat_id = "BAT-006", "レポート生成バッチを起動しました");

    loop {
        tokio::time::sleep(Duration::from_secs(POLL_INTERVAL_SEC)).await;

        use sqlx::Row as _;

        let jobs = match sqlx::query(
            r#"
            SELECT id, report_type, from_date, to_date, filters, format, requested_by
            FROM report_jobs
            WHERE status = 'queued'
            ORDER BY created_at ASC
            LIMIT 5
            "#,
        )
        .fetch_all(&read_pool)
        .await
        {
            Ok(j) => j,
            Err(e) => {
                tracing::error!(bat_id = "BAT-006", error = %e, "レポートジョブ取得に失敗しました");
                continue;
            }
        };

        for job in &jobs {
            let job_id: uuid::Uuid = job.get("id");
            let report_type: String = job.get("report_type");

            let _ = sqlx::query(r#"UPDATE report_jobs SET status = 'running' WHERE id = $1"#)
                .bind(job_id)
                .execute(&write_pool)
                .await;

            tracing::info!(bat_id = "BAT-006", job_id = %job_id, report_type = %report_type, "レポート生成を開始します");

            match generate_report(&write_pool, &report_type, job_id).await {
                Ok(download_url) => {
                    let _ = sqlx::query(
                        r#"
                        UPDATE report_jobs
                        SET status = 'completed', download_url = $1, completed_at = NOW(),
                            expires_at = NOW() + INTERVAL '7 days'
                        WHERE id = $2
                        "#,
                    )
                    .bind(&download_url)
                    .bind(job_id)
                    .execute(&write_pool)
                    .await;

                    tracing::info!(bat_id = "BAT-006", job_id = %job_id, "レポート生成が完了しました");
                }
                Err(e) => {
                    let _ = sqlx::query(
                        r#"UPDATE report_jobs SET status = 'failed', error_message = $1 WHERE id = $2"#,
                    )
                    .bind(&e)
                    .bind(job_id)
                    .execute(&write_pool)
                    .await;

                    tracing::error!(bat_id = "BAT-006", job_id = %job_id, error = %e, "レポート生成に失敗しました");
                }
            }
        }
    }
}

/// レポート種別に応じてレポートを生成してダウンロード URL を返す（スケルトン実装）。
async fn generate_report(
    write_pool: &PgPool,
    report_type: &str,
    job_id: uuid::Uuid,
) -> Result<String, String> {
    let filename = format!("report_{job_id}.pdf");

    let _ = sqlx::query(
        r#"
        INSERT INTO report_files (id, job_id, filename, created_at)
        VALUES (gen_random_uuid(), $1, $2, NOW())
        ON CONFLICT DO NOTHING
        "#,
    )
    .bind(job_id)
    .bind(&filename)
    .execute(write_pool)
    .await;

    tracing::debug!(bat_id = "BAT-006", report_type = %report_type, job_id = %job_id, "レポート生成処理（スケルトン実装）");

    Ok(format!("/api/v1/report-files/{filename}"))
}
