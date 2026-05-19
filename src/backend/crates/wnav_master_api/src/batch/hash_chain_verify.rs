// BAT-001: ハッシュチェーン定期検証（Hash Chain Verifier）
//
// 実行スケジュール: 設定の `hash_chain_verify.cron` から読む（週次 月曜 03:00）。
// 全 case_id のハッシュチェーンを検証し、破断を検知した場合は tracing::error! でアラートを出す。
// SQLX_OFFLINE=true 環境のため sqlx::query() 動的クエリを使用する。

use sqlx::PgPool;
use std::time::Duration;

/// BAT-001 を常駐 tokio task として実行する。
pub async fn run(write_pool: PgPool, read_pool: PgPool, cron_expr: String) {
    tracing::info!(
        bat_id = "BAT-001",
        cron = %cron_expr,
        "ハッシュチェーン検証バッチを起動しました",
    );

    loop {
        // 週次（7 日周期）で実行する
        tokio::time::sleep(Duration::from_secs(7 * 24 * 3600)).await;

        tracing::info!(bat_id = "BAT-001", "ハッシュチェーン全量検証を開始します");

        run_verification(&read_pool).await;

        let _ = sqlx::query(
            r#"
            INSERT INTO batch_execution_logs
                (id, bat_id, status, executed_at)
            VALUES (gen_random_uuid(), 'BAT-001', 'completed', NOW())
            "#,
        )
        .execute(&write_pool)
        .await;
    }
}

/// ハッシュチェーン検証の実処理
async fn run_verification(read_pool: &PgPool) {
    use sqlx::Row as _;
    use wnav_hash_chain::{ChainBlock, verify_chain};
    use chrono::Utc;

    let case_ids: Vec<uuid::Uuid> = match sqlx::query_scalar(
        r#"SELECT DISTINCT case_id FROM work_event_blocks ORDER BY case_id"#,
    )
    .fetch_all(read_pool)
    .await
    {
        Ok(ids) => ids,
        Err(e) => {
            tracing::error!(bat_id = "BAT-001", error = %e, "case_id 一覧の取得に失敗しました");
            return;
        }
    };

    let mut verified_count: i64 = 0;
    let mut broken_count: i64 = 0;

    for case_id in &case_ids {
        let blocks = match sqlx::query(
            r#"
            SELECT id, case_id, sequence_number, prev_block_hash, content_hash, block_hash, created_at
            FROM work_event_blocks
            WHERE case_id = $1
            ORDER BY sequence_number ASC
            "#,
        )
        .bind(case_id)
        .fetch_all(read_pool)
        .await
        {
            Ok(b) => b,
            Err(e) => {
                tracing::error!(bat_id = "BAT-001", case_id = %case_id, error = %e, "ブロック取得に失敗しました");
                continue;
            }
        };

        let chain_blocks: Vec<ChainBlock> = blocks
            .iter()
            .map(|b| {
                let prev: Vec<u8> = b.get("prev_block_hash");
                let content: Vec<u8> = b.get("content_hash");
                let block: Vec<u8> = b.get("block_hash");
                let created_at: chrono::DateTime<Utc> = b.get("created_at");
                ChainBlock {
                    id: b.get("id"),
                    case_id: b.get("case_id"),
                    sequence_number: b.get("sequence_number"),
                    prev_block_hash: prev.try_into().unwrap_or([0u8; 32]),
                    content_hash: content.try_into().unwrap_or([0u8; 32]),
                    block_hash: block.try_into().unwrap_or([0u8; 32]),
                    created_at,
                }
            })
            .collect();

        verified_count += chain_blocks.len() as i64;

        if let Err(e) = verify_chain(&chain_blocks) {
            broken_count += 1;
            tracing::error!(
                bat_id = "BAT-001",
                case_id = %case_id,
                error = %e,
                "ハッシュチェーン破断を検知しました！改ざんの可能性があります",
            );
        }
    }

    tracing::info!(
        bat_id = "BAT-001",
        verified_count = verified_count,
        broken_count = broken_count,
        "ハッシュチェーン全量検証が完了しました",
    );

    if broken_count > 0 {
        tracing::error!(
            bat_id = "BAT-001",
            broken_count = broken_count,
            "ハッシュチェーン破断が検知されました。即時調査が必要です",
        );
    }
}
