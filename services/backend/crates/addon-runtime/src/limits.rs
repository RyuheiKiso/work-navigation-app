//! アドオン実行リソース制限
//!
//! 対応 §: ロードマップ §17.5 §27 F-004 §29 R-008

// thiserror
use thiserror::Error;

/// CPU／メモリ／実行時間の制限値
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResourceLimits {
    /// メモリ上限（bytes）
    pub memory_bytes: usize,
    /// 実行時間上限（ms）
    pub max_execution_ms: u64,
    /// fuel（Wasmtime の命令カウンタ）上限
    pub max_fuel: u64,
}

impl ResourceLimits {
    /// §17.5 既定値（端末側）
    pub const TERMINAL_DEFAULTS: Self = Self {
        // 64 MB
        memory_bytes: 64 * 1024 * 1024,
        // 100ms
        max_execution_ms: 100,
        // 1000 万命令
        max_fuel: 10_000_000,
    };

    /// §17.5 既定値（サーバ側）
    pub const SERVER_DEFAULTS: Self = Self {
        // 128 MB
        memory_bytes: 128 * 1024 * 1024,
        // 500ms
        max_execution_ms: 500,
        // 5000 万命令
        max_fuel: 50_000_000,
    };
}

/// 制限超過エラー
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ResourceLimitsViolation {
    /// メモリ超過
    #[error("メモリ上限を超過しました: 要求 {requested} bytes, 上限 {limit} bytes")]
    Memory { requested: usize, limit: usize },
    /// 実行時間超過
    #[error("実行時間上限を超過しました: 経過 {elapsed_ms}ms, 上限 {limit_ms}ms")]
    Time { elapsed_ms: u64, limit_ms: u64 },
    /// fuel 超過
    #[error("命令数上限を超過しました（fuel）: 消費 {consumed}, 上限 {limit}")]
    Fuel { consumed: u64, limit: u64 },
}

// =====================================================================
// 単体テスト
// =====================================================================

#[cfg(test)]
mod tests {
    // テスト対象
    use super::*;

    // 既定値が想定通り
    #[test]
    fn terminal_defaults_match_spec() {
        // §17.5 既定値
        let l = ResourceLimits::TERMINAL_DEFAULTS;
        // メモリ 64MB
        assert_eq!(l.memory_bytes, 64 * 1024 * 1024);
        // 実行時間 100ms
        assert_eq!(l.max_execution_ms, 100);
    }

    // サーバ既定値
    #[test]
    fn server_defaults_have_higher_limits() {
        // 端末よりサーバの方が上限が高い
        let t = ResourceLimits::TERMINAL_DEFAULTS;
        let s = ResourceLimits::SERVER_DEFAULTS;
        assert!(s.memory_bytes >= t.memory_bytes);
        assert!(s.max_execution_ms >= t.max_execution_ms);
    }
}
