// wnav_hash_chain クレート
//
// SHA-256 ハッシュチェーン計算・検証ライブラリ（MOD-BE-003）。
// 製造作業記録の改ざんを構造的に検出するために使用する（FR-EV-001/002）。
//
// # アーキテクチャ
// - `canonical`: ALG-006 canonical JSON 正規化
// - `hash`: ALG-007/008 ハッシュ計算
// - `verify`: ハッシュチェーン検証
// - `correction`: ALG-025 / FNC-BE-017〜020 補正ハッシュ計算
// - `error`: エラー型定義
//
// # 依存クレート禁止
// 本クレートは他のカスタムクレート（wnav_domain / wnav_db 等）に依存しない。
// 完全独立のユーティリティクレートとして設計する。

// unsafe コードを禁止する（src/CLAUDE.md および src/backend/CLAUDE.md の必須要件）
#![forbid(unsafe_code)]

pub mod canonical;
pub mod correction;
pub mod error;
pub mod hash;
pub mod verify;

// 主要な型・関数を再エクスポートして使いやすくする
pub use canonical::canonical_json;
pub use correction::{
    compute_content_hash_for_disposition, compute_content_hash_for_inspection,
    compute_content_hash_for_rework, compute_correction_chain_hash,
};
pub use error::HashChainError;
pub use hash::{
    GENESIS_PREV_HASH, bytes32_to_hex, compute_chain_hash, compute_content_hash, hex_to_bytes32,
};
pub use verify::{ChainBlock, ChainVerifyError, verify_chain};

#[cfg(test)]
mod tests {
    use crate::canonical::canonical_json;
    use crate::hash::{GENESIS_PREV_HASH, compute_chain_hash, compute_content_hash};
    use crate::verify::{ChainBlock, ChainVerifyError, verify_chain};
    use chrono::Utc;
    use serde_json::json;
    use uuid::Uuid;

    // テスト用ブロックを生成するヘルパー関数。
    fn make_block(
        case_id: Uuid,
        sequence_number: i64,
        prev_hash: [u8; 32],
        payload: &serde_json::Value,
    ) -> ChainBlock {
        let canonical = canonical_json(payload);
        let content_hash = compute_content_hash(&canonical);
        let block_hash = compute_chain_hash(&prev_hash, &content_hash);
        ChainBlock {
            id: Uuid::now_v7(),
            case_id,
            sequence_number,
            prev_block_hash: prev_hash,
            content_hash,
            block_hash,
            created_at: Utc::now(),
        }
    }

    #[test]
    fn test_alg_007_known_vector_canonical_json_determinism() {
        // ALG-007: canonical JSON の決定論性を既知ベクターで確認する
        // 異なるキー順序の同一内容のオブジェクトが同一の canonical JSON になることを確認する
        let value1 = json!({ "z": "last", "a": "first", "m": "middle" });
        let value2 = json!({ "a": "first", "m": "middle", "z": "last" });
        let value3 = json!({ "m": "middle", "z": "last", "a": "first" });

        let canonical1 = canonical_json(&value1);
        let canonical2 = canonical_json(&value2);
        let canonical3 = canonical_json(&value3);

        // すべて同一の canonical JSON になることを確認する
        assert_eq!(canonical1, canonical2);
        assert_eq!(canonical2, canonical3);

        // アルファベット順のキーで出力されることを確認する
        assert_eq!(canonical1, r#"{"a":"first","m":"middle","z":"last"}"#);

        // 同一 canonical JSON から同一のコンテンツハッシュが得られることを確認する
        let h1 = compute_content_hash(&canonical1);
        let h2 = compute_content_hash(&canonical2);
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_alg_008_chain_genesis_to_block2() {
        // ALG-008: genesis → ブロック1 → ブロック2 のチェーン計算が正しいことを確認する
        let case_id = Uuid::now_v7();

        // genesis ブロック（prev = [0u8;32]）
        let canonical1 =
            canonical_json(&json!({ "activity": "case_started", "case_id": case_id.to_string() }));
        let content1 = compute_content_hash(&canonical1);
        let chain1 = compute_chain_hash(&GENESIS_PREV_HASH, &content1);

        // ブロック1（prev = genesis の chain_hash）
        let canonical2 = canonical_json(
            &json!({ "activity": "step_1_completed", "case_id": case_id.to_string() }),
        );
        let content2 = compute_content_hash(&canonical2);
        let chain2 = compute_chain_hash(&chain1, &content2);

        // ブロック2（prev = ブロック1 の chain_hash）
        let canonical3 = canonical_json(
            &json!({ "activity": "case_completed", "case_id": case_id.to_string() }),
        );
        let content3 = compute_content_hash(&canonical3);
        let chain3 = compute_chain_hash(&chain2, &content3);

        // 各ハッシュが 32 バイトで非ゼロであることを確認する
        assert_eq!(content1.len(), 32);
        assert_ne!(content1, [0u8; 32]);
        assert_ne!(chain1, GENESIS_PREV_HASH);
        assert_ne!(chain2, chain1);
        assert_ne!(chain3, chain2);

        // 決定論性の確認: 同じ canonical JSON から同じハッシュが生成されることを確認する
        let content1_again = compute_content_hash(&canonical1);
        let chain1_again = compute_chain_hash(&GENESIS_PREV_HASH, &content1_again);
        assert_eq!(chain1, chain1_again);
    }

    #[test]
    fn test_verify_chain_valid() {
        // verify_chain: 正常チェーンが Ok を返すことを確認する
        let case_id = Uuid::now_v7();

        let block1 = make_block(
            case_id,
            1,
            GENESIS_PREV_HASH,
            &json!({ "activity": "start" }),
        );
        let block2 = make_block(
            case_id,
            2,
            block1.block_hash,
            &json!({ "activity": "step1" }),
        );
        let block3 = make_block(
            case_id,
            3,
            block2.block_hash,
            &json!({ "activity": "finish" }),
        );

        assert!(verify_chain(&[block1, block2, block3]).is_ok());
    }

    #[test]
    fn test_verify_chain_broken() {
        // verify_chain: 破断チェーンが HashMismatch エラーを返すことを確認する
        let case_id = Uuid::now_v7();

        let block1 = make_block(
            case_id,
            1,
            GENESIS_PREV_HASH,
            &json!({ "activity": "start" }),
        );
        let mut block2 = make_block(
            case_id,
            2,
            block1.block_hash,
            &json!({ "activity": "step1" }),
        );
        // block_hash を改ざんして破断させる
        block2.block_hash = [0xFF_u8; 32];
        let block3 = make_block(
            case_id,
            3,
            block2.block_hash,
            &json!({ "activity": "finish" }),
        );

        let result = verify_chain(&[block1, block2, block3]);
        assert!(matches!(result, Err(ChainVerifyError::HashMismatch { .. })));
    }

    #[test]
    fn test_verify_chain_sequence_gap() {
        // verify_chain: シーケンスギャップが SequenceGap エラーを返すことを確認する
        let case_id = Uuid::now_v7();

        let block1 = make_block(
            case_id,
            1,
            GENESIS_PREV_HASH,
            &json!({ "activity": "start" }),
        );
        // シーケンス番号を 2 を飛ばして 3 にする
        let block_gap = make_block(
            case_id,
            3,
            block1.block_hash,
            &json!({ "activity": "finish" }),
        );

        let result = verify_chain(&[block1, block_gap]);
        assert!(matches!(result, Err(ChainVerifyError::SequenceGap { .. })));
    }

    #[test]
    fn test_correction_hash_continues_chain() {
        // 補正ハッシュ: 補正イベント後もチェーンが連続することを確認する（ADR-008）
        use crate::correction::compute_correction_chain_hash;

        let case_id = Uuid::now_v7();
        let original_event_id = Uuid::now_v7();

        // 正常ブロックを 2 つ作成する
        let block1 = make_block(
            case_id,
            1,
            GENESIS_PREV_HASH,
            &json!({ "activity": "start" }),
        );
        let block2 = make_block(
            case_id,
            2,
            block1.block_hash,
            &json!({ "activity": "step1" }),
        );

        // block2 を「破断ブロック」として補正ブロックを計算する
        let correction_payload = json!({
            "approver_primary": Uuid::now_v7().to_string(),
            "approver_secondary": Uuid::now_v7().to_string(),
            "correction_reason": "記録ミスの訂正",
            "is_correction": true,
            "original_record_id": original_event_id.to_string(),
        });

        let (correction_content, correction_chain) = compute_correction_chain_hash(
            &block2.block_hash, // 破断ブロックの chain_hash を prev として使用
            &correction_payload,
            original_event_id,
        );

        // 補正ブロックのハッシュが非ゼロで計算されていることを確認する
        assert_ne!(correction_content, [0u8; 32]);
        assert_ne!(correction_chain, [0u8; 32]);

        // 補正ブロックは genesis（[0u8;32]）ではなく破断ブロックの chain_hash を基に計算されている
        // よって genesis を prev として計算した場合とは異なるはず
        let (_, correction_chain_if_genesis) = compute_correction_chain_hash(
            &GENESIS_PREV_HASH,
            &correction_payload,
            original_event_id,
        );
        assert_ne!(correction_chain, correction_chain_if_genesis);

        // 補正ブロックの chain_hash は SHA-256(block2.chain_hash || correction_content) と一致する
        let expected = compute_chain_hash(&block2.block_hash, &correction_content);
        assert_eq!(correction_chain, expected);
    }
}
