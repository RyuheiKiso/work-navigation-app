// Outbox Consumer（MOD-BE-006 / BAT-002）
// PENDING 状態のOutboxEventを取得して親機APIにHMAC-SHA256署名付きでPOSTする常駐tokioタスク。
// FOR UPDATE SKIP LOCKED で複数インスタンス間の二重配信を防止する（Active-Standby 設計）。
// 指数バックオフ: initial_backoff_ms * 2^(retry_count-1)（上限 max_backoff_ms、ALG-009）
//
// NOTE: sqlx::query! マクロはコンパイル時 DB 検証（SQLX_OFFLINE キャッシュ）が必要。
//       DB が利用可能になったら `cargo sqlx prepare` を実行してキャッシュを生成し、
//       動的クエリ（sqlx::query_as/execute）を sqlx::query! に切り替えること。

use std::sync::Arc;

use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::OutboxError;
use wnav_webhook::{WebhookSender, signature::sign_payload};

/// Outbox Consumer の設定（CFG-003〜005 対応）。
#[derive(Debug, Clone, serde::Deserialize)]
pub struct OutboxConfig {
    /// ポーリング間隔（ミリ秒、CFG-003、デフォルト 60000）
    pub poll_interval_ms: u64,
    /// 1 回のポーリングで処理するバッチサイズ（デフォルト 10）
    pub batch_size: i64,
    /// 最大リトライ回数（CFG-004、デフォルト 5）
    pub max_retry_attempts: u32,
    /// バックオフ初期値（ミリ秒、デフォルト 1000）
    pub initial_backoff_ms: u64,
    /// バックオフ最大値（ミリ秒、デフォルト 32000）
    pub max_backoff_ms: u64,
    /// 親機 API エンドポイント URL（IF-002）
    /// 環境変数 WNAV_TERMINAL_PARENT_API_URL から取得する
    pub parent_api_url: String,
    /// HMAC 署名秘密鍵（UTF-8 文字列として使用）
    pub hmac_secret: String,
    /// HTTP タイムアウト秒（デフォルト 30）
    pub http_timeout_secs: u64,
}

impl Default for OutboxConfig {
    fn default() -> Self {
        Self {
            poll_interval_ms: 60_000,
            batch_size: 10,
            max_retry_attempts: 5,
            initial_backoff_ms: 1_000,
            max_backoff_ms: 32_000,
            parent_api_url: String::new(),
            hmac_secret: String::new(),
            http_timeout_secs: 30,
        }
    }
}

/// ディスパッチ処理の結果サマリ。
#[derive(Debug, Default)]
pub struct DispatchResult {
    /// 配信成功件数
    pub sent_count: u32,
    /// 配信失敗・リトライ予約件数
    pub failed_count: u32,
    /// DLQ 移行件数
    pub dlq_count: u32,
}

/// Outbox Consumer（BAT-002）。
///
/// `run()` を `tokio::spawn` で起動して常駐タスクとして動作させる。
/// `wnav_terminal_api` の `main.rs` のみが呼び出す（`wnav_master_api` は使用しない）。
pub struct OutboxConsumer {
    /// PostgreSQL 接続プール（app_event_insert ロール）
    pub pool: Arc<PgPool>,
    /// Webhook 配信サービス（HMAC-SHA256 署名付き HTTP POST）
    pub webhook_sender: Arc<WebhookSender>,
    /// ポーリング間隔（ミリ秒）
    pub interval_ms: u64,
    /// 最大リトライ回数
    pub retry_max: u32,
    /// バックオフ最大値（ミリ秒）
    pub backoff_max_ms: u64,
    /// バックオフ初期値（ミリ秒）
    pub initial_backoff_ms: u64,
    /// バッチサイズ
    pub batch_size: i64,
    /// 親機 API エンドポイント URL
    pub parent_api_url: String,
    /// HMAC 秘密鍵文字列
    pub hmac_secret: String,
}

impl OutboxConsumer {
    /// OutboxConsumer を生成する。
    ///
    /// # 引数
    /// - `pool`: PostgreSQL 接続プール
    /// - `sender`: Webhook 配信サービス
    /// - `interval_ms`: ポーリング間隔（ミリ秒）
    /// - `retry_max`: 最大リトライ回数
    /// - `backoff_max_sec`: バックオフ最大値（秒）
    pub fn new(
        pool: sqlx::PgPool,
        sender: Arc<WebhookSender>,
        interval_ms: u64,
        retry_max: u32,
        backoff_max_sec: u64,
    ) -> Self {
        Self {
            pool: Arc::new(pool),
            webhook_sender: sender,
            interval_ms,
            retry_max,
            backoff_max_ms: backoff_max_sec.saturating_mul(1_000),
            initial_backoff_ms: 1_000,
            batch_size: 10,
            parent_api_url: String::new(),
            hmac_secret: String::new(),
        }
    }

    /// 設定から OutboxConsumer を生成するファクトリメソッド。
    pub fn from_config(pool: sqlx::PgPool, config: OutboxConfig) -> Result<Self, OutboxError> {
        let sender_config = wnav_webhook::sender::WebhookSenderConfig {
            hmac_secret: config.hmac_secret.clone(),
            timeout_ms: config.http_timeout_secs.saturating_mul(1_000),
        };

        let sender = Arc::new(
            WebhookSender::new(sender_config)
                .map_err(|e| OutboxError::InvalidConfig(e.to_string()))?,
        );

        Ok(Self {
            pool: Arc::new(pool),
            webhook_sender: sender,
            interval_ms: config.poll_interval_ms,
            retry_max: config.max_retry_attempts,
            backoff_max_ms: config.max_backoff_ms,
            initial_backoff_ms: config.initial_backoff_ms,
            batch_size: config.batch_size,
            parent_api_url: config.parent_api_url,
            hmac_secret: config.hmac_secret,
        })
    }

    /// BAT-002: 常駐 tokio task として起動する。
    ///
    /// shutdown シグナル受信時に安全に終了する。
    /// wnav_terminal_api の main.rs で `tokio::spawn(consumer.run(shutdown_rx))` として起動する。
    #[tracing::instrument(skip(self, shutdown))]
    pub async fn run(self, mut shutdown: tokio::sync::broadcast::Receiver<()>) {
        tracing::info!(
            log_id = "LOG-BAT-002",
            event = "outbox_consumer.started",
            interval_ms = self.interval_ms,
            retry_max = self.retry_max,
        );

        loop {
            // shutdown シグナルを非同期に確認する（tokio::select! は macros フィーチャーが必要）
            tokio::select! {
                _ = shutdown.recv() => {
                    tracing::info!(
                        log_id = "LOG-BAT-002",
                        event = "outbox_consumer.shutdown",
                        "Outbox Consumer received shutdown signal, stopping gracefully",
                    );
                    break;
                }
                _ = tokio::time::sleep(tokio::time::Duration::from_millis(self.interval_ms)) => {
                    // ポーリング間隔後にバッチ処理を実行する
                }
            }

            // PENDING イベントをバッチ処理する
            match self.process_batch().await {
                Ok(result) => {
                    tracing::info!(
                        log_id = "LOG-BAT-002",
                        event = "outbox.dispatch_cycle",
                        sent = result.sent_count,
                        failed = result.failed_count,
                        dlq = result.dlq_count,
                    );
                }
                Err(e) => {
                    tracing::error!(
                        log_id = "LOG-ERR-003",
                        event = "outbox.dispatch_error",
                        error = %e,
                    );
                }
            }
        }
    }

    /// PENDING 状態の OutboxEvent を最大 batch_size 件取得してディスパッチする。
    ///
    /// アルゴリズム（ALG-009 / ALG-010）:
    /// 1. SELECT ... FOR UPDATE SKIP LOCKED で競合なく最大 batch_size 件取得
    /// 2. 各イベントを親機 API に HMAC-SHA256 署名付き POST
    /// 3. 2xx: status='sent' に UPDATE
    /// 4. 4xx 非リトライ可能: status='dead_lettered' に UPDATE（ERR-EXT-001）
    /// 5. 5xx / ネットワークエラー: retry_count++ / backoff 計算
    /// 6. retry_count >= retry_max: status='dead_lettered' に UPDATE
    ///
    /// NOTE: sqlx::query_as は動的クエリ形式（コンパイル時 DB 検証なし）を使用している。
    ///       DB が利用可能になったら cargo sqlx prepare でキャッシュを生成し、
    ///       sqlx::query! マクロに切り替えること（src/backend/CLAUDE.md 参照）。
    #[tracing::instrument(skip(self), err)]
    pub async fn process_batch(&self) -> Result<DispatchResult, OutboxError> {
        let mut result = DispatchResult::default();

        // PENDING かつ next_retry_at が現在以前の行を SKIP LOCKED で取得する
        // 悲観的ロックにより複数インスタンス間の二重配信を防止する
        let rows = sqlx::query_as::<_, (Uuid, String, serde_json::Value, i16)>(
            r#"
            SELECT
                outbox_id,
                event_type,
                payload,
                retry_count
            FROM outbox_events
            WHERE status = 'PENDING'
              AND (next_retry_at IS NULL OR next_retry_at <= NOW())
            ORDER BY created_at ASC
            LIMIT $1
            FOR UPDATE SKIP LOCKED
            "#,
        )
        .bind(self.batch_size)
        .fetch_all(self.pool.as_ref())
        .await
        .map_err(OutboxError::Database)?;

        for (outbox_id, event_type, payload, retry_count) in rows {
            // ペイロードを文字列にシリアライズする
            let payload_str = payload.to_string();
            let payload_bytes = payload_str.as_bytes();

            // HMAC-SHA256 署名を計算する（"sha256={hex}" 形式）
            let _signature = sign_payload(payload_bytes, &self.hmac_secret);

            // 親機 API に JSON ペイロードを POST する
            let post_result = self
                .post_to_parent(&payload, &event_type, outbox_id)
                .await;

            match post_result {
                Ok(()) => {
                    // 配信成功: status を 'sent' に更新する
                    sqlx::query(
                        r#"
                        UPDATE outbox_events
                        SET status = 'sent',
                            sent_at = NOW(),
                            updated_at = NOW()
                        WHERE outbox_id = $1
                        "#,
                    )
                    .bind(outbox_id)
                    .execute(self.pool.as_ref())
                    .await
                    .map_err(OutboxError::Database)?;

                    tracing::info!(
                        log_id = "LOG-BAT-002",
                        event = "outbox.sent",
                        outbox_id = %outbox_id,
                        event_type = %event_type,
                    );
                    result.sent_count += 1;
                }
                Err(e) => {
                    let new_retry_count = i32::from(retry_count) + 1;
                    let error_msg = e.to_string();

                    // 非リトライ可能エラー（4xx）または最大リトライ超過の場合 DLQ に移行する
                    let is_non_retryable =
                        matches!(&e, OutboxError::HttpStatus(s) if (400u16..500u16).contains(s));
                    let is_max_retries = new_retry_count >= self.retry_max as i32;

                    if is_non_retryable || is_max_retries {
                        // DLQ 移行: status を 'dead_lettered' に更新する（ERR-EXT-001）
                        sqlx::query(
                            r#"
                            UPDATE outbox_events
                            SET status = 'dead_lettered',
                                retry_count = $2,
                                last_error = $3,
                                updated_at = NOW()
                            WHERE outbox_id = $1
                            "#,
                        )
                        .bind(outbox_id)
                        .bind(new_retry_count)
                        .bind(&error_msg)
                        .execute(self.pool.as_ref())
                        .await
                        .map_err(OutboxError::Database)?;

                        tracing::error!(
                            log_id = "LOG-DLQ-001",
                            event = "outbox.dlq.moved",
                            outbox_id = %outbox_id,
                            event_type = %event_type,
                            retry_count = new_retry_count,
                            error = %error_msg,
                        );
                        result.dlq_count += 1;
                    } else {
                        // リトライスケジュール: 指数バックオフで next_retry_at を更新する
                        let backoff_ms = self.compute_backoff_ms(new_retry_count as u32);
                        let next_retry_at =
                            Utc::now() + chrono::Duration::milliseconds(backoff_ms as i64);

                        sqlx::query(
                            r#"
                            UPDATE outbox_events
                            SET status = 'PENDING',
                                retry_count = $2,
                                next_retry_at = $3,
                                last_error = $4,
                                updated_at = NOW()
                            WHERE outbox_id = $1
                            "#,
                        )
                        .bind(outbox_id)
                        .bind(new_retry_count)
                        .bind(next_retry_at)
                        .bind(&error_msg)
                        .execute(self.pool.as_ref())
                        .await
                        .map_err(OutboxError::Database)?;

                        tracing::warn!(
                            log_id = "LOG-BAT-002",
                            event = "outbox.retry_scheduled",
                            outbox_id = %outbox_id,
                            retry_count = new_retry_count,
                            backoff_ms = backoff_ms,
                            error = %error_msg,
                        );
                        result.failed_count += 1;
                    }
                }
            }
        }

        Ok(result)
    }

    /// 指数バックオフ計算（ALG-009）。
    ///
    /// delay_ms = MIN(initial_backoff_ms × 2^(retry_count-1), max_backoff_ms)
    /// デフォルト遅延: 1000 / 2000 / 4000 / 8000 / 16000 ms（上限 32000 ms）
    fn compute_backoff_ms(&self, retry_count: u32) -> u64 {
        // 1-based の retry_count を 0-based の指数に変換する
        let exponent = retry_count.saturating_sub(1);
        let delay = self
            .initial_backoff_ms
            .saturating_mul(2u64.saturating_pow(exponent));
        // 上限を max_backoff_ms でキャップする
        delay.min(self.backoff_max_ms)
    }

    /// Webhook 配信サービス経由で親機 API に POST する。
    async fn post_to_parent(
        &self,
        payload: &serde_json::Value,
        event_type: &str,
        idempotency_key: Uuid,
    ) -> Result<(), OutboxError> {
        self.webhook_sender
            .send(
                &self.parent_api_url,
                event_type,
                payload,
                idempotency_key,
            )
            .await
            .map_err(|e| match e {
                wnav_webhook::WebhookError::HttpStatus { status } => {
                    OutboxError::HttpStatus(status)
                }
                wnav_webhook::WebhookError::HttpError(msg) => OutboxError::Http(msg),
                other => OutboxError::WebhookFailed(other.to_string()),
            })
    }
}

/// BAT-002: Outbox Consumer を常駐ループで起動する（関数形式 API）。
///
/// `wnav_terminal_api` の `main.rs` で
/// `tokio::spawn(run_consumer(Arc::new(consumer), shutdown_rx))` として起動する。
/// `wnav_master_api` はこの関数を呼び出さない。
#[tracing::instrument(skip(consumer, shutdown))]
pub async fn run_consumer(
    consumer: Arc<OutboxConsumer>,
    mut shutdown: tokio::sync::broadcast::Receiver<()>,
) {
    tracing::info!(
        log_id = "LOG-BAT-002",
        event = "outbox_consumer.started",
    );

    loop {
        // shutdown シグナルを確認してから処理する
        tokio::select! {
            _ = shutdown.recv() => {
                tracing::info!(
                    log_id = "LOG-BAT-002",
                    event = "outbox_consumer.shutdown",
                );
                break;
            }
            _ = tokio::time::sleep(tokio::time::Duration::from_millis(consumer.interval_ms)) => {}
        }

        // バッチ処理を実行する
        match consumer.process_batch().await {
            Ok(result) => {
                tracing::info!(
                    log_id = "LOG-BAT-002",
                    event = "outbox.dispatch_cycle",
                    sent = result.sent_count,
                    failed = result.failed_count,
                    dlq = result.dlq_count,
                );
            }
            Err(e) => {
                tracing::error!(
                    log_id = "LOG-ERR-003",
                    event = "outbox.dispatch_error",
                    error = %e,
                );
            }
        }
    }
}
