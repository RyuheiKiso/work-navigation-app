//! work-navigation-app addon SDK (Rust)
//!
//! 対応 §: ロードマップ §17 §17.2 §17.3 §17.4 §17.5
//!
//! capability ベース API の **トレイト宣言** を提供する。
//! アドオン作者は本 crate を依存に追加し、ホスト側が注入する `Host` 実装を介して API を呼ぶ。
//!
//! # 例
//!
//! ```ignore
//! use wna_addon_sdk::{Host, AddonContext};
//!
//! fn run<H: Host>(host: &H, ctx: AddonContext) -> anyhow::Result<()> {
//!     let task = host.get_current_task()?;
//!     host.log("info", &format!("current: {}", task.id));
//!     Ok(())
//! }
//! ```

// 公開する子モジュール（API 領域定義）
pub mod api;

// 代表型を再エクスポート
pub use api::{
    AddonContext, AddonError, Capability, Host, NotificationChannel, TaskInfo,
};
