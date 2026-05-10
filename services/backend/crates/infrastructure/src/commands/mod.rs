//! `wna-backend` の CLI サブコマンド群
//!
//! 対応 §: ロードマップ §14.2
//!
//! 各サブコマンドは `main.rs` から `clap` 経由でディスパッチされる。
//! 1 ファイル ≤ 500 行制約のため、データ仕様は `seed_data` に分離する。

// シードコマンド本体
pub mod seed;
// シードのデータ仕様（preset 別の固定値）
pub mod seed_data;
