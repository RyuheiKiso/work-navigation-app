// wnav_hash_chain クレートのエラー型定義モジュール
// thiserror を使用してエラーを構造的に表現する。

/// ハッシュチェーン計算・検証のエラー型。
#[derive(Debug, thiserror::Error)]
pub enum HashChainError {
    /// hex 文字列の長さが不正な場合のエラー。
    #[error("hex 文字列の長さが不正です: 期待値={expected}, 実際値={actual}")]
    InvalidHexLength {
        /// 期待する長さ（通常 64）
        expected: usize,
        /// 実際の長さ
        actual: usize,
    },

    /// 有効な hex 文字列でない場合のエラー。
    #[error("無効な hex 文字列です: {0}")]
    InvalidHex(String),
}
