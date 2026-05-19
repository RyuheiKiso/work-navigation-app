// API ハンドラモジュール（wnav_master_api）
//
// 全エンドポイントのハンドラを各サブモジュールに配置する。
// reworks は terminal-api に移管済み（バイナリ割当ミス修正）。

pub mod alerts;
pub mod auth;
pub mod capas;
pub mod health;
pub mod iqc;
pub mod master;
pub mod nonconformities;
pub mod ops;
pub mod public_config;
pub mod reports;
pub mod scraps;
pub mod trace;
pub mod work_assignments;
