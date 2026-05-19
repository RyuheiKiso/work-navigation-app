// wnav_domain クレート
//
// 作業ナビゲーションシステムのドメインモデル・サービス Trait・リポジトリ Trait を提供する（MOD-BE-002）。
// 本クレートは Domain 層の唯一の実装であり、外部クレート（axum・sqlx 等）への依存を持たない。
//
// # アーキテクチャ
// - `model`: コアドメインエンティティ・値オブジェクト
// - `repository`: リポジトリ Trait（実装は `wnav_db` が担う）
// - `service`: ドメインサービス・アプリケーションサービス Trait
// - `events`: ドメインイベント（tokio broadcast channel で配信）
// - `rules`: ビジネスルール関数（BR-BUS-001〜046）
// - `error`: ドメインエラー型

// unsafe コードを禁止する（src/CLAUDE.md および src/backend/CLAUDE.md の必須要件）
#![forbid(unsafe_code)]
// 例外: doc コメントのリンク省略は許容（テスト補助関数等）
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
// 例外: モジュール名重複は許容（例: service::WorkExecutionService）
#![allow(clippy::module_name_repetitions)]
// 例外: must_use 警告は許容（ドメインモデルの多くは結果を使用しない場合がある）
#![allow(clippy::must_use_candidate)]

pub mod error;
pub mod events;
pub mod model;
pub mod repository;
pub mod rules;
pub mod service;

// 主要な型を再エクスポートして使いやすくする
pub use error::DomainError;
pub use events::{
    DomainEvent, DomainEventReceiver, DomainEventSender, create_domain_event_channel,
};
