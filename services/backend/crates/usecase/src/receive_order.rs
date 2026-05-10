//! 順序情報受領ユースケース（Idempotency-Key 重複排除）
//!
//! 対応 §: ロードマップ §10.3 §10.3.1 §10.3.2 §27 F-005
//!
//! 基幹システムから生産順序を受領し、24h 重複排除窓を経て永続化する。
//! 重複排除は `OrderRepository` 実装側に責務を委譲し、ユースケースは決定木のみを担う。

// ドメイン依存
use wna_domain::{
    IdempotencyKey, ProductionOrder, ProductionOrderError,
};

// =====================================================================
// OrderRepository（trait）
// =====================================================================

/// 順序情報リポジトリ
pub trait OrderRepository: Send + Sync {
    /// 実装固有エラー
    type Error: std::error::Error + Send + Sync + 'static;

    /// Idempotency-Key が 24h 窓内に既に観測されているか
    fn key_seen_within_window(
        &self,
        key: &IdempotencyKey,
    ) -> impl std::future::Future<Output = Result<bool, Self::Error>> + Send;

    /// 順序情報を永続化する
    ///
    /// 同時に Idempotency-Key を 24h 窓に登録する。
    fn store(
        &self,
        order: &ProductionOrder,
    ) -> impl std::future::Future<Output = Result<(), Self::Error>> + Send;
}

// =====================================================================
// ReceiveOrderCommand
// =====================================================================

/// 順序情報受領コマンド
#[derive(Debug, Clone)]
pub struct ReceiveOrderCommand {
    /// 受領する順序情報
    pub order: ProductionOrder,
}

// =====================================================================
// ReceiveOrderError
// =====================================================================

/// 順序情報受領のエラー
#[derive(Debug)]
pub enum ReceiveOrderError<E>
where
    E: std::error::Error + Send + Sync + 'static,
{
    /// ドメイン規則違反（例: 数量ゼロ）
    Domain(ProductionOrderError),
    /// 24h 窓内の重複（§10.3.1）
    Duplicate,
    /// リポジトリ層のエラー
    Repository(E),
}

// Display 実装
impl<E> std::fmt::Display for ReceiveOrderError<E>
where
    E: std::error::Error + Send + Sync + 'static,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // バリアント別に分岐
        match self {
            ReceiveOrderError::Domain(e) => write!(f, "ドメイン規則違反: {e}"),
            ReceiveOrderError::Duplicate => {
                write!(f, "Idempotency-Key が 24h 窓内に重複しています")
            }
            ReceiveOrderError::Repository(e) => write!(f, "リポジトリエラー: {e}"),
        }
    }
}

// Error 実装
impl<E> std::error::Error for ReceiveOrderError<E>
where
    E: std::error::Error + Send + Sync + 'static,
{
}

// =====================================================================
// ReceiveOrderUseCase
// =====================================================================

/// 順序情報受領ユースケース
pub struct ReceiveOrderUseCase<R: OrderRepository> {
    /// 注入される Repository
    repository: R,
}

impl<R: OrderRepository> ReceiveOrderUseCase<R> {
    /// コンストラクタ
    pub const fn new(repository: R) -> Self {
        // フィールドを保持
        Self { repository }
    }

    /// コマンドを実行する
    ///
    /// # Errors
    /// 重複検出時は `Duplicate`、その他はリポジトリ／ドメインエラー。
    pub async fn execute(
        &self,
        cmd: ReceiveOrderCommand,
    ) -> Result<(), ReceiveOrderError<R::Error>> {
        // Idempotency-Key の重複チェック（24h 窓）
        let seen = self
            .repository
            .key_seen_within_window(cmd.order.idempotency_key())
            .await
            .map_err(ReceiveOrderError::Repository)?;
        // 重複なら拒否（§10.3.1）
        if seen {
            return Err(ReceiveOrderError::Duplicate);
        }
        // 永続化（実装側で 24h 窓への登録もアトミックに行うこと）
        self.repository
            .store(&cmd.order)
            .await
            .map_err(ReceiveOrderError::Repository)?;
        // 正常終了
        Ok(())
    }
}

// =====================================================================
// 単体テスト
// =====================================================================

#[cfg(test)]
mod tests {
    // テスト対象
    use super::*;
    // ドメイン
    use std::sync::Mutex;
    use wna_domain::{ItemCode, OrderId, Quantity};

    // メモリ Repository（テスト用）
    #[derive(Default)]
    struct MemRepo {
        // 観測済みキー
        keys: Mutex<Vec<String>>,
        // 永続化された順序情報
        orders: Mutex<Vec<ProductionOrder>>,
    }

    // テスト用エラー
    #[derive(Debug)]
    struct E;
    impl std::fmt::Display for E {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            // 表示
            write!(f, "mem error")
        }
    }
    impl std::error::Error for E {}

    impl OrderRepository for MemRepo {
        type Error = E;
        async fn key_seen_within_window(
            &self,
            key: &IdempotencyKey,
        ) -> Result<bool, Self::Error> {
            // 観測済み判定
            let seen = self
                .keys
                .lock()
                .expect("lock")
                .iter()
                .any(|k| k == key.as_str());
            Ok(seen)
        }
        async fn store(&self, order: &ProductionOrder) -> Result<(), Self::Error> {
            // キーを記録
            self.keys
                .lock()
                .expect("lock")
                .push(order.idempotency_key().as_str().to_string());
            // 順序情報を保存
            self.orders.lock().expect("lock").push(order.clone());
            Ok(())
        }
    }

    // 補助: 妥当な ProductionOrder を作る
    fn fresh_order(key: &str) -> ProductionOrder {
        // 妥当な値
        let id = OrderId::new("o-1").expect("valid");
        let item = ItemCode::new("ITEM-1").expect("valid");
        let q = Quantity::from_u64(5);
        let k = IdempotencyKey::new(key).expect("valid");
        // 構築
        ProductionOrder::create(id, item, q, k).expect("valid")
    }

    // 1 件目は受領される
    #[tokio::test]
    async fn receives_first_occurrence() {
        // メモリリポジトリ
        let repo = MemRepo::default();
        // ユースケース
        let uc = ReceiveOrderUseCase::new(repo);
        // 1 件目
        let cmd = ReceiveOrderCommand {
            order: fresh_order("k-001"),
        };
        // 実行
        let r = uc.execute(cmd).await;
        // OK
        assert!(r.is_ok());
    }

    // 2 件目（同キー）は重複として拒否される
    #[tokio::test]
    async fn rejects_duplicate_within_window() {
        // メモリリポジトリ
        let repo = MemRepo::default();
        let uc = ReceiveOrderUseCase::new(repo);
        // 1 件目を受領
        uc.execute(ReceiveOrderCommand {
            order: fresh_order("k-001"),
        })
        .await
        .expect("first ok");
        // 2 件目（同キー）
        let r = uc
            .execute(ReceiveOrderCommand {
                order: fresh_order("k-001"),
            })
            .await;
        // 重複エラー
        assert!(matches!(r, Err(ReceiveOrderError::Duplicate)));
    }
}
