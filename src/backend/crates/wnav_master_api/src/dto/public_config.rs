// 公開設定 DTO（ADR-IMPL-002）
//
// GET /api/v1/public/config のレスポンス型。
// 認証不要・非機密情報のみ含む。

use serde::Serialize;
use utoipa::ToSchema;

/// マスタ SPA 起動時に取得する公開設定（ADR-IMPL-002）
///
/// 認証なしで取得可能。機密情報（DB URL・秘密鍵等）は絶対に含めない。
#[derive(Debug, Serialize, ToSchema)]
pub struct PublicConfigResponse {
    /// マスタ SPA が接続する API のベース URL
    pub api_base_url: String,
    /// OpenAPI スキーマ配信 URL
    pub openapi_url: String,
    /// セッションタイムアウト（分）
    pub session_timeout_min: u64,
    /// マスタ SPA のポーリング間隔（ミリ秒）
    pub polling_interval_ms: u64,
}
