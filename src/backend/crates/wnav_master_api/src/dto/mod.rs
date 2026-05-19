// Request/Response DTO 定義
//
// 全 API エンドポイントの入出力型を定義する。
// serde の Serialize/Deserialize と utoipa の ToSchema を付与して
// JSON ⇔ Rust 型変換と OpenAPI スキーマ自動生成を両立させる。
// reworks は terminal-api に移管済み（バイナリ割当ミス修正）。

pub mod alerts;
pub mod auth;
pub mod capas;
pub mod health;
pub mod iqc;
pub mod master;
pub mod metrics;
pub mod nonconformities;
pub mod ops;
pub mod public_config;
pub mod reports;
pub mod scraps;
pub mod trace;
pub mod work_assignments;
