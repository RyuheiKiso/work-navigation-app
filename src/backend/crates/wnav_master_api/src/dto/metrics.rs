// メトリクス DTO（API-ops-002）
//
// Prometheus 互換のメトリクス出力型。

use serde::Serialize;
use utoipa::ToSchema;

/// Prometheus 互換メトリクスレスポンス
///
/// Content-Type: text/plain; version=0.0.4 で返す。
/// ただし OpenAPI ドキュメント用に Rust 型も定義する。
#[derive(Debug, Serialize, ToSchema)]
pub struct MetricsResponse {
    /// Prometheus テキスト形式のメトリクス本文
    pub metrics_text: String,
}
