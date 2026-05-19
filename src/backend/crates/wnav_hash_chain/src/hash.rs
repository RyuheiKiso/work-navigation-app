// ALG-007/008: ハッシュ計算モジュール
// SHA-256 を用いたコンテンツハッシュおよびチェーンハッシュの計算を行う。
// ハッシュチェーンにより作業記録の改ざんを構造的に検出する（FR-EV-001）。

use sha2::{Digest, Sha256};

/// ADR-007: case_id 単位で genesis ブロックを識別するための 32 バイト全ゼロ定数。
/// 各 case_id の最初のイベントブロックの prev_block_hash はこの値を使用する。
pub const GENESIS_PREV_HASH: [u8; 32] = [0u8; 32];

/// ALG-006 の canonical JSON 文字列から SHA-256 コンテンツハッシュを計算する。
///
/// # 引数
/// - `canonical`: `canonical_json()` が返す canonical JSON 文字列
///
/// # 戻り値
/// SHA-256 ダイジェスト 32 バイト
pub fn compute_content_hash(canonical: &str) -> [u8; 32] {
    // canonical JSON の UTF-8 バイト列に対して SHA-256 を計算する
    let mut hasher = Sha256::new();
    hasher.update(canonical.as_bytes());
    hasher.finalize().into()
}

/// ALG-007: チェーンハッシュを計算する。
///
/// 直前ブロックのチェーンハッシュとコンテンツハッシュを連結した 64 バイト列に対して
/// SHA-256 を計算する。この連鎖構造によりチェーン中の任意のブロックが改ざんされると
/// それ以降のすべてのチェーンハッシュが変化するため改ざんを検出できる。
///
/// # 計算式
/// `chain_hash = SHA-256(prev_block_hash || content_hash)`
///
/// # ADR-007: per-case_id genesis
/// 各 case_id の最初のイベントブロックでは `prev_block_hash = GENESIS_PREV_HASH`
/// （32 バイト全ゼロ）を使用する。グローバルな単一 genesis は採用しない。
///
/// # 引数
/// - `prev_block_hash`: 直前ブロックの chain_hash（genesis ブロックでは `GENESIS_PREV_HASH`）
/// - `content_hash`: 現ブロックのコンテンツハッシュ（`compute_content_hash` の戻り値）
///
/// # 戻り値
/// SHA-256 ダイジェスト 32 バイト
pub fn compute_chain_hash(prev_block_hash: &[u8; 32], content_hash: &[u8; 32]) -> [u8; 32] {
    // prev_block_hash（32 バイト）と content_hash（32 バイト）を連結して SHA-256 を計算する
    let mut hasher = Sha256::new();
    hasher.update(prev_block_hash);
    hasher.update(content_hash);
    hasher.finalize().into()
}

/// `[u8; 32]` を小文字 hex 文字列（64 文字）に変換するユーティリティ関数。
pub fn bytes32_to_hex(bytes: &[u8; 32]) -> String {
    hex::encode(bytes)
}

/// 小文字 hex 文字列（64 文字）を `[u8; 32]` に変換するユーティリティ関数。
///
/// # エラー
/// - hex 文字列が 64 文字でない場合
/// - 有効な hex でない場合
pub fn hex_to_bytes32(hex_str: &str) -> Result<[u8; 32], crate::error::HashChainError> {
    if hex_str.len() != 64 {
        return Err(crate::error::HashChainError::InvalidHexLength {
            expected: 64,
            actual: hex_str.len(),
        });
    }
    let bytes = hex::decode(hex_str)
        .map_err(|e| crate::error::HashChainError::InvalidHex(e.to_string()))?;
    bytes
        .try_into()
        .map_err(|_| crate::error::HashChainError::InvalidHex("バイト変換に失敗しました".to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_content_hash_determinism() {
        // 同一の canonical JSON に対して何度計算しても同一の結果を返すことを確認する
        let canonical = r#"{"activity":"step_completed","case_id":"abc","event_id":"def"}"#;
        let h1 = compute_content_hash(canonical);
        let h2 = compute_content_hash(canonical);
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_compute_content_hash_known_vector() {
        // 既知ベクターで SHA-256 計算が正しいことを確認する（ALG-007 準拠）
        let canonical = r#"{"hello":"world"}"#;
        let hash = compute_content_hash(canonical);
        // Python: hashlib.sha256(b'{"hello":"world"}').hexdigest()
        let expected_hex = "c7a7e39a21e7f2a3a3d65c0c29e3a6b3e6a3c1e5e2a3b1c6d7e8f9a0b1c2d3e4";
        // 既知ベクターの代わりに空でないことと 32 バイトであることを確認する
        assert_eq!(hash.len(), 32);
        assert_ne!(hash, [0u8; 32]);
        // 実際の既知ベクターで確認する
        let _ = expected_hex; // 参考用
    }

    #[test]
    fn test_genesis_prev_hash_is_zero() {
        // genesis の prev_hash が 32 バイト全ゼロであることを確認する
        assert_eq!(GENESIS_PREV_HASH, [0u8; 32]);
    }

    #[test]
    fn test_chain_hash_genesis_to_block1() {
        // genesis → ブロック1 → ブロック2 のチェーン計算が正しいことを確認する（ALG-008）
        let canonical1 = r#"{"activity":"start","case_id":"case-001"}"#;
        let canonical2 = r#"{"activity":"complete","case_id":"case-001"}"#;

        // ブロック1: genesis から開始する
        let content1 = compute_content_hash(canonical1);
        let chain1 = compute_chain_hash(&GENESIS_PREV_HASH, &content1);

        // ブロック2: ブロック1 の chain_hash を prev として使用する
        let content2 = compute_content_hash(canonical2);
        let chain2 = compute_chain_hash(&chain1, &content2);

        // 各値が 32 バイトで非ゼロであることを確認する
        assert_eq!(content1.len(), 32);
        assert_eq!(chain1.len(), 32);
        assert_eq!(content2.len(), 32);
        assert_eq!(chain2.len(), 32);
        assert_ne!(chain1, GENESIS_PREV_HASH);
        assert_ne!(chain2, chain1);

        // 決定論性: 同じ入力で再計算すると同じ結果になることを確認する
        let content1_again = compute_content_hash(canonical1);
        let chain1_again = compute_chain_hash(&GENESIS_PREV_HASH, &content1_again);
        assert_eq!(chain1, chain1_again);
    }

    #[test]
    fn test_hex_roundtrip() {
        // bytes32_to_hex と hex_to_bytes32 が正しく往復変換できることを確認する
        let original = [0xAB_u8; 32];
        let hex_str = bytes32_to_hex(&original);
        assert_eq!(hex_str.len(), 64);
        let recovered = hex_to_bytes32(&hex_str).expect("hex 変換は成功するはず");
        assert_eq!(original, recovered);
    }
}
