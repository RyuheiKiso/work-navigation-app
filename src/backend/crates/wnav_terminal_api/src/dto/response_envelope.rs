// 全エンドポイント共通のレスポンスエンベロープ型（OpenAPI 共通仕様 §3）
//
// 成功レスポンスは `ApiResponse<T>` でラップする。
// 一覧取得は `PaginatedResponse<T>` でラップする。

use chrono::{DateTime, Utc};
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

/// 全エンドポイント共通の meta フィールド
#[derive(Debug, Serialize, ToSchema)]
pub struct ResponseMeta {
    /// サーバーが採番したリクエスト追跡 ID（UUID v7）
    pub request_id: Uuid,
    /// サーバー処理完了時刻（ISO 8601 UTC）
    pub server_time: DateTime<Utc>,
    /// API バージョン（常に "v1"）
    pub api_version: &'static str,
}

impl ResponseMeta {
    /// 現在時刻で ResponseMeta を生成する
    pub fn now() -> Self {
        Self {
            request_id: Uuid::now_v7(),
            server_time: Utc::now(),
            api_version: "v1",
        }
    }
}

/// 単件レスポンスエンベロープ
///
/// 成功レスポンスの標準形式: `{ "data": T, "meta": {...} }`
#[derive(Debug, Serialize, ToSchema)]
pub struct ApiResponse<T: Serialize> {
    pub data: T,
    pub meta: ResponseMeta,
}

impl<T: Serialize> ApiResponse<T> {
    /// データを渡して ApiResponse を構築する
    pub fn new(data: T) -> Self {
        Self {
            data,
            meta: ResponseMeta::now(),
        }
    }
}

/// ページングメタ情報（一覧取得エンドポイント用）
#[derive(Debug, Serialize, ToSchema)]
pub struct PaginatedMeta {
    /// サーバーが採番したリクエスト追跡 ID
    pub request_id: Uuid,
    /// サーバー処理完了時刻
    pub server_time: DateTime<Utc>,
    /// API バージョン
    pub api_version: &'static str,
    /// フィルタ後の全件数
    pub total: i64,
    /// 現在のページ番号（1 始まり）
    pub page: i64,
    /// 1 ページあたりの件数
    pub per_page: i64,
    /// 総ページ数
    pub total_pages: i64,
}

impl PaginatedMeta {
    /// ページング情報を渡して PaginatedMeta を構築する
    pub fn new(total: i64, page: i64, per_page: i64) -> Self {
        let total_pages = if per_page > 0 {
            (total + per_page - 1) / per_page
        } else {
            0
        };
        Self {
            request_id: Uuid::now_v7(),
            server_time: Utc::now(),
            api_version: "v1",
            total,
            page,
            per_page,
            total_pages,
        }
    }
}

/// 一覧取得レスポンスエンベロープ
///
/// カーソルページング用のメタフィールドを持つ
#[derive(Debug, Serialize, ToSchema)]
pub struct PaginatedResponse<T: Serialize> {
    pub data: Vec<T>,
    pub meta: PaginatedMeta,
}

impl<T: Serialize> PaginatedResponse<T> {
    /// データと全件数を渡して PaginatedResponse を構築する
    pub fn new(data: Vec<T>, total: i64, page: i64, per_page: i64) -> Self {
        Self {
            meta: PaginatedMeta::new(total, page, per_page),
            data,
        }
    }
}

/// カーソルページングレスポンスのメタ（work_assignments 等で使用）
#[derive(Debug, Serialize, ToSchema)]
pub struct CursorMeta {
    /// サーバーが採番したリクエスト追跡 ID
    pub request_id: Uuid,
    /// サーバー処理完了時刻
    pub server_time: DateTime<Utc>,
    /// API バージョン
    pub api_version: &'static str,
    /// リクエストで指定した limit 値
    pub limit: i64,
    /// まだ取得できるレコードが存在する場合 true
    pub has_more: bool,
    /// 次ページ取得時に after パラメータに指定する値（UUID v7 文字列）
    pub next_cursor: Option<Uuid>,
}

/// カーソルページングレスポンスエンベロープ
#[derive(Debug, Serialize, ToSchema)]
pub struct CursorResponse<T: Serialize> {
    pub data: Vec<T>,
    pub meta: CursorMeta,
}
