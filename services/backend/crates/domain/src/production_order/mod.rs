//! 順序情報（Production Order Sequence）ドメイン
//!
//! 対応 §: ロードマップ §10.3 §10.3.2 §10.3.6 §27 F-005 §28
//!
//! 基幹システム（MES／ERP）から受領する **生産順序** を表すモデル。
//! 本アプリはこのデータの「原典」ではなく、§10.3.6 RACI に従い
//! 受領・表示・実績フィードバックの責務を持つ。
//!
//! Idempotency-Key（§10.3.1）による重複排除はドメイン層で表現し、
//! 24h の保持期間は境界層（adapter）で実装する。
//!
//! 状態遷移:
//! ```text
//! [Released] --start--> [InProgress] --complete--> [Done]
//!     \                      \
//!      \--cancel--+           \--cancel--> [Cancelled]
//!                 v
//!            [Cancelled]
//! ```
//! Done と Cancelled は終端状態であり、以後の遷移は拒否される。

use core::fmt;

use crate::error::DomainError;

#[cfg(test)]
mod tests;

/// 順序情報 ID
///
/// 基幹システムが発行した文字列 ID をそのまま保持する。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct OrderId(String);

impl OrderId {
    /// 文字列から構築する
    ///
    /// # Errors
    /// 空または 256 文字超は不正。
    pub fn new(value: impl Into<String>) -> Result<Self, DomainError> {
        let v: String = value.into();
        if v.is_empty() {
            return Err(DomainError::InvalidIdentifier("OrderId が空です"));
        }
        if v.len() > 256 {
            return Err(DomainError::InvalidIdentifier("OrderId が長すぎます"));
        }
        Ok(Self(v))
    }

    /// 内部 &str を取得する
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// 品目コード値オブジェクト（§10.3.6 RACI: 基幹側原典）
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ItemCode(String);

impl ItemCode {
    /// 文字列から構築する
    ///
    /// # Errors
    /// 空または 64 文字超は不正。
    pub fn new(value: impl Into<String>) -> Result<Self, DomainError> {
        let v: String = value.into();
        if v.is_empty() {
            return Err(DomainError::InvalidIdentifier("ItemCode が空です"));
        }
        if v.len() > 64 {
            return Err(DomainError::InvalidIdentifier("ItemCode が長すぎます"));
        }
        Ok(Self(v))
    }

    /// 内部 &str を取得する
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// 数量値オブジェクト
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Quantity(u64);

impl Quantity {
    /// `u64` から構築する
    #[must_use]
    pub const fn from_u64(v: u64) -> Self {
        Self(v)
    }

    /// 内部値を取得する
    #[must_use]
    pub const fn value(self) -> u64 {
        self.0
    }
}

/// Idempotency-Key 値オブジェクト
///
/// 24h の重複排除窓は adapter 層で実装する。
/// 仕様: 1〜128 文字の ASCII 印字可能文字。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IdempotencyKey(String);

impl IdempotencyKey {
    /// 文字列から構築する
    ///
    /// # Errors
    /// 空／128 文字超／非 ASCII 印字可能の場合は不正。
    pub fn new(value: impl Into<String>) -> Result<Self, DomainError> {
        let v: String = value.into();
        if v.is_empty() {
            return Err(DomainError::InvalidIdentifier("IdempotencyKey が空です"));
        }
        if v.len() > 128 {
            return Err(DomainError::InvalidIdentifier("IdempotencyKey が長すぎます"));
        }
        if !v.chars().all(|c| c.is_ascii_graphic()) {
            return Err(DomainError::InvalidIdentifier(
                "IdempotencyKey に許容されない文字が含まれています",
            ));
        }
        Ok(Self(v))
    }

    /// 内部 &str を取得する
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for IdempotencyKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// 順序情報の状態
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderState {
    /// 受領済み（着手前）
    Released,
    /// 生産中
    InProgress,
    /// 完了（終端）
    Done,
    /// 取消（終端）
    Cancelled,
}

impl OrderState {
    /// 状態を表す文字列リテラル（エラーメッセージ用）
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            OrderState::Released => "Released",
            OrderState::InProgress => "InProgress",
            OrderState::Done => "Done",
            OrderState::Cancelled => "Cancelled",
        }
    }

    /// 終端状態か（以後の遷移を受け付けない）
    #[must_use]
    pub const fn is_terminal(self) -> bool {
        matches!(self, OrderState::Done | OrderState::Cancelled)
    }
}

/// 順序情報 Aggregate
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProductionOrder {
    id: OrderId,
    item: ItemCode,
    quantity: Quantity,
    idempotency_key: IdempotencyKey,
    state: OrderState,
}

impl ProductionOrder {
    /// 新しい `ProductionOrder` を Released 状態で構築する
    ///
    /// # Errors
    /// 数量がゼロの場合は §10.3.2 受領シナリオで意味を成さないため拒否する。
    pub fn create(
        id: OrderId,
        item: ItemCode,
        quantity: Quantity,
        idempotency_key: IdempotencyKey,
    ) -> Result<Self, ProductionOrderError> {
        if quantity.value() == 0 {
            return Err(ProductionOrderError::ZeroQuantity);
        }
        Ok(Self {
            id,
            item,
            quantity,
            idempotency_key,
            state: OrderState::Released,
        })
    }

    /// 永続化された値から再構築する（adapter 層から使う）
    #[must_use]
    pub const fn rehydrate(
        id: OrderId,
        item: ItemCode,
        quantity: Quantity,
        idempotency_key: IdempotencyKey,
        state: OrderState,
    ) -> Self {
        Self { id, item, quantity, idempotency_key, state }
    }

    /// 順序情報 ID を取得する
    #[must_use]
    pub const fn id(&self) -> &OrderId {
        &self.id
    }

    /// 品目コードを取得する
    #[must_use]
    pub const fn item(&self) -> &ItemCode {
        &self.item
    }

    /// 数量を取得する
    #[must_use]
    pub const fn quantity(&self) -> Quantity {
        self.quantity
    }

    /// Idempotency-Key を取得する
    #[must_use]
    pub const fn idempotency_key(&self) -> &IdempotencyKey {
        &self.idempotency_key
    }

    /// 状態を取得する
    #[must_use]
    pub const fn state(&self) -> OrderState {
        self.state
    }

    /// 生産を開始する（Released → InProgress）
    ///
    /// # Errors
    /// 状態が Released 以外なら遷移不正。
    pub fn start(&mut self) -> Result<(), DomainError> {
        if self.state != OrderState::Released {
            return Err(DomainError::InvalidStateTransition {
                current: self.state.label(),
                attempted: "InProgress",
            });
        }
        self.state = OrderState::InProgress;
        Ok(())
    }

    /// 完了する（InProgress → Done）
    ///
    /// # Errors
    /// 状態が InProgress 以外なら遷移不正。
    pub fn complete(&mut self) -> Result<(), DomainError> {
        if self.state != OrderState::InProgress {
            return Err(DomainError::InvalidStateTransition {
                current: self.state.label(),
                attempted: "Done",
            });
        }
        self.state = OrderState::Done;
        Ok(())
    }

    /// 取消する（非終端状態 → Cancelled）
    ///
    /// # Errors
    /// 既に終端 (Done / Cancelled) なら遷移不正。
    pub fn cancel(&mut self) -> Result<(), DomainError> {
        if self.state.is_terminal() {
            return Err(DomainError::InvalidStateTransition {
                current: self.state.label(),
                attempted: "Cancelled",
            });
        }
        self.state = OrderState::Cancelled;
        Ok(())
    }
}

/// 順序情報ドメイン特有のエラー
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProductionOrderError {
    /// 数量ゼロ
    ZeroQuantity,
    /// Idempotency-Key の重複（24h 窓内、§10.3.1）
    DuplicateKey,
}

impl fmt::Display for ProductionOrderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProductionOrderError::ZeroQuantity => write!(f, "数量がゼロです"),
            ProductionOrderError::DuplicateKey => {
                write!(f, "Idempotency-Key が重複しています（24h 窓内）")
            }
        }
    }
}

impl std::error::Error for ProductionOrderError {}
