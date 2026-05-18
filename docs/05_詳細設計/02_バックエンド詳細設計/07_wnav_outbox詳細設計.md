# 06 wnav_outbox 詳細設計（MOD-BE-006）

> **配置**: 本クレートは **`wnav_terminal_api`（ポート 8080）バイナリのみが依存する crate** である。
> Outbox は端末で記録した作業実績を親機へ非同期送信するための仕組みであり、`wnav_master_api` は本クレートに依存しない。
> `OutboxConsumer` の常駐タスク（BAT-002）は `wnav_terminal_api` の `main.rs` 内で `tokio::spawn` される。
> `parent_api_url` は環境変数 `WNAV_TERMINAL_PARENT_API_URL` から取得する。

本章は `crates/wnav_outbox/` の Outbox Consumer 実装・指数バックオフ・DLQ 移行・HMAC-SHA256 署名付き HTTP ディスパッチの詳細設計を確定する。本クレートは BAT-002（常駐 Outbox Consumer）を実装し、FR-SY-002/005 を直接実現する。

---

## 1. OutboxConsumer 構造体と設定

```rust
// crates/wnav_outbox/src/consumer.rs

use std::sync::Arc;
use sqlx::PgPool;
use reqwest::Client;

/// Outbox Consumer（BAT-002）。
/// `run_consumer` として tokio::spawn で起動する。
pub struct OutboxConsumer {
    db: Arc<PgPool>,
    http_client: Client,
    config: OutboxConfig,
}

/// Outbox Consumer の設定。CFG-003〜005 に対応する。
#[derive(Debug, Clone, serde::Deserialize)]
pub struct OutboxConfig {
    /// ポーリング間隔（ミリ秒、CFG-003、デフォルト 60000）
    pub poll_interval_ms: u64,
    /// 1 回のポーリングで処理するバッチサイズ（デフォルト 100）
    pub batch_size: u32,
    /// 最大リトライ回数（CFG-004、デフォルト 5）
    pub max_retry_attempts: u8,
    /// バックオフ初期値（ミリ秒、デフォルト 1000）
    pub initial_backoff_ms: u64,
    /// バックオフ最大値（ミリ秒、デフォルト 32000）
    pub max_backoff_ms: u64,
    /// DLQ 移行閾値（CFG-005、max_retry_attempts と同一値）
    pub dlq_threshold: u8,
    /// 親機 API エンドポイント URL（IF-002）。
    /// wnav_terminal_api の環境変数 `WNAV_TERMINAL_PARENT_API_URL` から取得する。
    pub parent_api_url: String,
    /// HMAC 署名秘密鍵（hex エンコード済み）
    pub hmac_secret_hex: String,
    /// HTTP タイムアウト秒（デフォルト 30）
    pub http_timeout_secs: u64,
}

impl Default for OutboxConfig {
    fn default() -> Self {
        Self {
            poll_interval_ms: 60_000,
            batch_size: 100,
            max_retry_attempts: 5,
            initial_backoff_ms: 1_000,
            max_backoff_ms: 32_000,
            dlq_threshold: 5,
            parent_api_url: String::new(),
            hmac_secret_hex: String::new(),
            http_timeout_secs: 30,
        }
    }
}

/// Outbox ディスパッチの結果サマリ
#[derive(Debug)]
pub struct DispatchResult {
    pub sent_count: u32,
    pub failed_count: u32,
    pub dlq_count: u32,
}
```

---

## 2. dispatch_pending アルゴリズム

```rust
// crates/wnav_outbox/src/consumer.rs（続き）

use crate::{OutboxConsumer, OutboxConfig, DispatchResult, OutboxError};
use chrono::Utc;
use uuid::Uuid;

impl OutboxConsumer {
    pub fn new(db: Arc<PgPool>, config: OutboxConfig) -> Self {
        let http_client = reqwest::ClientBuilder::new()
            .timeout(std::time::Duration::from_secs(config.http_timeout_secs))
            .build()
            .expect("HTTP client build failed");
        Self { db, http_client, config }
    }

    /// (FNC-BE-012) PENDING 状態の OutboxEvent を取得してディスパッチする。
    ///
    /// アルゴリズム:
    /// 1. SELECT ... FOR UPDATE SKIP LOCKED で競合なく取得
    /// 2. status を SENDING に更新（楽観ロック）
    /// 3. 親機 API に HMAC-SHA256 署名付き POST
    /// 4. 2xx: status = SENT, sent_at = NOW()
    /// 5. 非 2xx: retry_count++ と next_retry_at を更新
    /// 6. retry_count >= dlq_threshold: status = DEAD_LETTERED
    pub async fn dispatch_pending(&self) -> Result<DispatchResult, OutboxError> {
        let mut result = DispatchResult {
            sent_count: 0,
            failed_count: 0,
            dlq_count: 0,
        };

        // 1. PENDING かつ next_retry_at が現在以前の行を SKIP LOCKED で取得
        let rows = sqlx::query!(
            r#"
            SELECT outbox_id, event_type, event_id, payload, retry_count
            FROM outbox_events
            WHERE status = 'PENDING'
              AND (next_retry_at IS NULL OR next_retry_at <= NOW())
            ORDER BY created_at ASC
            LIMIT $1
            FOR UPDATE SKIP LOCKED
            "#,
            self.config.batch_size as i64,
        )
        .fetch_all(self.db.as_ref())
        .await
        .map_err(OutboxError::Db)?;

        for row in rows {
            let outbox_id: Uuid = row.outbox_id;
            let retry_count: i16 = row.retry_count;
            let payload_json = row.payload.to_string();

            // 2. SENDING に更新
            sqlx::query!(
                "UPDATE outbox_events SET status = 'SENDING' WHERE outbox_id = $1",
                outbox_id
            )
            .execute(self.db.as_ref())
            .await
            .map_err(OutboxError::Db)?;

            // 3. 親機 API に POST（HMAC-SHA256 署名付き）
            match self.post_to_parent(&payload_json).await {
                Ok(_) => {
                    // 4. 成功: SENT に更新
                    sqlx::query!(
                        "UPDATE outbox_events SET status = 'SENT', sent_at = NOW() WHERE outbox_id = $1",
                        outbox_id
                    )
                    .execute(self.db.as_ref())
                    .await
                    .map_err(OutboxError::Db)?;

                    tracing::info!(
                        log_id = "LOG-BAT-002",
                        event_name = "outbox.sent",
                        outbox_id = %outbox_id,
                    );
                    result.sent_count += 1;
                }
                Err(e) => {
                    let new_retry_count = retry_count + 1;

                    if new_retry_count as u8 >= self.config.dlq_threshold {
                        // 6. DLQ 移行
                        sqlx::query!(
                            r#"
                            UPDATE outbox_events
                            SET status = 'DEAD_LETTERED',
                                retry_count = $2,
                                last_error = $3
                            WHERE outbox_id = $1
                            "#,
                            outbox_id,
                            new_retry_count,
                            e.to_string(),
                        )
                        .execute(self.db.as_ref())
                        .await
                        .map_err(OutboxError::Db)?;

                        tracing::error!(
                            log_id = "LOG-DLQ-001",
                            event_name = "outbox.dlq.moved",
                            outbox_id = %outbox_id,
                            error = %e,
                        );
                        result.dlq_count += 1;
                    } else {
                        // 5. リトライスケジュール
                        let backoff = self.backoff_duration(new_retry_count as u8);
                        let next_retry = Utc::now() + backoff;

                        sqlx::query!(
                            r#"
                            UPDATE outbox_events
                            SET status = 'PENDING',
                                retry_count = $2,
                                next_retry_at = $3,
                                last_error = $4
                            WHERE outbox_id = $1
                            "#,
                            outbox_id,
                            new_retry_count,
                            next_retry,
                            e.to_string(),
                        )
                        .execute(self.db.as_ref())
                        .await
                        .map_err(OutboxError::Db)?;

                        result.failed_count += 1;
                    }
                }
            }
        }

        Ok(result)
    }

    /// 指数バックオフ計算。
    /// backoff = min(initial_backoff_ms * 2^retry_count, max_backoff_ms)
    fn backoff_duration(&self, retry_count: u8) -> chrono::Duration {
        let ms = self.config.initial_backoff_ms
            .saturating_mul(2u64.saturating_pow(retry_count as u32))
            .min(self.config.max_backoff_ms);
        chrono::Duration::milliseconds(ms as i64)
    }

    /// HMAC-SHA256 署名付きで親機 API に POST する。
    async fn post_to_parent(&self, payload: &str) -> Result<(), OutboxError> {
        let signature = crate::sign::sign_payload(
            &hex::decode(&self.config.hmac_secret_hex)
                .map_err(|_| OutboxError::InvalidConfig("hmac_secret_hex is not valid hex".to_string()))?,
            payload.as_bytes(),
        );

        let response = self.http_client
            .post(&self.config.parent_api_url)
            .header("Content-Type", "application/json")
            .header("X-WNav-Signature", format!("sha256={}", signature))
            .header("Idempotency-Key", Uuid::now_v7().to_string())
            .body(payload.to_string())
            .send()
            .await
            .map_err(|e| OutboxError::Http(e.to_string()))?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(OutboxError::HttpStatus(response.status().as_u16()))
        }
    }
}
```

---

## 3. Consumer 常駐ループ（BAT-002）

```rust
// crates/wnav_outbox/src/lib.rs

use std::sync::Arc;
use tokio::time::{sleep, Duration};

/// BAT-002: Outbox Consumer を常駐ループで起動する。
/// `wnav_terminal_api` の `main.rs` で
/// `tokio::spawn(run_consumer(consumer))` として非同期タスクを起動する。
/// `wnav_master_api` はこの関数を呼び出さない。
pub async fn run_consumer(consumer: Arc<OutboxConsumer>) {
    tracing::info!(
        log_id = "LOG-BAT-002",
        event_name = "outbox_consumer.started",
    );

    loop {
        match consumer.dispatch_pending().await {
            Ok(result) => {
                tracing::info!(
                    log_id = "LOG-BAT-002",
                    event_name = "outbox.dispatch_cycle",
                    sent = result.sent_count,
                    failed = result.failed_count,
                    dlq = result.dlq_count,
                );
            }
            Err(e) => {
                tracing::error!(
                    log_id = "LOG-ERR-003",
                    event_name = "outbox.dispatch_error",
                    error = %e,
                );
            }
        }

        sleep(Duration::from_millis(consumer.config.poll_interval_ms)).await;
    }
}
```

---

## 4. エラー型

```rust
// crates/wnav_outbox/src/error.rs

#[derive(Debug, thiserror::Error)]
pub enum OutboxError {
    #[error("Database error: {0}")]
    Db(#[from] sqlx::Error),
    #[error("HTTP error: {0}")]
    Http(String),
    #[error("HTTP status error: {0}")]
    HttpStatus(u16),
    #[error("Invalid config: {0}")]
    InvalidConfig(String),
}
```

---

## 5. アウトボックスイベントのステータス遷移

| 遷移 | 条件 |
|---|---|
| PENDING → SENDING | Consumer がポーリング取得した時点 |
| SENDING → SENT | 親機 API が 2xx を返した時点 |
| SENDING → PENDING | 親機 API が非 2xx かつ retry_count < dlq_threshold |
| SENDING → DEAD_LETTERED | retry_count >= dlq_threshold |

DEAD_LETTERED に移行したイベントは API-ops-002（`POST /ops/outbox/{id}/requeue`）で手動再投入できる。再投入時は status を PENDING に戻し retry_count を 0 にリセットする。

---

**本節で確定した方針**
- **`SELECT ... FOR UPDATE SKIP LOCKED` でポーリング競合を排除し、複数インスタンスが並行して安全に Outbox を処理できる設計を確定した。**
- **指数バックオフは `initial_backoff_ms * 2^retry_count` で計算し、`max_backoff_ms`（32 秒）を上限として短時間での過負荷を防ぐ設計を確定した。**
- **DLQ 移行後は管理コンソール（SCR-MC-007）への表示と LOG-DLQ-001 ログ出力を行い、IT 担当者が手動で再投入できる運用設計を確定した。**
- **本クレートは `wnav_terminal_api` のみが依存し、`wnav_master_api` は依存しない。`parent_api_url` は環境変数 `WNAV_TERMINAL_PARENT_API_URL` から取得する設計を確定した。**
- **BAT-002（常駐 Outbox Consumer）は terminal-api バイナリ（`wnav_terminal_api`）の `main.rs` で `tokio::spawn` して起動することを確定した。**

---

## 参照業界分析

### 必須
- [`90_業界分析/09_セキュリティとアクセス制御.md`](../../90_業界分析/09_セキュリティとアクセス制御.md)

### 関連
- [`90_業界分析/06_品質管理とトレーサビリティ.md`](../../90_業界分析/06_品質管理とトレーサビリティ.md)
