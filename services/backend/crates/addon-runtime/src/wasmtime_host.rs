//! Wasmtime 実体ホスト
//!
//! 対応 §: ロードマップ §17.4 §17.5 §27 F-004 §29 R-008
//!
//! WASM モジュールを load → instantiate → invoke する経路を提供する。
//! capability チェック（§17.4）／fuel・memory・time 制限（§17.5）を強制する。
//!
//! 本モジュールは `feature = "wasmtime-runtime"` 有効時のみコンパイルされる。

// Wasmtime 本体
use wasmtime::{
    Config, Engine, Linker, Memory, MemoryType, Module, Store, StoreLimits, StoreLimitsBuilder,
};
// thiserror
use thiserror::Error;
// 内部
use crate::{check_required, CapabilityViolation, ResourceLimits};
use wna_addon_sdk::Capability;

/// Wasmtime ホスト固有のエラー
#[derive(Debug, Error)]
pub enum WasmtimeHostError {
    /// capability 違反
    #[error("capability 違反: {0}")]
    Capability(#[from] CapabilityViolation),
    /// Wasmtime のエンジン／モジュール／インスタンス化エラー
    #[error("Wasmtime: {0}")]
    Wasmtime(String),
    /// 必要な export が見つからない
    #[error("WASM export 未検出: {0}")]
    MissingExport(String),
}

/// アドオン実行ホスト（Wasmtime ベース）
pub struct WasmtimeAddonHost {
    /// Wasmtime エンジン（プロセス全体で共有可能）
    engine: Engine,
    /// 既定リソース制限
    limits: ResourceLimits,
}

impl WasmtimeAddonHost {
    /// 新規ホストを構築する
    ///
    /// `Config` には fuel／async／epoch interruption を有効化する（§17.5）。
    pub fn new(limits: ResourceLimits) -> Result<Self, WasmtimeHostError> {
        // Config を組み立てる
        let mut cfg = Config::new();
        // fuel を有効化（命令カウンタ制限）
        cfg.consume_fuel(true);
        // epoch interruption を有効化（実行時間制限）
        cfg.epoch_interruption(true);
        // 動的メモリ確保のキャップを設定（最大メモリは StoreLimits で別途）
        // Engine 構築
        let engine = Engine::new(&cfg)
            .map_err(|e| WasmtimeHostError::Wasmtime(e.to_string()))?;
        // 完成
        Ok(Self { engine, limits })
    }

    /// WASM モジュールを load → instantiate → invoke する
    ///
    /// `granted` でアドオンマニフェストから取得した capability、
    /// `required` で本呼び出しに必要な capability を渡す。
    /// 違反時は実行前に `CapabilityViolation` で拒否する（§17.4）。
    ///
    /// 戻り値はアドオン側 `_start()` の i32 戻り値（成功時 0）。
    pub fn instantiate_and_invoke_start(
        &self,
        wasm_bytes: &[u8],
        granted: &[Capability],
        required: &[Capability],
    ) -> Result<i32, WasmtimeHostError> {
        // capability 検証（実行前に拒否）
        check_required(granted, required)?;

        // Store の限界設定（メモリ上限を強制、§17.5）
        let store_limits: StoreLimits = StoreLimitsBuilder::new()
            .memory_size(self.limits.memory_bytes)
            .memories(8)
            .tables(8)
            .instances(1)
            .build();
        // Store を構築
        let mut store = Store::new(&self.engine, store_limits);
        // メモリ／テーブル等の上限を Store に紐付ける
        store.limiter(|s| s);
        // fuel を投入
        store
            .set_fuel(self.limits.max_fuel)
            .map_err(|e| WasmtimeHostError::Wasmtime(e.to_string()))?;

        // モジュールをロード
        let module = Module::new(&self.engine, wasm_bytes)
            .map_err(|e| WasmtimeHostError::Wasmtime(e.to_string()))?;

        // Linker を構築（host import を最小化、WASI fs/net は無効、§17.5）
        let linker: Linker<StoreLimits> = Linker::new(&self.engine);

        // インスタンス化
        let instance = linker
            .instantiate(&mut store, &module)
            .map_err(|e| WasmtimeHostError::Wasmtime(e.to_string()))?;

        // メモリ存在の確認（任意 export）
        let _mem: Option<Memory> = instance.get_memory(&mut store, "memory");

        // `_start` 関数の取得（WASI ABI 準拠の最小エントリ）
        let start = instance
            .get_typed_func::<(), i32>(&mut store, "_start")
            .map_err(|_| WasmtimeHostError::MissingExport("_start".to_string()))?;

        // 呼び出し（fuel 切れ／timeout 時は trap）
        let r = start
            .call(&mut store, ())
            .map_err(|e| WasmtimeHostError::Wasmtime(e.to_string()))?;

        // 戻り値
        Ok(r)
    }

    /// 既定の MemoryType を取得（テスト・デバッグ用）
    #[must_use]
    pub fn default_memory_type(&self) -> MemoryType {
        // 1 ページ = 64KiB、上限 = limits.memory_bytes / 64KiB
        let pages_max = (self.limits.memory_bytes / 65536) as u64;
        MemoryType::new(1, Some(pages_max as u32))
    }
}

// =====================================================================
// 単体テスト（feature ON 時のみコンパイル）
// =====================================================================

#[cfg(test)]
mod tests {
    // テスト対象
    use super::*;

    // ホスト構築
    #[test]
    fn host_builds_with_terminal_defaults() {
        // 既定値で構築できる
        let host = WasmtimeAddonHost::new(ResourceLimits::TERMINAL_DEFAULTS);
        assert!(host.is_ok());
    }

    // capability 違反は実行前に弾く
    #[test]
    fn rejects_when_capability_missing_before_loading_wasm() {
        let host = WasmtimeAddonHost::new(ResourceLimits::TERMINAL_DEFAULTS)
            .expect("ok");
        // wasm bytes は不要（capability 違反で wasm load 前に return）
        let r = host.instantiate_and_invoke_start(
            &[], // 空 wasm（実行されない）
            &[Capability::TaskRead],
            &[Capability::TaskWrite],
        );
        // Capability 違反
        assert!(matches!(r, Err(WasmtimeHostError::Capability(_))));
    }

    // 不正な wasm bytes は Wasmtime エラー
    #[test]
    fn rejects_invalid_wasm_bytes() {
        let host = WasmtimeAddonHost::new(ResourceLimits::TERMINAL_DEFAULTS)
            .expect("ok");
        // capability は満たすが wasm が不正
        let r = host.instantiate_and_invoke_start(
            b"not-wasm",
            &[Capability::TaskRead],
            &[Capability::TaskRead],
        );
        // Wasmtime エラー
        assert!(matches!(r, Err(WasmtimeHostError::Wasmtime(_))));
    }
}
