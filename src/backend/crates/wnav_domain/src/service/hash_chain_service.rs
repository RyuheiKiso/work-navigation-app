// ハッシュチェーンサービス（wnav_hash_chain クレートのラッパー）
// WorkEvent・IQC・リワーク・処置判定のコンテンツハッシュ計算と挿入を担う。
// BAT-001 週次検証の呼び出し口を提供する。

use std::sync::Arc;

use serde_json::Value;
use uuid::Uuid;

use crate::error::DomainError;
use crate::repository::{HashChainBlock, HashChainBlockRepository};

/// ハッシュチェーンサービス。
/// wnav_hash_chain クレートのラッパーとして、
/// ドメイン層から一貫したハッシュチェーン操作 API を提供する。
pub struct HashChainService {
    /// ハッシュチェーンブロックリポジトリ
    pub block_repo: Arc<dyn HashChainBlockRepository>,
}

impl HashChainService {
    /// ハッシュチェーンブロックを計算して INSERT する。
    /// CompleteStep トランザクション内で同期的に呼び出す（ALG-002）。
    pub async fn compute_and_insert_block(
        &self,
        case_id: Uuid,
        event_id: Uuid,
        payload: &Value,
    ) -> Result<HashChainBlock, DomainError> {
        // 最新ブロックを取得して prev_hash とシーケンス番号を決定する
        let latest_block = self.block_repo.find_latest(case_id).await?;
        let prev_hash = latest_block
            .as_ref()
            .map_or(wnav_hash_chain::GENESIS_PREV_HASH, |b| b.block_hash);

        // 次のシーケンス番号を決定する
        let sequence_number = latest_block.as_ref().map_or(1, |b| b.sequence_number + 1);

        // コンテンツハッシュを計算する（canonical JSON → SHA-256）
        let canonical = wnav_hash_chain::canonical_json(payload);
        let content_hash = wnav_hash_chain::compute_content_hash(&canonical);

        // チェーンハッシュを計算する（SHA-256(prev_hash || content_hash)）
        let block_hash = wnav_hash_chain::compute_chain_hash(&prev_hash, &content_hash);

        let block = HashChainBlock {
            block_id: event_id,
            case_id,
            sequence_number,
            prev_block_hash: prev_hash,
            content_hash,
            block_hash,
            created_at: chrono::Utc::now(),
        };

        self.block_repo.insert(block.clone()).await?;

        tracing::debug!(
            case_id = %case_id,
            event_id = %event_id,
            sequence_number,
            "ハッシュチェーンブロックを挿入しました"
        );

        Ok(block)
    }

    /// BAT-001 週次検証の呼び出し口。
    /// 指定した case_id のチェーン連続性を検証する。
    pub async fn verify_chain(&self, case_id: Uuid) -> Result<ChainVerifyResult, DomainError> {
        let blocks = self.block_repo.list_by_case(case_id).await?;

        let block_count = blocks.len();

        if blocks.is_empty() {
            return Ok(ChainVerifyResult {
                case_id,
                block_count: 0,
                is_valid: true,
                broken_at_sequence: None,
                error_message: None,
            });
        }

        // wnav_hash_chain の ChainBlock 型に変換する
        let chain_blocks: Vec<wnav_hash_chain::ChainBlock> = blocks
            .iter()
            .map(|b| wnav_hash_chain::ChainBlock {
                id: b.block_id,
                case_id: b.case_id,
                sequence_number: b.sequence_number,
                prev_block_hash: b.prev_block_hash,
                content_hash: b.content_hash,
                block_hash: b.block_hash,
                created_at: b.created_at,
            })
            .collect();

        // wnav_hash_chain クレートでチェーン検証する
        match wnav_hash_chain::verify_chain(&chain_blocks) {
            Ok(()) => Ok(ChainVerifyResult {
                case_id,
                block_count,
                is_valid: true,
                broken_at_sequence: None,
                error_message: None,
            }),
            Err(e) => {
                tracing::error!(case_id = %case_id, error = %e, "ハッシュチェーン破断を検知しました");
                Ok(ChainVerifyResult {
                    case_id,
                    block_count,
                    is_valid: false,
                    broken_at_sequence: extract_broken_sequence(&e),
                    error_message: Some(e.to_string()),
                })
            }
        }
    }
}

/// チェーン検証結果。
#[derive(Debug)]
pub struct ChainVerifyResult {
    /// 検証した case_id
    pub case_id: Uuid,
    /// ブロック数
    pub block_count: usize,
    /// 検証成功かどうか
    pub is_valid: bool,
    /// 破断が検出されたシーケンス番号（破断なし時は None）
    pub broken_at_sequence: Option<i64>,
    /// エラーメッセージ（正常時は None）
    pub error_message: Option<String>,
}

/// ChainVerifyError からシーケンス番号を抽出するヘルパー。
fn extract_broken_sequence(error: &wnav_hash_chain::ChainVerifyError) -> Option<i64> {
    match error {
        wnav_hash_chain::ChainVerifyError::HashMismatch {
            sequence_number, ..
        } => Some(*sequence_number),
        wnav_hash_chain::ChainVerifyError::SequenceGap {
            sequence_number, ..
        } => Some(*sequence_number),
    }
}
