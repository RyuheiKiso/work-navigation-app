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

## 2a. IQC・リワーク用コンテンツハッシュ計算（FNC-BE-018〜020 / ADR-011）

```rust
// crates/wnav_hash_chain/src/iqc_hash.rs

use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

/// (FNC-BE-018) IncomingInspection のコンテンツハッシュを計算する。
///
/// 対象フィールド: inspection_id / lot_id / supplier_id / material_id /
///   sampling_plan_id / sampling_plan_version / lot_quantity / sample_size_n /
///   accept_number_ac / reject_number_re / severity_state / inspector_id / received_at
/// NOTE: qc_status / judged_at は可変フィールドのためハッシュ対象外（ADR-011）。
pub fn compute_content_hash_for_inspection(record: &InspectionRecord) -> [u8; 32] {
    let mut fields = BTreeMap::new();
    fields.insert("accept_number_ac",      record.accept_number_ac.to_string());
    fields.insert("inspector_id",          record.inspector_id.to_string());
    fields.insert("inspection_id",         record.inspection_id.to_string());
    fields.insert("lot_id",                record.lot_id.to_string());
    fields.insert("lot_quantity",          record.lot_quantity.to_string());
    fields.insert("material_id",           record.material_id.to_string());
    fields.insert("received_at",           record.received_at.to_rfc3339());
    fields.insert("reject_number_re",      record.reject_number_re.to_string());
    fields.insert("sample_size_n",         record.sample_size_n.to_string());
    fields.insert("sampling_plan_id",      record.sampling_plan_id.to_string());
    fields.insert("sampling_plan_version", record.sampling_plan_version.to_string());
    fields.insert("severity_state",        record.severity_state.clone());
    fields.insert("supplier_id",           record.supplier_id.to_string());

    let canonical = serde_json::to_string(&fields)
        .expect("BTreeMap シリアライズは必ず成功する");

    let mut hasher = Sha256::new();
    hasher.update(canonical.as_bytes());
    hasher.finalize().into()
}

/// (FNC-BE-019) リワーク系テーブル（rework_verifications / reworked_lot_labels /
///   scrap_records / return_to_vendor_records）のコンテンツハッシュを計算する。
///
/// 各テーブル固有のフィールドを BTreeMap で受け取り、汎用計算する。
/// 呼び出し元でテーブル種別ごとにフィールドを組み立てること。
pub fn compute_content_hash_for_rework(fields: BTreeMap<&str, String>) -> [u8; 32] {
    let canonical = serde_json::to_string(&fields)
        .expect("BTreeMap シリアライズは必ず成功する");

    let mut hasher = Sha256::new();
    hasher.update(canonical.as_bytes());
    hasher.finalize().into()
}

/// (FNC-BE-020) Disposition のコンテンツハッシュを計算する。
///
/// 対象フィールド: nonconformity_id / decision / quality_admin_sign_id /
///   supervisor_sign_id / decided_at
/// NOTE: Two-Person Integrity 保証のため両署名者 ID をハッシュに含める（ADR-011）。
pub fn compute_content_hash_for_disposition(record: &DispositionRecord) -> [u8; 32] {
    let mut fields = BTreeMap::new();
    fields.insert("decided_at",              record.decided_at.to_rfc3339());
    fields.insert("decision",               record.decision.clone());
    fields.insert("nonconformity_id",       record.nonconformity_id.to_string());
    fields.insert("quality_admin_sign_id",  record.quality_admin_sign_id.to_string());
    fields.insert("supervisor_sign_id",     record.supervisor_sign_id.to_string());

    let canonical = serde_json::to_string(&fields)
        .expect("BTreeMap シリアライズは必ず成功する");

    let mut hasher = Sha256::new();
    hasher.update(canonical.as_bytes());
    hasher.finalize().into()
}

/// IQC ハッシュ計算用中間表現
#[derive(Debug)]
pub struct InspectionRecord {
    pub inspection_id:         uuid::Uuid,
    pub lot_id:                uuid::Uuid,
    pub supplier_id:           uuid::Uuid,
    pub material_id:           uuid::Uuid,
    pub sampling_plan_id:      uuid::Uuid,
    pub sampling_plan_version: i32,
    pub lot_quantity:          i32,
    pub sample_size_n:         i32,
    pub accept_number_ac:      i32,
    pub reject_number_re:      i32,
    pub severity_state:        String,
    pub inspector_id:          uuid::Uuid,
    pub received_at:           chrono::DateTime<chrono::Utc>,
}

#[derive(Debug)]
pub struct DispositionRecord {
    pub disposition_id:          uuid::Uuid,
    pub nonconformity_id:        uuid::Uuid,
    pub decision:                String,
    pub quality_admin_sign_id:   uuid::Uuid,
    pub supervisor_sign_id:      uuid::Uuid,
    pub decided_at:              chrono::DateTime<chrono::Utc>,
}
```

**FNC-BE-018〜020 シグネチャ要約**

| 関数 ID | 関数名 | 入力 | 対象テーブル |
|---|---|---|---|
| FNC-BE-018 | `compute_content_hash_for_inspection` | `InspectionRecord` | TBL-038/040/041 |
| FNC-BE-019 | `compute_content_hash_for_rework` | `BTreeMap<&str, String>` | TBL-045/047/049/050 |
| FNC-BE-020 | `compute_content_hash_for_disposition` | `DispositionRecord` | TBL-044 |

---

## 3. Genesis ブロック定義（per-case_id / ADR-007）

```rust
// crates/wnav_hash_chain/src/genesis.rs

/// Genesis ブロック: ハッシュチェーンの先頭に挿入される初期ブロック。
///
/// # per-case_id genesis（ADR-007）
/// ジェネシスブロックの定義は case_id ごとに独立する。グローバルな単一 genesis は採用しない。
/// 各 case_id の最初のイベントブロックが genesis となり、prev_hash = [0u8; 32] を使用する。
/// chain_hash = SHA-256([0u8; 32] || content_hash)
pub const GENESIS_PREV_HASH: [u8; 32] = [0u8; 32];
pub const GENESIS_PREV_HASH_HEX: &str =
    "0000000000000000000000000000000000000000000000000000000000000000";

#[derive(Debug)]
pub struct GenesisBlock {
    pub block_id: uuid::Uuid,
    pub case_id: uuid::Uuid,
    pub prev_hash: String,
    pub content_hash: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}
```

---

## 3a. 補正ブロックのチェーンハッシュ計算（FNC-BE-017 / ADR-008）

```rust
// crates/wnav_hash_chain/src/correction.rs

use sha2::{Digest, Sha256};

/// (FNC-BE-017) 補正ブロックのチェーンハッシュを計算する。
///
/// # 設計判断 D2（ADR-008）
/// 補正レコードの prev_hash は破断ブロック（broken_at_block_id）の chain_hash を継承する。
/// フォーク（独立した genesis の割り当て）は禁止する。
/// 破断ブロック自体は Append-only 原則（ALCOA+ Original）により削除・更新しない。
///
/// 入力:
///   broken_block_chain_hash : [u8; 32]  -- 破断ブロックの chain_hash 値
///   correction_content_hash : [u8; 32]  -- 補正ブロックの content_hash（FNC-BE-009 で計算）
///
/// 処理: SHA-256(broken_block_chain_hash || correction_content_hash)
///
/// 出力: [u8; 32]  -- 補正ブロックの chain_hash
pub fn compute_correction_chain_hash(
    broken_block_chain_hash: &[u8; 32],
    correction_content_hash: &[u8; 32],
) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(broken_block_chain_hash);
    hasher.update(correction_content_hash);
    hasher.finalize().into()
}
```

**FNC-BE-017 シグネチャ要約**

| 項目 | 値 |
|---|---|
| 関数 ID | FNC-BE-017 |
| 入力 1 | `broken_block_chain_hash: [u8; 32]` — 破断ブロックの chain_hash |
| 入力 2 | `correction_content_hash: [u8; 32]` — 補正ブロックの content_hash |
| 処理 | `SHA-256(broken_block_chain_hash \|\| correction_content_hash)` |
| 出力 | `[u8; 32]` — 補正ブロックの chain_hash |
| 根拠 | D2（ADR-008）: 補正レコードは破断ブロックの chain_hash を継承（fork 禁止）|

---

## 3b. IQC・リワーク qc_case_id Genesis 設計（ADR-011 / ADR-007 拡張）

```rust
// crates/wnav_hash_chain/src/iqc_genesis.rs

/// IQC チェーンの genesis ブロック定義。
/// ADR-007（per case_id genesis）の思想を継承し、
/// qc_case_id 単位で独立した genesis を持つ。
///
/// # qc_case_id の割当規約（ADR-011）
/// | テーブル                            | qc_case_id 値           |
/// |-------------------------------------|------------------------|
/// | incoming_inspections                | self.inspection_id（genesis）|
/// | incoming_inspection_measurements    | inspection_id           |
/// | concession_approvals                | inspection_id           |
/// | dispositions                        | nonconformity_id        |
/// | rework_verifications                | rework_id               |
/// | reworked_lot_labels                 | rework_id               |
/// | scrap_records                       | rework_id               |
/// | return_to_vendor_records            | rework_id               |
///
/// # Genesis ルール
/// - qc_case_id ごとに最初に INSERT されたレコードが genesis となる
/// - genesis の prev_hash = GENESIS_PREV_HASH_HEX（"0"×64）
/// - genesis の content_hash = compute_content_hash_for_{table}(...) の結果
/// - 2 件目以降は前レコードの content_hash を prev_hash に使用する
///
/// # アプリ層での実装責務（wnav_master_api）
/// INSERT 前に以下を実行する:
/// 1. `SELECT content_hash FROM {table} WHERE qc_case_id = $1 ORDER BY {pk} DESC LIMIT 1`
/// 2. 結果が None なら prev_hash = GENESIS_PREV_HASH_HEX（genesis）
/// 3. 結果が Some(h) なら prev_hash = h（チェーン継続）
/// 4. content_hash = compute_content_hash_for_{table}(record)
/// 5. INSERT 実行

pub const QC_GENESIS_PREV_HASH: [u8; 32] = [0u8; 32];
pub const QC_GENESIS_PREV_HASH_HEX: &str =
    "0000000000000000000000000000000000000000000000000000000000000000";
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
            let is_correction: bool = row.try_get("is_correction").unwrap_or(false);
            let broken_at_block_id: Option<Uuid> = row.try_get("broken_at_block_id").ok().flatten();

            // 補正ブロック（is_correction=TRUE）の検証（ALG-025 / D2 / ADR-008）:
            //   expected_prev_hash = 直前の破断ブロック（broken_at_block_id）の chain_hash
            //   破断ブロック以降の検証は補正ブロックを起点として再開する
            if is_correction {
                if let Some(broken_id) = broken_at_block_id {
                    let broken_chain_hash: String = sqlx::query_scalar(
                        "SELECT chain_hash FROM hash_chain_blocks WHERE block_id = $1"
                    )
                    .bind(broken_id)
                    .fetch_one(self.pool.as_ref())
                    .await?;
                    expected_prev_hash = hex_to_bytes32(&broken_chain_hash)
                        .unwrap_or(crate::genesis::GENESIS_PREV_HASH);
                }
                // 補正ブロック以降の通常の連続性検証へ続く
            }

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

## 4a. IQC チェーン検証メソッド（ADR-011）

```rust
// crates/wnav_hash_chain/src/iqc_verifier.rs

impl HashChainService {
    /// IQC 検査チェーン（qc_case_id 単位）を全件検証する。
    /// BAT-001 スケジューラから呼び出される。
    pub async fn verify_inspection_chain_all(&self) -> Result<VerificationResult, sqlx::Error> {
        let start_time = chrono::Utc::now();
        let mut total_checked: u64 = 0;

        // 全 qc_case_id（distinct）を取得
        let case_ids: Vec<uuid::Uuid> = sqlx::query_scalar(
            "SELECT DISTINCT qc_case_id FROM incoming_inspection_measurements ORDER BY qc_case_id"
        )
        .fetch_all(self.pool.as_ref())
        .await?;

        for qc_case_id in case_ids {
            let result = self.verify_inspection_chain(qc_case_id).await?;
            if !result.is_valid {
                return Ok(VerificationResult {
                    is_valid: false,
                    broken_at_block_id: result.broken_at_block_id,
                    checked_count: total_checked + result.checked_count,
                    verified_at: start_time,
                });
            }
            total_checked += result.checked_count;
        }

        Ok(VerificationResult {
            is_valid: true,
            broken_at_block_id: None,
            checked_count: total_checked,
            verified_at: start_time,
        })
    }

    /// 指定 qc_case_id（= inspection_id）の IQC 測定値チェーンを検証する。
    /// incoming_inspection_measurements を measurement_id（UUID v7 = 時系列）昇順で取得し
    /// prev_hash / content_hash の連続性を確認する。
    pub async fn verify_inspection_chain(
        &self,
        qc_case_id: uuid::Uuid,
    ) -> Result<VerificationResult, sqlx::Error> {
        let start_time = chrono::Utc::now();

        let rows = sqlx::query(
            r#"
            SELECT measurement_id, inspection_id, sample_no, measured_value,
                   defect_flag, measured_at, qc_case_id, prev_hash, content_hash
            FROM incoming_inspection_measurements
            WHERE qc_case_id = $1
            ORDER BY measurement_id ASC
            "#,
        )
        .bind(qc_case_id)
        .fetch_all(self.pool.as_ref())
        .await?;

        let mut expected_prev_hash = crate::iqc_genesis::QC_GENESIS_PREV_HASH;
        let mut checked_count: u64 = 0;

        for row in &rows {
            let stored_content_hash: String = row.try_get("content_hash").unwrap_or_default();
            let stored_prev_hash: String = row.try_get("prev_hash").unwrap_or_default();
            let measurement_id: uuid::Uuid = row.try_get("measurement_id").unwrap_or_default();

            if stored_prev_hash != crate::hash::bytes32_to_hex(&expected_prev_hash) {
                return Ok(VerificationResult {
                    is_valid: false,
                    broken_at_block_id: Some(measurement_id),
                    checked_count,
                    verified_at: start_time,
                });
            }

            // content_hash 再計算と比較（FNC-BE-018 ロジック）
            let recomputed = crate::iqc_hash::recompute_inspection_measurement_hash(&row)?;
            if crate::hash::bytes32_to_hex(&recomputed) != stored_content_hash {
                return Ok(VerificationResult {
                    is_valid: false,
                    broken_at_block_id: Some(measurement_id),
                    checked_count,
                    verified_at: start_time,
                });
            }

            let prev_bytes = crate::hash::hex_to_bytes32(&stored_prev_hash)
                .unwrap_or(crate::iqc_genesis::QC_GENESIS_PREV_HASH);
            expected_prev_hash = crate::hash::compute_chain_hash(&prev_bytes, &recomputed);
            checked_count += 1;
        }

        Ok(VerificationResult {
            is_valid: true,
            broken_at_block_id: None,
            checked_count,
            verified_at: start_time,
        })
    }

    /// リワーク系チェーン（rework_verifications / reworked_lot_labels 等）全件検証。
    /// 各テーブルを qc_case_id（= rework_id / nonconformity_id）単位で検証する。
    /// 実装はタスクとして分離し、verify_inspection_chain_all と同パターンで実装する。
    pub async fn verify_rework_chain_all(&self) -> Result<VerificationResult, sqlx::Error> {
        // 実装省略（verify_inspection_chain_all と同パターン）
        // 対象テーブル: rework_verifications / reworked_lot_labels / scrap_records / return_to_vendor_records
        // 各テーブルの qc_case_id DISTINCT で全 rework_id / nonconformity_id を取得し検証する
        todo!("ADR-011 実装タスクで確定")
    }
}
```

---

## 5. 週次検証スケジューラ（BAT-001 常駐タスク）

> **起動バイナリ**: `run_weekly_verifier` は `wnav_master_api` の `main.rs` で
> `tokio::spawn(run_weekly_verifier(svc.clone()))` として起動する（BAT-001）。
> `wnav_terminal_api` はこの関数を呼び出さない。

```rust
// crates/wnav_hash_chain/src/scheduler.rs

use std::sync::Arc;
use crate::HashChainService;

/// BAT-001: 週次ハッシュチェーン検証スケジューラ。
/// tokio-cron-scheduler を使用して月曜 02:00（JST）に実行する。
/// wnav_master_api の main.rs で tokio::spawn して起動する。
///
/// 検証順序:
/// 1. work_events チェーン（既存 verify_chain）
/// 2. IQC 入荷検査チェーン（verify_inspection_chain_all / ADR-011）
/// 3. リワーク系チェーン（verify_rework_chain_all / ADR-011）
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
                // 1. work_events チェーン検証（既存）
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

                // 2. IQC 入荷検査チェーン検証（ADR-011）
                match svc.verify_inspection_chain_all().await {
                    Ok(result) => {
                        if !result.is_valid {
                            tracing::error!(
                                log_id = "LOG-ERR-004",
                                event_name = "iqc_chain.broken",
                                broken_at = ?result.broken_at_block_id,
                                checked_count = result.checked_count,
                            );
                        } else {
                            tracing::info!(
                                log_id = "LOG-BAT-001",
                                event_name = "iqc_chain.weekly_ok",
                                checked_count = result.checked_count,
                            );
                        }
                    }
                    Err(e) => {
                        tracing::error!(
                            log_id = "LOG-ERR-005",
                            event_name = "iqc_chain.verify_failed",
                            error = %e,
                        );
                    }
                }

                // 3. リワーク系チェーン検証（ADR-011）
                match svc.verify_rework_chain_all().await {
                    Ok(result) => {
                        if !result.is_valid {
                            tracing::error!(
                                log_id = "LOG-ERR-006",
                                event_name = "rework_chain.broken",
                                broken_at = ?result.broken_at_block_id,
                                checked_count = result.checked_count,
                            );
                        } else {
                            tracing::info!(
                                log_id = "LOG-BAT-001",
                                event_name = "rework_chain.weekly_ok",
                                checked_count = result.checked_count,
                            );
                        }
                    }
                    Err(e) => {
                        tracing::error!(
                            log_id = "LOG-ERR-007",
                            event_name = "rework_chain.verify_failed",
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
- **ジェネシスブロックは per-case_id genesis（ADR-007）とし、グローバルな単一 genesis は採用しない。各 case_id の最初のイベントブロックが genesis（prev_hash = [0u8;32]）となる。**
- **compute_correction_chain_hash（FNC-BE-017）を追加し、補正ブロックの prev_hash が破断ブロックの chain_hash を継承することを ADR-008 として確定した（fork 禁止）。**
- **週次検証（BAT-001 / master-api 内）は event_id（UUID v7 = 時系列）の昇順ストリーミングで全件検証し、不整合を検知した時点で即座に ERR-DB-003 ログを出力することを確定した。補正ブロック（is_correction=TRUE）検出時は破断ブロックの chain_hash を expected_prev_hash に設定して検証を継続する（ALG-025）。**
- **FNC-BE-018〜020（IQC/リワーク/ディスポジション向けコンテンツハッシュ計算）を追加し、既存の compute_content_hash ロジックを再利用した IQC 専用ラッパ関数として設計した（ADR-011）。**
- **per qc_case_id genesis 方式を §3b で確定し、ADR-007 の per case_id genesis の延長として IQC チェーンに適用した。アプリ層（wnav_master_api）が INSERT 前に前レコードの content_hash を取得して prev_hash に設定する責務を持つ。**
- **BAT-001 スケジューラを拡張し、work_events チェーン → IQC 入荷検査チェーン → リワーク系チェーンの順に週次検証を実行する 3 段構成とした（ADR-011）。**
- **本クレートは `wnav_master_api` のみが依存し、`wnav_terminal_api` は依存しない。ハッシュチェーン検証は管理操作であり、端末 API からは実行しない設計を確定した。**

---

## 参照業界分析

### 必須
- [`90_業界分析/09_セキュリティとアクセス制御.md`](../../90_業界分析/09_セキュリティとアクセス制御.md)

### 関連
- [`90_業界分析/06_品質管理とトレーサビリティ.md`](../../90_業界分析/06_品質管理とトレーサビリティ.md)
