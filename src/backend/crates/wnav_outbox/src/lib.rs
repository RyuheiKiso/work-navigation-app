// wnav_outbox クレート（MOD-BE-006）
//
// Outbox Consumer（BAT-002）を提供する常駐 tokio タスクライブラリ。
// 本クレートは wnav_terminal_api バイナリのみが依存する（wnav_master_api は使用しない）。
//
// # 責務
// - PENDING 状態の outbox_events を PostgreSQL から取得して親機 API に配信する
// - SELECT ... FOR UPDATE SKIP LOCKED で複数インスタンス間の二重配信を防止する
// - 指数バックオフ（ALG-009）でリトライスケジュールを管理する
// - 最大リトライ超過または 4xx 非リトライ可能エラーで DLQ（dead_lettered）に移行する
// - shutdown シグナルを受信して安全に終了する
//
// # 起動方法
// ```rust
// // wnav_terminal_api main.rs での起動例
// let (shutdown_tx, shutdown_rx) = tokio::sync::broadcast::channel(1);
// let consumer = OutboxConsumer::from_config(pool, config)?;
// tokio::spawn(consumer.run(shutdown_rx));
// ```

// unsafe コードを禁止する（src/CLAUDE.md および src/backend/CLAUDE.md の必須要件）
#![forbid(unsafe_code)]
// 例外: doc コメントのリンク省略は許容
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
// 例外: モジュール名重複は許容
#![allow(clippy::module_name_repetitions)]
// 例外: must_use 警告は許容
#![allow(clippy::must_use_candidate)]

pub mod consumer;
pub mod error;
pub mod event_type;

// 主要な型を再エクスポートして使いやすくする
pub use consumer::{DispatchResult, OutboxConfig, OutboxConsumer, run_consumer};
pub use error::OutboxError;
pub use event_type::OutboxEventType;
