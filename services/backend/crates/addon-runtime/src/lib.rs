//! work-navigation-app アドオンランタイム
//!
//! 対応 §: ロードマップ §17.4 §17.5 §17.6 §19.3 §27 F-004 §29 R-008
//!
//! capability ベース権限（§17.4）と CPU／メモリ／実行時間制限（§17.5）、
//! 署名検証（§17.6 §19.3）を強制する。実行基盤は Wasmtime（feature = "wasmtime-runtime"）。

// 子モジュール
pub mod manifest;
pub mod limits;
pub mod capability_check;
// アドオン署名検証（§17.6 §19.3）
pub mod signature;
// Wasmtime 実体ホスト（feature 有効時のみコンパイル）
#[cfg(feature = "wasmtime-runtime")]
pub mod wasmtime_host;

// 代表型を再エクスポート
pub use capability_check::{check_required, CapabilityViolation};
pub use limits::{ResourceLimits, ResourceLimitsViolation};
pub use manifest::{AddonManifest, ManifestError};
pub use signature::{
    AddonSignatureVerifier, NoopVerifier, SignatureVerificationError, StrictPolicyVerifier,
};
#[cfg(feature = "wasmtime-runtime")]
pub use wasmtime_host::{WasmtimeAddonHost, WasmtimeHostError};
