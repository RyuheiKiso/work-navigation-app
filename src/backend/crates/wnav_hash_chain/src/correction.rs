// ALG-025 + FNC-BE-017〜020: 補正ハッシュ計算モジュール
// ADR-008「補正レコードはチェーンを継続する」規則に従い、破断ブロックの
// chain_hash を引き継いでチェーンを継続する補正ブロックのハッシュを計算する。

use crate::canonical::canonical_json;
use crate::hash::compute_content_hash;
use crate::hash::compute_chain_hash;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use uuid::Uuid;

/// FNC-BE-017: 補正イベントのチェーンハッシュを計算する。
///
/// # ADR-008: 補正レコード継続規則
/// - 補正レコードの `prev_hash` = 破断ブロック（broken_at_block_id）の `chain_hash` を継承する
/// - フォーク（独立した genesis の割り当て）は禁止する（ALCOA+ Original 違反）
/// - 破断ブロック自体は Append-only 原則により削除・更新しない
///
/// # 補正ブロックの content_hash 計算
/// 以下のフィールドを BTreeMap アルファベット順で canonical JSON に変換してハッシュを計算する:
/// - approver_primary: 1 名目承認者の UUID
/// - approver_secondary: 2 名目承認者の UUID（approver_primary と異なること）
/// - correction_reason: 訂正理由（自由記述、必須）
/// - is_correction: true 固定
/// - original_record_id: 訂正対象の work_events.event_id
///
/// # 引数
/// - `prev_block_hash`: 破断ブロックの chain_hash（補正の起点となるハッシュ）
/// - `correction_payload`: 補正メタデータを含む JSON（上記フィールドを含むこと）
/// - `original_event_id`: 訂正対象の work_events.event_id
///
/// # 戻り値
/// `(content_hash, chain_hash)` のタプル
pub fn compute_correction_chain_hash(
    prev_block_hash: &[u8; 32],
    correction_payload: &serde_json::Value,
    original_event_id: Uuid,
) -> ([u8; 32], [u8; 32]) {
    // 補正メタデータを含む BTreeMap を構築してアルファベット順の canonical JSON にする
    // original_record_id を payload に追加して content_hash を計算する
    let mut merged = match correction_payload {
        serde_json::Value::Object(map) => {
            let mut m: BTreeMap<String, serde_json::Value> = BTreeMap::new();
            for (k, v) in map {
                m.insert(k.clone(), v.clone());
            }
            m
        }
        _ => BTreeMap::new(),
    };

    // original_event_id が payload に含まれていない場合は追加する
    merged
        .entry("original_record_id".to_string())
        .or_insert_with(|| serde_json::Value::String(original_event_id.to_string()));

    // is_correction フラグを確実に true に設定する
    merged.insert(
        "is_correction".to_string(),
        serde_json::Value::Bool(true),
    );

    // BTreeMap を serde_json::Value に変換して canonical JSON を生成する
    let merged_value = serde_json::Value::Object(
        merged
            .into_iter()
            .collect::<serde_json::Map<String, serde_json::Value>>(),
    );
    let canonical = canonical_json(&merged_value);

    // コンテンツハッシュを計算する（canonical JSON の SHA-256）
    let content_hash = compute_content_hash(&canonical);

    // チェーンハッシュを計算する（破断ブロックの chain_hash を prev として使用）
    // ADR-008: prev_hash = 破断ブロックの chain_hash（fork 禁止）
    let chain_hash = compute_chain_hash(prev_block_hash, &content_hash);

    (content_hash, chain_hash)
}

/// FNC-BE-018: 入荷検査（incoming_inspections）のコンテンツハッシュを計算する。
///
/// # ADR-011: IQC テーブルへのハッシュチェーン拡張
/// TBL-038 incoming_inspections の改ざん検知のためのコンテンツハッシュを計算する。
///
/// # ハッシュ対象フィールド（BTreeMap アルファベット順）
/// inspection_id / lot_id / supplier_id / material_id / inspector_id / received_at / qc_status
/// NOTE: qc_status / judged_at は可変フィールドのためハッシュ対象外は ADR-011 参照
///
/// # 引数
/// - `payload`: 入荷検査レコードのフィールドを含む JSON オブジェクト
pub fn compute_content_hash_for_inspection(payload: &serde_json::Value) -> [u8; 32] {
    // canonical_json でアルファベット順ソートの canonical JSON に変換する
    let canonical = canonical_json(payload);
    compute_content_hash(&canonical)
}

/// FNC-BE-019: リワーク系テーブルのコンテンツハッシュを計算する。
///
/// # ADR-011: IQC テーブルへのハッシュチェーン拡張
/// 以下のテーブルに対応する汎用ハッシュ計算関数:
/// - TBL-045 rework_verifications
/// - TBL-047 reworked_lot_labels
/// - TBL-049 scrap_records
/// - TBL-050 return_to_vendor_records
///
/// 呼び出し元がテーブル種別ごとに対象フィールドを組み立てて渡すこと。
///
/// # 引数
/// - `payload`: リワーク系レコードのフィールドを含む JSON オブジェクト
pub fn compute_content_hash_for_rework(payload: &serde_json::Value) -> [u8; 32] {
    // canonical_json でアルファベット順ソートの canonical JSON に変換する
    let canonical = canonical_json(payload);
    compute_content_hash(&canonical)
}

/// FNC-BE-020: ディスポジション（dispositions）のコンテンツハッシュを計算する。
///
/// # ADR-011: IQC テーブルへのハッシュチェーン拡張
/// TBL-044 dispositions の改ざん検知のためのコンテンツハッシュを計算する。
/// Two-Person Integrity 保証のため両署名者 ID をハッシュに含める。
///
/// # ハッシュ対象フィールド（BTreeMap アルファベット順）
/// nonconformity_id / decision / quality_admin_sign_id / supervisor_sign_id / decided_at
///
/// # 引数
/// - `payload`: ディスポジションレコードのフィールドを含む JSON オブジェクト
pub fn compute_content_hash_for_disposition(payload: &serde_json::Value) -> [u8; 32] {
    // canonical_json でアルファベット順ソートの canonical JSON に変換する
    let canonical = canonical_json(payload);
    compute_content_hash(&canonical)
}

/// IQC ハッシュ計算の内部実装: canonical JSON 生成前のフィールド抽出。
/// 呼び出し元が任意の BTreeMap を構築して canonical_json を呼ぶためのユーティリティ。
pub fn compute_content_hash_from_fields(fields: BTreeMap<&str, String>) -> [u8; 32] {
    // BTreeMap を serde_json::Value に変換して canonical JSON を生成する
    let value: serde_json::Value = fields
        .into_iter()
        .map(|(k, v)| (k.to_string(), serde_json::Value::String(v)))
        .collect::<serde_json::Map<_, _>>()
        .into();
    let canonical = canonical_json(&value);
    let mut hasher = Sha256::new();
    hasher.update(canonical.as_bytes());
    hasher.finalize().into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hash::GENESIS_PREV_HASH;
    use serde_json::json;

    #[test]
    fn test_compute_correction_chain_hash_continues_chain() {
        // 補正イベント後もチェーンが連続することを確認する（ADR-008）
        let original_event_id = Uuid::now_v7();

        // 仮の破断ブロックの chain_hash を設定する（実際は DB から取得する）
        let broken_block_chain_hash: [u8; 32] = [0xAB_u8; 32];

        let correction_payload = json!({
            "approver_primary": Uuid::now_v7().to_string(),
            "approver_secondary": Uuid::now_v7().to_string(),
            "correction_reason": "データ入力ミスの訂正",
            "is_correction": true,
            "original_record_id": original_event_id.to_string(),
        });

        let (content_hash, chain_hash) = compute_correction_chain_hash(
            &broken_block_chain_hash,
            &correction_payload,
            original_event_id,
        );

        // content_hash と chain_hash が非ゼロであることを確認する
        assert_ne!(content_hash, [0u8; 32]);
        assert_ne!(chain_hash, [0u8; 32]);

        // chain_hash が SHA-256(broken_block_chain_hash || content_hash) であることを確認する
        let expected_chain = compute_chain_hash(&broken_block_chain_hash, &content_hash);
        assert_eq!(chain_hash, expected_chain);
    }

    #[test]
    fn test_compute_correction_chain_hash_not_genesis() {
        // 補正ブロックは genesis（prev = [0u8;32]）ではなく、破断ブロックの chain_hash を引き継ぐことを確認する
        let original_event_id = Uuid::now_v7();
        let non_genesis_prev: [u8; 32] = [0xDE_u8; 32];

        let correction_payload = json!({
            "correction_reason": "テスト訂正",
            "is_correction": true,
        });

        let (_, chain_hash_with_non_genesis) = compute_correction_chain_hash(
            &non_genesis_prev,
            &correction_payload,
            original_event_id,
        );

        let (_, chain_hash_with_genesis) = compute_correction_chain_hash(
            &GENESIS_PREV_HASH,
            &correction_payload,
            original_event_id,
        );

        // genesis を使った場合と non-genesis を使った場合で chain_hash が異なることを確認する
        // これにより補正ブロックが正しく破断ブロックの chain_hash を引き継いでいることが分かる
        assert_ne!(chain_hash_with_non_genesis, chain_hash_with_genesis);
    }

    #[test]
    fn test_compute_content_hash_for_inspection() {
        // 入荷検査のコンテンツハッシュ計算が決定論的であることを確認する
        let payload = json!({
            "inspection_id": "11111111-0000-7000-8000-000000000001",
            "lot_id": "11111111-0000-7000-8000-000000000002",
            "supplier_id": "11111111-0000-7000-8000-000000000003",
            "material_id": "11111111-0000-7000-8000-000000000004",
            "inspector_id": "11111111-0000-7000-8000-000000000005",
            "received_at": "2026-05-19T00:00:00Z",
            "qc_status": "PENDING",
        });
        let h1 = compute_content_hash_for_inspection(&payload);
        let h2 = compute_content_hash_for_inspection(&payload);
        assert_eq!(h1, h2);
        assert_ne!(h1, [0u8; 32]);
    }

    #[test]
    fn test_compute_content_hash_for_rework() {
        // リワークのコンテンツハッシュ計算が決定論的であることを確認する
        let payload = json!({
            "rework_id": "22222222-0000-7000-8000-000000000001",
            "verifier_id": "22222222-0000-7000-8000-000000000002",
            "verdict": "PASS",
            "verified_at": "2026-05-19T00:00:00Z",
        });
        let h1 = compute_content_hash_for_rework(&payload);
        let h2 = compute_content_hash_for_rework(&payload);
        assert_eq!(h1, h2);
        assert_ne!(h1, [0u8; 32]);
    }

    #[test]
    fn test_compute_content_hash_for_disposition() {
        // ディスポジションのコンテンツハッシュ計算が決定論的であることを確認する
        let payload = json!({
            "nonconformity_id": "33333333-0000-7000-8000-000000000001",
            "decision": "SCRAP",
            "quality_admin_sign_id": "33333333-0000-7000-8000-000000000002",
            "supervisor_sign_id": "33333333-0000-7000-8000-000000000003",
            "decided_at": "2026-05-19T00:00:00Z",
        });
        let h1 = compute_content_hash_for_disposition(&payload);
        let h2 = compute_content_hash_for_disposition(&payload);
        assert_eq!(h1, h2);
        assert_ne!(h1, [0u8; 32]);
    }
}
