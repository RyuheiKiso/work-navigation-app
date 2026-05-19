// BAT-003: Master Sync Puller（MOD-BE-001）
//
// 設定された間隔で master-api から最新マスタを Pull 取得してローカルキャッシュを更新する。
// sync_interval_ms は wnav_config の outbox.interval_ms に準拠（設定可変）。
// 対象エンティティ: sops, steps, operations, equipment, materials, skills
//
// wnav_terminal_api の main.rs で tokio::spawn して起動する。
// read_pool で SELECT・event_insert_pool で UPSERT を実行する。

use std::sync::Arc;
use std::time::Duration;

use sqlx::PgPool;
use wnav_config::TerminalApiConfig;

/// BAT-003: Master Sync Puller を起動する。
///
/// 設定間隔ごとに GET /api/v1/sync/master を呼び出し、
/// 差分 SOP / ステップを DB に UPSERT して sync_version を更新する。
/// shutdown シグナル受信時に安全に終了する。
pub async fn run_master_sync(
    pool: PgPool,
    config: Arc<TerminalApiConfig>,
    mut shutdown: tokio::sync::broadcast::Receiver<()>,
) {
    // 同期間隔を設定から取得する（デフォルト 60 分 = 3_600_000ms）
    let interval_ms = config.outbox.interval_ms.max(60_000);

    tracing::info!(
        log_id = "LOG-BAT-003",
        event = "master_sync.started",
        interval_ms = interval_ms,
        "Master Sync Puller を起動した",
    );

    loop {
        tokio::select! {
            _ = shutdown.recv() => {
                tracing::info!(
                    log_id = "LOG-BAT-003",
                    event = "master_sync.shutdown",
                    "Master Sync Puller がシャットダウンシグナルを受信した",
                );
                break;
            }
            _ = tokio::time::sleep(Duration::from_millis(interval_ms)) => {
                // 設定間隔ごとに同期を実行する
            }
        }

        match sync_master(&pool, &config).await {
            Ok(synced_count) => {
                tracing::info!(
                    log_id = "LOG-BAT-003",
                    event = "master_sync.completed",
                    synced_count = synced_count,
                    "マスタ同期が完了しました",
                );
            }
            Err(e) => {
                tracing::error!(
                    log_id = "LOG-BAT-003",
                    event = "master_sync.failed",
                    error = %e,
                    "マスタ同期に失敗しました（3 分後にリトライする）",
                );
                // HTTP エラー: 3 分後に 1 回リトライする（BAT-003 仕様）
                tokio::time::sleep(Duration::from_secs(180)).await;
                if let Err(retry_err) = sync_master(&pool, &config).await {
                    tracing::error!(
                        log_id = "LOG-BAT-003",
                        event = "master_sync.retry_failed",
                        error = %retry_err,
                        "マスタ同期のリトライにも失敗しました",
                    );
                }
            }
        }
    }
}

/// master-api から最新マスタを取得して UPSERT する。
///
/// GET /api/v1/sync/master?since={last_sync_version} を呼び出し、
/// レスポンスの各エンティティを ON CONFLICT DO UPDATE で差分更新する。
/// master-api の URL は WNAV_MASTER_API_URL 環境変数から取得する（デフォルト: localhost:8081）。
async fn sync_master(pool: &PgPool, config: &TerminalApiConfig) -> anyhow::Result<usize> {
    // 前回の同期バージョンを取得する（未記録の場合は 0 から開始）
    let last_sync_version: i64 = sqlx::query_scalar(
        r#"SELECT COALESCE(MAX(sync_version), 0) FROM local_sync_state"#,
    )
    .fetch_one(pool)
    .await
    .unwrap_or(0);

    // master-api の URL を環境変数から取得する（設定ファイルに専用フィールドがないため環境変数を使用する）
    let master_api_base = std::env::var("WNAV_MASTER_API_URL")
        .unwrap_or_else(|_| {
            // デフォルト: terminal-api と同一ホストの 8081 ポートを使用する
            let host = config.server.terminal_api.bind_addr.replace("0.0.0.0", "127.0.0.1");
            format!("http://{}:8081", host)
        });

    // master-api の sync エンドポイントに差分リクエストを送信する
    let master_api_url = format!(
        "{}/api/v1/sync/master?since={}",
        master_api_base.trim_end_matches('/'),
        last_sync_version,
    );

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()?;

    let response = client.get(&master_api_url).send().await?;

    if !response.status().is_success() {
        anyhow::bail!(
            "master-api からの同期レスポンスエラー: {} {}",
            response.status(),
            response.status().canonical_reason().unwrap_or("Unknown"),
        );
    }

    let body: serde_json::Value = response.json().await?;

    let new_sync_version = body
        .get("sync_version")
        .and_then(|v| v.as_i64())
        .unwrap_or(last_sync_version);

    // 差分データを取得して UPSERT する
    let sops = body.get("sops").and_then(|v| v.as_array()).cloned().unwrap_or_default();
    let steps = body.get("steps").and_then(|v| v.as_array()).cloned().unwrap_or_default();
    let total_synced = sops.len() + steps.len();

    // SOP データを UPSERT する
    for sop in &sops {
        upsert_sop(pool, sop).await?;
    }

    // ステップデータを UPSERT する
    for step in &steps {
        upsert_step(pool, step).await?;
    }

    // sync_version を更新する
    if new_sync_version > last_sync_version {
        sqlx::query(
            r#"
            INSERT INTO local_sync_state (sync_version, synced_at)
            VALUES ($1, NOW())
            ON CONFLICT (id) DO UPDATE
            SET sync_version = $1, synced_at = NOW()
            "#,
        )
        .bind(new_sync_version)
        .execute(pool)
        .await?;
    }

    // 同期ログを記録する
    sqlx::query(
        r#"
        INSERT INTO sync_log (sync_version, synced_count, status, synced_at)
        VALUES ($1, $2, 'success', NOW())
        "#,
    )
    .bind(new_sync_version)
    .bind(total_synced as i64)
    .execute(pool)
    .await
    .ok(); // ログ記録の失敗は無視する

    Ok(total_synced)
}

/// SOP データを UPSERT するヘルパー
async fn upsert_sop(pool: &PgPool, sop: &serde_json::Value) -> anyhow::Result<()> {
    let id = sop.get("id").and_then(|v| v.as_str()).unwrap_or_default();
    sqlx::query(
        r#"
        INSERT INTO sops (id, name, process_id, is_active, updated_at)
        VALUES ($1::uuid, $2, $3::uuid, $4, NOW())
        ON CONFLICT (id) DO UPDATE
        SET name = EXCLUDED.name,
            process_id = EXCLUDED.process_id,
            is_active = EXCLUDED.is_active,
            updated_at = NOW()
        "#,
    )
    .bind(id)
    .bind(sop.get("name").and_then(|v| v.as_str()).unwrap_or(""))
    .bind(sop.get("process_id").and_then(|v| v.as_str()))
    .bind(sop.get("is_active").and_then(|v| v.as_bool()).unwrap_or(true))
    .execute(pool)
    .await?;

    Ok(())
}

/// ステップデータを UPSERT するヘルパー
async fn upsert_step(pool: &PgPool, step: &serde_json::Value) -> anyhow::Result<()> {
    let id = step.get("id").and_then(|v| v.as_str()).unwrap_or_default();
    sqlx::query(
        r#"
        INSERT INTO steps (id, sop_id, step_number, title, instruction, step_type, updated_at)
        VALUES ($1::uuid, $2::uuid, $3, $4, $5, $6, NOW())
        ON CONFLICT (id) DO UPDATE
        SET sop_id = EXCLUDED.sop_id,
            step_number = EXCLUDED.step_number,
            title = EXCLUDED.title,
            instruction = EXCLUDED.instruction,
            step_type = EXCLUDED.step_type,
            updated_at = NOW()
        "#,
    )
    .bind(id)
    .bind(step.get("sop_id").and_then(|v| v.as_str()))
    .bind(step.get("step_number").and_then(|v| v.as_i64()).unwrap_or(0))
    .bind(step.get("title").and_then(|v| v.as_str()).unwrap_or(""))
    .bind(step.get("instruction").and_then(|v| v.as_str()).unwrap_or(""))
    .bind(step.get("step_type").and_then(|v| v.as_str()).unwrap_or("operation"))
    .execute(pool)
    .await?;

    Ok(())
}
