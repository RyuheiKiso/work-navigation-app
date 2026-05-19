// システム API（API-system-001〜003）の DTO 定義（07_運用・監視API仕様.md §9〜11）

use chrono::{DateTime, Utc};
use serde::Serialize;
use utoipa::ToSchema;

/// ヘルスチェックレスポンス（API-system-001: GET /api/v1/healthz）
///
/// バックエンドプロセスが起動中であれば常に HTTP 200 / status: "ok" を返す
#[derive(Debug, Serialize, ToSchema)]
pub struct HealthResponse {
    /// ステータス（常に "ok"）
    pub status: &'static str,
    /// チェック実行時刻
    pub timestamp: DateTime<Utc>,
}

/// Readiness チェック詳細（API-system-002: GET /api/v1/readyz）
#[derive(Debug, Serialize, ToSchema)]
pub struct ReadyzChecks {
    /// PostgreSQL 接続確認（ok / error）
    pub database: String,
    /// Outbox Consumer 稼働確認（ok / error）
    pub outbox_consumer: String,
    /// LDAP 接続確認（ok / degraded / error）
    pub ldap: String,
}

/// Readiness チェックレスポンス（API-system-002）
#[derive(Debug, Serialize, ToSchema)]
pub struct ReadyzResponse {
    /// ステータス（ready / not_ready）
    pub status: String,
    /// 各コンポーネントのチェック結果
    pub checks: ReadyzChecks,
    /// チェック実行時刻
    pub timestamp: DateTime<Utc>,
}
