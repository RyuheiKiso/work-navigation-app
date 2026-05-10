//! メモリ Idempotency ストア（開発・テスト用）
//!
//! 対応 §: ロードマップ §10.3.1 §27 F-005
//!
//! `wna_usecase::OrderRepository` をメモリ内で実装する。
//! 24h 窓は時刻を保持して expire 判定するが、本実装は単純な `HashMap<key, instant>` で行う。
//! 本番環境では PostgreSQL／Redis 実装に差し替える。

// ドメイン
use wna_domain::{IdempotencyKey, ProductionOrder};
// ユースケース
use wna_usecase::OrderRepository;
// 標準
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// 24h 窓の長さ
const WINDOW: Duration = Duration::from_secs(24 * 60 * 60);

/// エラー
#[derive(Debug)]
pub struct MemoryError;

// Display
impl std::fmt::Display for MemoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // 表示
        write!(f, "memory idempotency store error")
    }
}

// Error
impl std::error::Error for MemoryError {}

/// メモリ Idempotency ストア
#[derive(Default)]
pub struct MemoryOrderRepository {
    // キー → 観測時刻
    keys: Mutex<HashMap<String, Instant>>,
    // 永続化された順序情報（FIFO）
    orders: Mutex<Vec<ProductionOrder>>,
}

impl MemoryOrderRepository {
    /// 新しいストアを返す
    #[must_use]
    pub fn new() -> Self {
        // デフォルトを返す
        Self::default()
    }

    /// 期限切れキーを掃除する（テストで明示的に呼ぶ）
    pub fn purge_expired(&self) {
        // 現在時刻
        let now = Instant::now();
        // ロックを取得して filter
        let mut map = self.keys.lock().expect("lock");
        // 24h 超のエントリを削除
        map.retain(|_, ts| now.duration_since(*ts) < WINDOW);
    }
}

impl OrderRepository for MemoryOrderRepository {
    type Error = MemoryError;

    async fn key_seen_within_window(
        &self,
        key: &IdempotencyKey,
    ) -> Result<bool, Self::Error> {
        // 現在時刻
        let now = Instant::now();
        // ロック取得
        let map = self.keys.lock().expect("lock");
        // 同一キーがあり、かつ 24h 窓内なら true
        Ok(map
            .get(key.as_str())
            .map(|ts| now.duration_since(*ts) < WINDOW)
            .unwrap_or(false))
    }

    async fn store(&self, order: &ProductionOrder) -> Result<(), Self::Error> {
        // 現在時刻
        let now = Instant::now();
        // キーを記録
        self.keys
            .lock()
            .expect("lock")
            .insert(order.idempotency_key().as_str().to_string(), now);
        // 順序情報を追加
        self.orders.lock().expect("lock").push(order.clone());
        // 正常
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
    use wna_domain::{ItemCode, OrderId, Quantity};

    // 補助
    fn fresh_order(key: &str) -> ProductionOrder {
        // 妥当な値
        let id = OrderId::new("o-1").expect("valid");
        let item = ItemCode::new("ITEM-1").expect("valid");
        let q = Quantity::from_u64(5);
        let k = IdempotencyKey::new(key).expect("valid");
        // 構築
        ProductionOrder::create(id, item, q, k).expect("valid")
    }

    // 1 件目の受領は seen=false → store → seen=true
    #[tokio::test]
    async fn first_store_then_seen() {
        // ストア
        let store = MemoryOrderRepository::new();
        // 順序情報
        let order = fresh_order("k-001");
        // 初回は seen=false
        assert!(!store
            .key_seen_within_window(order.idempotency_key())
            .await
            .expect("ok"));
        // 保存
        store.store(&order).await.expect("ok");
        // 2 度目は seen=true
        assert!(store
            .key_seen_within_window(order.idempotency_key())
            .await
            .expect("ok"));
    }

    // 異なるキーは互いに独立
    #[tokio::test]
    async fn distinct_keys_are_independent() {
        // ストア
        let store = MemoryOrderRepository::new();
        // 1 件目
        store.store(&fresh_order("k-001")).await.expect("ok");
        // 別キーは未観測
        let k = IdempotencyKey::new("k-002").expect("valid");
        assert!(!store.key_seen_within_window(&k).await.expect("ok"));
    }
}
