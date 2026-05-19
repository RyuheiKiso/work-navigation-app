// ハッシュチェーン検証モジュール
// 単一 case_id のブロック列を検証し、チェーンの整合性を確認する（ALG-008）。
// per-case_id genesis（ADR-007）に従い、各 case_id は独立したチェーンを持つ。

use crate::hash::{GENESIS_PREV_HASH, compute_chain_hash};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// ハッシュチェーンブロックの構造体。
/// TBL-031 `hash_chain_blocks` テーブルの行に対応する。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainBlock {
    /// ブロック識別子（UUID v7、タイムスタンプ埋め込み）
    pub id: Uuid,
    /// 作業セッション ID（TBL-005 work_executions の FK）
    pub case_id: Uuid,
    /// case_id 内での連番（1 始まり）
    pub sequence_number: i64,
    /// 直前ブロックの chain_hash（genesis ブロックは GENESIS_PREV_HASH = [0u8;32]）
    pub prev_block_hash: [u8; 32],
    /// SHA-256(canonical_json(event_record)) で計算したコンテンツハッシュ
    pub content_hash: [u8; 32],
    /// SHA-256(prev_block_hash || content_hash) で計算したチェーンハッシュ
    pub block_hash: [u8; 32],
    /// サーバー記録時刻（UTC）
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// チェーン検証エラー型。
///
/// ハッシュ不一致とシーケンス番号の不連続を明示的に区別する。
#[derive(Debug, thiserror::Error)]
pub enum ChainVerifyError {
    /// ハッシュ値が不一致な場合のエラー（チェーン破断）。
    #[error(
        "チェーンが破断しています: case_id={case_id}, seq={sequence_number}, expected={expected_hex}, actual={actual_hex}"
    )]
    HashMismatch {
        /// チェーンが破断した作業セッション ID
        case_id: Uuid,
        /// 破断が検出されたシーケンス番号
        sequence_number: i64,
        /// 期待されるハッシュ値（hex 表現）
        expected_hex: String,
        /// 実際のハッシュ値（hex 表現）
        actual_hex: String,
    },

    /// シーケンス番号が不連続な場合のエラー（ブロック欠落）。
    #[error(
        "チェーンのシーケンス番号が不連続です: case_id={case_id}, gap_at={sequence_number}"
    )]
    SequenceGap {
        /// シーケンスギャップが発生した作業セッション ID
        case_id: Uuid,
        /// ギャップが検出されたシーケンス番号
        sequence_number: i64,
    },
}

/// 単一 case_id のブロック列を検証する。
///
/// # 事前条件
/// - `blocks` は sequence_number 昇順でソートされていること
/// - `blocks` は同一 case_id のブロックのみを含むこと
///
/// # 検証内容
/// 1. sequence_number が 1 始まりで連続しているか確認する
/// 2. genesis ブロックの prev_block_hash が GENESIS_PREV_HASH（全ゼロ）であるか確認する
/// 3. 各ブロックの block_hash が SHA-256(prev_block_hash || content_hash) と一致するか確認する
/// 4. 各ブロックの prev_block_hash が直前ブロックの block_hash と一致するか確認する
///
/// # 戻り値
/// - `Ok(())`: チェーン全体が整合している
/// - `Err(ChainVerifyError::SequenceGap)`: シーケンス番号が不連続
/// - `Err(ChainVerifyError::HashMismatch)`: ハッシュ値が不一致（チェーン破断）
pub fn verify_chain(blocks: &[ChainBlock]) -> Result<(), ChainVerifyError> {
    // 空のチェーンは常に有効とみなす
    if blocks.is_empty() {
        return Ok(());
    }

    // genesis ブロックは GENESIS_PREV_HASH から始まる
    let mut expected_prev_hash = GENESIS_PREV_HASH;
    let mut expected_seq: i64 = 1;

    for block in blocks {
        // シーケンス番号の連続性を確認する
        if block.sequence_number != expected_seq {
            return Err(ChainVerifyError::SequenceGap {
                case_id: block.case_id,
                sequence_number: block.sequence_number,
            });
        }

        // prev_block_hash が期待値と一致するか確認する
        if block.prev_block_hash != expected_prev_hash {
            return Err(ChainVerifyError::HashMismatch {
                case_id: block.case_id,
                sequence_number: block.sequence_number,
                expected_hex: hex::encode(expected_prev_hash),
                actual_hex: hex::encode(block.prev_block_hash),
            });
        }

        // block_hash を再計算して保存値と一致するか確認する
        let recomputed_hash = compute_chain_hash(&block.prev_block_hash, &block.content_hash);
        if recomputed_hash != block.block_hash {
            return Err(ChainVerifyError::HashMismatch {
                case_id: block.case_id,
                sequence_number: block.sequence_number,
                expected_hex: hex::encode(recomputed_hash),
                actual_hex: hex::encode(block.block_hash),
            });
        }

        // 次のブロックの期待値を更新する
        expected_prev_hash = block.block_hash;
        expected_seq += 1;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::canonical::canonical_json;
    use crate::hash::{compute_chain_hash, compute_content_hash, GENESIS_PREV_HASH};
    use chrono::Utc;
    use serde_json::json;

    /// テスト用ブロックを生成するヘルパー関数。
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
    fn test_verify_chain_valid_single_block() {
        // 単一ブロックの正常チェーンを検証する
        let case_id = Uuid::now_v7();
        let payload = json!({ "activity": "start", "case_id": case_id.to_string() });
        let block = make_block(case_id, 1, GENESIS_PREV_HASH, &payload);
        assert!(verify_chain(&[block]).is_ok());
    }

    #[test]
    fn test_verify_chain_valid_multiple_blocks() {
        // genesis → ブロック1 → ブロック2 の正常チェーンを検証する
        let case_id = Uuid::now_v7();

        let payload1 = json!({ "activity": "start", "seq": 1 });
        let block1 = make_block(case_id, 1, GENESIS_PREV_HASH, &payload1);

        let payload2 = json!({ "activity": "step_complete", "seq": 2 });
        let block2 = make_block(case_id, 2, block1.block_hash, &payload2);

        let payload3 = json!({ "activity": "finish", "seq": 3 });
        let block3 = make_block(case_id, 3, block2.block_hash, &payload3);

        assert!(verify_chain(&[block1, block2, block3]).is_ok());
    }

    #[test]
    fn test_verify_chain_empty() {
        // 空のチェーンは有効とみなすことを確認する
        assert!(verify_chain(&[]).is_ok());
    }

    #[test]
    fn test_verify_chain_broken_hash() {
        // チェーンが破断している場合に HashMismatch エラーが返ることを確認する
        let case_id = Uuid::now_v7();

        let payload1 = json!({ "activity": "start" });
        let block1 = make_block(case_id, 1, GENESIS_PREV_HASH, &payload1);

        let payload2 = json!({ "activity": "step_complete" });
        let mut block2 = make_block(case_id, 2, block1.block_hash, &payload2);
        // block_hash を意図的に改ざんする
        block2.block_hash = [0xFF_u8; 32];

        let payload3 = json!({ "activity": "finish" });
        // block2 の改ざんされた block_hash を prev として使用する
        let block3 = make_block(case_id, 3, block2.block_hash, &payload3);

        let result = verify_chain(&[block1, block2, block3]);
        assert!(matches!(
            result,
            Err(ChainVerifyError::HashMismatch {
                sequence_number: 2,
                ..
            })
        ));
    }

    #[test]
    fn test_verify_chain_sequence_gap() {
        // シーケンス番号にギャップがある場合に SequenceGap エラーが返ることを確認する
        let case_id = Uuid::now_v7();

        let payload1 = json!({ "activity": "start" });
        let block1 = make_block(case_id, 1, GENESIS_PREV_HASH, &payload1);

        let payload3 = json!({ "activity": "finish" });
        // シーケンス番号が 2 を飛ばして 3 になっているブロックを作成する
        let block3 = make_block(case_id, 3, block1.block_hash, &payload3);

        let result = verify_chain(&[block1, block3]);
        assert!(matches!(
            result,
            Err(ChainVerifyError::SequenceGap {
                sequence_number: 3,
                ..
            })
        ));
    }

    #[test]
    fn test_verify_chain_prev_hash_mismatch() {
        // prev_block_hash が直前ブロックの block_hash と異なる場合に HashMismatch が返ることを確認する
        let case_id = Uuid::now_v7();

        let payload1 = json!({ "activity": "start" });
        let block1 = make_block(case_id, 1, GENESIS_PREV_HASH, &payload1);

        let payload2 = json!({ "activity": "step_complete" });
        // 正しくない prev_hash（全ゼロ）を使ってブロック2 を作成する
        let block2_wrong_prev = make_block(case_id, 2, GENESIS_PREV_HASH, &payload2);

        let result = verify_chain(&[block1, block2_wrong_prev]);
        assert!(matches!(
            result,
            Err(ChainVerifyError::HashMismatch {
                sequence_number: 2,
                ..
            })
        ));
    }
}
