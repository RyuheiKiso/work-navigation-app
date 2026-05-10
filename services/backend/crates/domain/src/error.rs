//! ドメイン層のエラー型
//!
//! 対応 §: ロードマップ §9.4 §11.6
//!
//! 境界エラーは外側 crate（adapter／presentation）で `thiserror` を使って合成するが、
//! ドメイン層自体は標準ライブラリのみに依存するため、自前の列挙型で表現する。

// std::fmt をローカル import（インテンプレート展開用）
use core::fmt;

/// ドメイン規則違反を表す列挙型
///
/// 以下は §3 ドメインモデルおよび §10.6 同期戦略の不変条件に対応する。
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum DomainError {
    /// 識別子の形式が不正（空・規定外）
    InvalidIdentifier(&'static str),
    /// 開始条件未充足のまま開始しようとした（§3.1.1）
    PreconditionNotSatisfied,
    /// 完了条件未充足のまま完了させようとした（§3.1.1）
    CompletionCriteriaNotMet,
    /// すでに完了／取消済の作業に対する遷移要求
    InvalidStateTransition {
        /// 現在状態
        current: &'static str,
        /// 試行された遷移
        attempted: &'static str,
    },
    /// Lamport timestamp の単調性違反（§10.6.1 / INV-08）
    LamportNonMonotonic,
}

// 標準の Display 実装（境界層でログ／エラーメッセージへ流す用途）
impl fmt::Display for DomainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // バリアントごとに人間可読なメッセージを出す（§20.1 「人を責めない」原則）
        match self {
            // 識別子エラーは原因タグ付きで返す
            DomainError::InvalidIdentifier(tag) => {
                write!(f, "識別子の形式が不正です: {tag}")
            }
            // 前提条件未充足
            DomainError::PreconditionNotSatisfied => {
                write!(f, "開始条件が満たされていません")
            }
            // 完了条件未充足
            DomainError::CompletionCriteriaNotMet => {
                write!(f, "完了条件が満たされていません")
            }
            // 状態遷移不正
            DomainError::InvalidStateTransition { current, attempted } => {
                write!(f, "不正な状態遷移: {current} -> {attempted}")
            }
            // Lamport 単調性違反
            DomainError::LamportNonMonotonic => {
                write!(f, "Lamport タイムスタンプの単調性に違反しています")
            }
        }
    }
}

// std::error::Error を実装することで anyhow／thiserror に整合させる
impl std::error::Error for DomainError {}
