// ヘルスチェック DTO
//
// GET /healthz および GET /api/v1/ops/health の Response 型。

use serde::Serialize;
use utoipa::ToSchema;

/// ヘルスチェックレスポンス
///
/// DB 疎通確認と JWT キーストアの存在確認を含む。
#[derive(Debug, Serialize, ToSchema)]
pub struct HealthResponse {
    /// "ok" または "degraded" または "unhealthy"
    pub status: String,
    /// サービス名
    pub service: String,
    /// バージョン文字列
    pub version: String,
    /// 各コンポーネントの詳細ステータス
    pub components: HealthComponents,
    /// タイムスタンプ（UTC ISO 8601）
    pub timestamp: String,
}

/// ヘルスチェック詳細コンポーネント
#[derive(Debug, Serialize, ToSchema)]
pub struct HealthComponents {
    /// 書き込みプール（app_write）の疎通確認結果
    pub write_db: ComponentStatus,
    /// 読み取りプール（app_read）の疎通確認結果
    pub read_db: ComponentStatus,
    /// JWT キーストアの存在確認結果
    pub jwt_keys: ComponentStatus,
}

/// 個別コンポーネントのステータス
#[derive(Debug, Serialize, ToSchema)]
pub struct ComponentStatus {
    /// "ok" または "error"
    pub status: String,
    /// エラーメッセージ（status が "error" の場合のみ）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// レイテンシ（ミリ秒）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latency_ms: Option<u128>,
}
