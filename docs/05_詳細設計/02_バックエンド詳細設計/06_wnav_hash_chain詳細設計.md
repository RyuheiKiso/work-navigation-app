# 05 wnav_hash_chain 詳細設計（MOD-BE-003）

> **配置**: 本クレートは **`wnav_master_api`（ポート 8081）バイナリのみが依存する crate** である。
> ハッシュチェーン検証は製造記録の改ざん検知という管理操作であり、`wnav_terminal_api` は本クレートに依存しない。
> `HashChainService` の常駐タスク（BAT-001）および週次検証バッチは `wnav_master_api` の `main.rs` 内で `tokio::spawn` される。

本章は `crates/wnav_hash_chain/` の SHA-256 ハッシュチェーン計算アルゴリズム・genesis ブロック定義・週次検証アルゴリズム（BAT-001）の詳細設計を確定する。本クレートは製造記録の改ざん検知を担保する核心的な機能であり、FR-EV-001/002 を直接実現する。

---

## 1. HashChainService 構造体

```rust
// crates/wnav_hash_chain/src/lib.rs

use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

/// ハッシュチェーン計算・検証サービス。
/// `wnav_db` の PgPool を保持し、週次検証時に直接クエリを実行する。
pub struct HashChainService {
    pool: Arc<PgPool>,
}

impl HashChainService {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

/// ハッシュチェーン週次検証の結果
#[derive(Debug, serde::Serialize)]
pub struct VerificationResult {
    /// 検証が合格したかどうか
    pub is_valid: bool,
    /// 最初に不整合が見つかったブロック ID（合格時は None）
    pub broken_at_block_id: Option<Uuid>,
    /// 検証したブロック数
    pub checked_count: u64,
    /// 検証開始日時（UTC）
    pub verified_at: chrono::DateTime<chrono::Utc>,
}
```

---

## 2. コンテンツハッシュ計算

```rust
// crates/wnav_hash_chain/src/hash.rs

use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

/// (FNC-BE-009) WorkEvent のコンテンツハッシュを計算する。
///
/// 入力: イベント固有フィールドを BTreeMap でアルファベット昇順にソートした
/// canonical JSON 文字列（JSON キーの順序を決定論的に統一するため BTreeMap を使用）
///
/// 出力: 32 バイト（SHA-256 ダイジェスト）
pub fn compute_content_hash(event: &WorkEventRecord) -> [u8; 32] {
    let mut fields = BTreeMap::new();
    fields.insert("activity",          event.activity.clone());
    fields.insert("case_id",           event.case_id.to_string());
    fields.insert("event_id",          event.event_id.to_string());
    fields.insert("payload",           event.payload_canonical.clone());
    fields.insert("resource",          event.resource.to_string());
    fields.insert("sop_version_id",    event.sop_version_id.to_string());
    fields.insert("terminal_id",       event.terminal_id.to_string());
    fields.insert("timestamp_client",  event.timestamp_client.to_rfc3339());
    fields.insert("timestamp_server",  event.timestamp_server.to_rfc3339());

    let canonical = serde_json::to_string(&fields)
        .expect("BTreeMap シリアライズは必ず成功する");

    let mut hasher = Sha256::new();
    hasher.update(canonical.as_bytes());
    hasher.finalize().into()
}

/// (FNC-BE-010) チェーンハッシュを計算する。
///
/// SHA-256(prev_hash_bytes || content_hash_bytes) の連結ハッシュ。
/// prev_hash_bytes: 前イベントの content_hash を 32 バイト に変換
///                  genesis（初回）は [0u8; 32]
pub fn compute_chain_hash(
    prev_hash: &[u8; 32],
    content_hash: &[u8; 32],
) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(prev_hash);
    hasher.update(content_hash);
    hasher.finalize().into()
}

/// hex 文字列を [u8; 32] に変換する。
/// genesis ハッシュ（"0"×64）は [0u8; 32] を返す。
pub fn hex_to_bytes32(hex: &str) -> Result<[u8; 32], HashError> {
    if hex.len() != 64 {
        return Err(HashError::InvalidHexLength(hex.len()));
    }
    let bytes = hex::decode(hex).map_err(|_| HashError::InvalidHex)?;
    bytes.try_into().map_err(|_| HashError::InvalidHex)
}

/// [u8; 32] を lowercase hex 文字列（64 文字）に変換する。
pub fn bytes32_to_hex(bytes: &[u8; 32]) -> String {
    hex::encode(bytes)
}

/// WorkEvent から呼び出し可能なラッパ関数（crates/wnav_db から呼ぶ）
pub fn compute_content_hash_for_event(
    event: &wnav_domain::model::work_event::WorkEvent,
    prev_hash_hex: &str,
) -> String {
    let record = WorkEventRecord {
        event_id: event.event_id,
        case_id: event.case_id,
        activity: event.activity.clone(),
        timestamp_client: event.timestamp_client,
        timestamp_server: event.timestamp_server,
        resource: event.resource,
        sop_version_id: event.sop_version_id,
        terminal_id: event.terminal_id,
        payload_canonical: serde_json::to_string(&event.payload)
            .unwrap_or_default(),
    };

    let content = compute_content_hash(&record);
    let prev = hex_to_bytes32(prev_hash_hex).unwrap_or([0u8; 32]);
    let chain = compute_chain_hash(&prev, &content);

    bytes32_to_hex(&chain)
}

/// ハッシュ計算に使用するイベントフィールドの中間表現
#[derive(Debug)]
pub struct WorkEventRecord {
    pub event_id: uuid::Uuid,
    pub case_id: uuid::Uuid,
    pub activity: String,
    pub timestamp_client: chrono::DateTime<chrono::Utc>,
    pub timestamp_server: chrono::DateTime<chrono::Utc>,
    pub resource: uuid::Uuid,
    pub sop_version_id: uuid::Uuid,
    pub terminal_id: uuid::Uuid,
    pub payload_canonical: String,
}

#[derive(Debug, thiserror::Error)]
pub enum HashError {
    #[error("Invalid hex length: expected 64, got {0}")]
    InvalidHexLength(usize),
    #[error("Invalid hex string")]
    InvalidHex,
}
```

---

## 3. Genesis ブロック定義

```rust
// crates/wnav_hash_chain/src/genesis.rs

/// Genesis ブロック: ハッシュチェーンの先頭に挿入される初期ブロック。
/// prev_hash はゼロ値（[0u8; 32]）を使用する。
/// chain_hash = SHA-256([0u8; 32] || content_hash)
pub const GENESIS_PREV_HASH: [u8; 32] = [0u8; 32];
pub const GENESIS_PREV_HASH_HEX: &str =
    "0000000000000000000000000000000000000000000000000000000000000000";

/// Genesis ブロック作成コマンド。
/// 初回デプロイ時またはファクトリ新規登録時に 1 度だけ実行する。
pub fn create_genesis_block(factory_id: uuid::Uuid) -> GenesisBlock {
    GenesisBlock {
        block_id: uuid::Uuid::now_v7(),
        factory_id,
        prev_hash: GENESIS_PREV_HASH_HEX.to_string(),
        content_hash: hex::encode(sha2::Sha256::digest(
            format!("genesis:{}", factory_id).as_bytes()
        )),
        created_at: chrono::Utc::now(),
    }
}

#[derive(Debug)]
pub struct GenesisBlock {
    pub block_id: uuid::Uuid,
    pub factory_id: uuid::Uuid,
    pub prev_hash: String,
    pub content_hash: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}
```

---

## 4. 週次検証アルゴリズム（BAT-001）

```rust
// crates/wnav_hash_chain/src/verifier.rs

use crate::{HashChainService, VerificationResult};
use crate::hash::{compute_content_hash, compute_chain_hash, hex_to_bytes32, bytes32_to_hex};
use uuid::Uuid;
use sqlx::Row;

impl HashChainService {
    /// (FNC-BE-011) ハッシュチェーンを先頭から末尾まで検証する。
    ///
    /// アルゴリズム:
    /// 1. TBL-001 work_events を event_id 昇順（UUID v7 = 時系列順）で全件ロード
    /// 2. genesis から始めて各イベントの content_hash を再計算
    /// 3. 再計算値が DB 保存値と一致するか比較
    /// 4. 不一致が見つかった時点で broken_at_block_id を記録して打ち切る
    ///
    /// from_block: 検証開始イベント ID（None なら genesis から）
    pub async fn verify_chain(
        &self,
        from_block: Option<Uuid>,
    ) -> Result<VerificationResult, sqlx::Error> {
        let start_time = chrono::Utc::now();

        // ストリーミングクエリで全ブロックを順次処理（メモリ節約）
        let rows = sqlx::query(
            r#"
            SELECT
                event_id,
                case_id,
                activity,
                timestamp_client,
                timestamp_server,
                resource,
                sop_version_id,
                terminal_id,
                payload::text AS payload_canonical,
                prev_hash,
                content_hash
            FROM work_events
            WHERE ($1::uuid IS NULL OR event_id >= $1)
            ORDER BY event_id ASC
            "#,
        )
        .bind(from_block)
        .fetch_all(self.pool.as_ref())
        .await?;

        let mut checked_count: u64 = 0;
        let mut expected_prev_hash = crate::genesis::GENESIS_PREV_HASH;

        for row in &rows {
            let stored_content_hash: String = row.try_get("content_hash").unwrap_or_default();
            let stored_prev_hash: String = row.try_get("prev_hash").unwrap_or_default();
            let event_id: Uuid = row.try_get("event_id").unwrap_or_default();

            // prev_hash の整合性チェック
            if stored_prev_hash != bytes32_to_hex(&expected_prev_hash) {
                return Ok(VerificationResult {
                    is_valid: false,
                    broken_at_block_id: Some(event_id),
                    checked_count,
                    verified_at: start_time,
                });
            }

            // content_hash の再計算と比較
            let record = WorkEventRecord::from_row(row)?;
            let recomputed = compute_content_hash(&record);
            let recomputed_hex = bytes32_to_hex(&recomputed);

            if recomputed_hex != stored_content_hash {
                return Ok(VerificationResult {
                    is_valid: false,
                    broken_at_block_id: Some(event_id),
                    checked_count,
                    verified_at: start_time,
                });
            }

            // 次のブロックの expected_prev_hash を更新
            let prev_bytes = hex_to_bytes32(&stored_prev_hash).unwrap_or(crate::genesis::GENESIS_PREV_HASH);
            expected_prev_hash = compute_chain_hash(&prev_bytes, &recomputed);

            checked_count += 1;
        }

        tracing::info!(
            log_id = "LOG-BAT-001",
            event_name = "hash_chain.verified",
            checked_count = checked_count,
            is_valid = true,
        );

        Ok(VerificationResult {
            is_valid: true,
            broken_at_block_id: None,
            checked_count,
            verified_at: start_time,
        })
    }
}
```

---

## 5. 週次検証スケジューラ（BAT-001 常駐タスク）

> **起動バイナリ**: `run_weekly_verifier` は `wnav_master_api` の `main.rs` で
> `tokio::spawn(run_weekly_verifier(svc.clone()))` として起動する。
> `wnav_terminal_api` はこの関数を呼び出さない。

```rust
// crates/wnav_hash_chain/src/scheduler.rs

use std::sync::Arc;
use crate::HashChainService;

/// BAT-001: 週次ハッシュチェーン検証スケジューラ。
/// tokio-cron-scheduler を使用して月曜 02:00（JST）に実行する。
/// wnav_master_api の main.rs で tokio::spawn して起動する。
pub async fn run_weekly_verifier(svc: Arc<HashChainService>) {
    use tokio_cron_scheduler::{Job, JobScheduler};

    let scheduler = JobScheduler::new().await
        .expect("scheduler init failed");

    let svc_clone = svc.clone();
    scheduler
        .add(Job::new_async("0 0 17 * * 1", move |_uuid, _lock| {
            // UTC 17:00 月曜 = JST 02:00 月曜
            let svc = svc_clone.clone();
            Box::pin(async move {
                match svc.verify_chain(None).await {
                    Ok(result) => {
                        if !result.is_valid {
                            tracing::error!(
                                log_id = "LOG-ERR-001",
                                event_name = "hash_chain.broken",
                                broken_at = ?result.broken_at_block_id,
                                checked_count = result.checked_count,
                            );
                        } else {
                            tracing::info!(
                                log_id = "LOG-BAT-001",
                                event_name = "hash_chain.weekly_ok",
                                checked_count = result.checked_count,
                            );
                        }
                    }
                    Err(e) => {
                        tracing::error!(
                            log_id = "LOG-ERR-002",
                            event_name = "hash_chain.verify_failed",
                            error = %e,
                        );
                    }
                }
            })
        }).expect("job creation failed"))
        .await
        .expect("job add failed");

    scheduler.start().await.expect("scheduler start failed");

    // スケジューラが終了しないよう待機
    tokio::signal::ctrl_c().await.ok();
}
```

---

**本節で確定した方針**
- **compute_content_hash は BTreeMap によるキーのアルファベット昇順ソートで canonical JSON を生成し、JSON キー順序の不確定性を排除する設計を確定した。**
- **compute_chain_hash は SHA-256(prev_hash_bytes || content_hash_bytes) の連結ハッシュとし、チェーン間の依存関係を保証することを確定した。**
- **週次検証（BAT-001 / master-api 内）は event_id（UUID v7 = 時系列）の昇順ストリーミングで全件検証し、不整合を検知した時点で即座に ERR-DB-003 ログを出力することを確定した。**
- **本クレートは `wnav_master_api` のみが依存し、`wnav_terminal_api` は依存しない。ハッシュチェーン検証は管理操作であり、端末 API からは実行しない設計を確定した。**

---

## 参照業界分析

### 必須
- [`90_業界分析/09_セキュリティとアクセス制御.md`](../../90_業界分析/09_セキュリティとアクセス制御.md)

### 関連
- [`90_業界分析/06_品質管理とトレーサビリティ.md`](../../90_業界分析/06_品質管理とトレーサビリティ.md)
