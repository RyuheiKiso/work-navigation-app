# 07 wnav_webhook 詳細設計（MOD-BE-007）

> **配置**: 本クレートは **`wnav_terminal_api`（ポート 8080）バイナリが担当する crate** である。
> Webhook は端末から親機へ実績を送信する機能であり、`wnav_outbox`（MOD-BE-006）と同じ文脈で `wnav_terminal_api` に属する。
> Webhook 配信タスク（BAT-008 webhook_retry）は `wnav_terminal_api` 内の tokio task として動作する。
> `wnav_master_api` は本クレートを直接使用しない。

本章は `crates/wnav_webhook/` の Webhook 配信サービス・HMAC-SHA256 ペイロード署名・配信リトライ・受信側での署名検証方法の詳細設計を確定する。本クレートは IF-002（外部親機 Webhook 通知）を実現する。

---

## 1. WebhookDeliveryService 構造体

```rust
// crates/wnav_webhook/src/service.rs

use sqlx::PgPool;
use reqwest::Client;
use std::sync::Arc;

/// Webhook 配信サービス。
/// Outbox Consumer（MOD-BE-006）から呼び出されて HTTP POST を実行する。
pub struct WebhookDeliveryService {
    db: Arc<PgPool>,
    http_client: Client,
    config: WebhookConfig,
}

/// Webhook 配信設定
#[derive(Debug, Clone, serde::Deserialize)]
pub struct WebhookConfig {
    /// HMAC-SHA256 署名秘密鍵（hex エンコード 256bit 以上推奨）
    pub signing_secret_hex: String,
    /// 配信先エンドポイント URL（親機 Inbound Webhook）
    pub endpoint_url: String,
    /// HTTP タイムアウト秒（デフォルト 30）
    pub timeout_secs: u64,
    /// 最大リトライ回数（デフォルト 3）
    pub max_retries: u8,
}

/// 配信試行の結果
#[derive(Debug)]
pub struct DeliveryAttemptResult {
    pub success: bool,
    pub http_status: Option<u16>,
    pub error_message: Option<String>,
    pub delivered_at: Option<chrono::DateTime<chrono::Utc>>,
}
```

---

## 2. HMAC-SHA256 ペイロード署名

```rust
// crates/wnav_webhook/src/sign.rs

use hmac::{Hmac, Mac};
use sha2::Sha256;

/// (FNC-BE-013) ペイロードを HMAC-SHA256 で署名して hex 文字列を返す。
///
/// 署名計算: HMAC-SHA256(secret_bytes, payload_bytes)
/// ヘッダ形式: X-WNav-Signature: sha256={hex_digest}
///
/// 受信側での検証手順:
/// 1. X-WNav-Signature ヘッダから hex_digest を取り出す
/// 2. 同じ secret で HMAC-SHA256(secret, received_body) を計算
/// 3. 定数時間比較（`subtle::ConstantTimeEq`）で一致を確認
///    （タイミング攻撃防止のため == 演算子を使わない）
pub fn sign_payload(secret: &[u8], payload: &[u8]) -> String {
    type HmacSha256 = Hmac<Sha256>;

    let mut mac = HmacSha256::new_from_slice(secret)
        .expect("HMAC can take key of any size");
    mac.update(payload);
    let result = mac.finalize();
    hex::encode(result.into_bytes())
}

/// 受信側での署名検証（定数時間比較）
pub fn verify_signature(
    secret: &[u8],
    payload: &[u8],
    expected_hex: &str,
) -> bool {
    use subtle::ConstantTimeEq;

    let computed = sign_payload(secret, payload);
    let computed_bytes = computed.as_bytes();
    let expected_bytes = expected_hex.as_bytes();

    if computed_bytes.len() != expected_bytes.len() {
        return false;
    }

    computed_bytes.ct_eq(expected_bytes).into()
}
```

---

## 3. Webhook 配信実装

```rust
// crates/wnav_webhook/src/service.rs（続き）

use crate::{WebhookDeliveryService, DeliveryAttemptResult};
use chrono::Utc;
use uuid::Uuid;

impl WebhookDeliveryService {
    pub fn new(db: Arc<PgPool>, config: WebhookConfig) -> Self {
        let http_client = reqwest::ClientBuilder::new()
            .timeout(std::time::Duration::from_secs(config.timeout_secs))
            .build()
            .expect("HTTP client build failed");
        Self { db, http_client, config }
    }

    /// Webhook エンドポイントにペイロードを配信する。
    /// 配信試行の結果を返す（成否問わず DB に記録するために呼び出し元で処理）
    pub async fn deliver(
        &self,
        webhook_event_id: Uuid,
        payload: &[u8],
        event_type: &str,
    ) -> DeliveryAttemptResult {
        let secret = match hex::decode(&self.config.signing_secret_hex) {
            Ok(s) => s,
            Err(_) => {
                return DeliveryAttemptResult {
                    success: false,
                    http_status: None,
                    error_message: Some("Invalid signing_secret_hex config".to_string()),
                    delivered_at: None,
                };
            }
        };

        let signature = crate::sign::sign_payload(&secret, payload);

        let response = self.http_client
            .post(&self.config.endpoint_url)
            .header("Content-Type", "application/json")
            .header("X-WNav-Signature", format!("sha256={}", signature))
            .header("X-WNav-Event", event_type)
            .header("X-WNav-Delivery-Id", webhook_event_id.to_string())
            .header("Idempotency-Key", webhook_event_id.to_string())
            .body(payload.to_vec())
            .send()
            .await;

        match response {
            Ok(resp) => {
                let status = resp.status().as_u16();
                if resp.status().is_success() {
                    tracing::info!(
                        log_id = "LOG-WH-001",
                        event_name = "webhook.delivered",
                        webhook_event_id = %webhook_event_id,
                        status = status,
                    );
                    DeliveryAttemptResult {
                        success: true,
                        http_status: Some(status),
                        error_message: None,
                        delivered_at: Some(Utc::now()),
                    }
                } else {
                    tracing::warn!(
                        log_id = "LOG-WH-002",
                        event_name = "webhook.delivery_failed",
                        webhook_event_id = %webhook_event_id,
                        status = status,
                    );
                    DeliveryAttemptResult {
                        success: false,
                        http_status: Some(status),
                        error_message: Some(format!("HTTP {}", status)),
                        delivered_at: None,
                    }
                }
            }
            Err(e) => {
                tracing::error!(
                    log_id = "LOG-WH-003",
                    event_name = "webhook.delivery_error",
                    webhook_event_id = %webhook_event_id,
                    error = %e,
                );
                DeliveryAttemptResult {
                    success: false,
                    http_status: None,
                    error_message: Some(e.to_string()),
                    delivered_at: None,
                }
            }
        }
    }
}
```

---

## 4. Webhook イベント種別

| イベント種別（X-WNav-Event ヘッダ）| 説明 | 対応 MSG-ID |
|---|---|---|
| `work_event.step_completed` | Step 完了イベント | MSG-001 |
| `work_event.work_completed` | 作業完了イベント | MSG-001 |
| `electronic_sign.recorded` | 電子サイン記録 | MSG-002 |
| `audit.alert_triggered` | アンドン発報 | MSG-003 |

---

## 5. リトライ設計（BAT-008 との連携）

Webhook 配信のリトライは `wnav_outbox` の Outbox Consumer（MOD-BE-006）が担当し、本クレートは純粋に 1 回の HTTP POST と署名生成を担う。リトライロジックを本クレートに持たせないことで、単一責任の原則を維持する。

BAT-008（webhook_retry）は `wnav_terminal_api` 内の tokio task として動作し、`wnav_outbox` の `run_consumer` と連携する。

```
[wnav_terminal_api] main.rs
    ├── tokio::spawn(wnav_outbox::run_consumer(...))   # BAT-002
    │       ↓ 配信先が Webhook の場合
    │   [wnav_webhook] WebhookDeliveryService.deliver()   # BAT-008 webhook_retry
    │       ↓ 失敗
    │   [wnav_outbox] retry_count++ または DEAD_LETTERED 移行
    └── ...
```

---

**本節で確定した方針**
- **HMAC-SHA256 署名は `hmac` crate（RustCrypto）で計算し、`X-WNav-Signature: sha256={hex}` ヘッダで送信する設計を確定した。**
- **受信側での署名検証は `subtle::ConstantTimeEq` による定数時間比較で実施し、タイミング攻撃を防ぐ設計を確定した。**
- **リトライロジックは本クレートに持たせず `wnav_outbox` に委譲することで単一責任原則を維持する設計を確定した。**
- **本クレートは `wnav_terminal_api` が担当し、BAT-008（webhook_retry）は `wnav_terminal_api` 内の tokio task として動作することを確定した。`wnav_master_api` は本クレートを使用しない。**

---

## 参照業界分析

### 必須
- [`90_業界分析/09_セキュリティとアクセス制御.md`](../../90_業界分析/09_セキュリティとアクセス制御.md)

### 関連
- [`90_業界分析/06_品質管理とトレーサビリティ.md`](../../90_業界分析/06_品質管理とトレーサビリティ.md)
