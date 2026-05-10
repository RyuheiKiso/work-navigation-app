//! RFC 7807 Problem Details for HTTP APIs に基づく統一エラー応答。
//!
//! 対応 §: ロードマップ §10.5 §10.6 §11.4 §20.1
//!
//! 生 `StatusCode` を返していた従来実装では、UI が `HTTP 403` のような
//! 技術文字列をそのままユーザーへ晒していた。本モジュールはサーバ側で
//! `error_code` / `request_id` を含む統一スキーマを返し、フロント側で
//! ローカライズ可能にする。

use axum::http::header;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;

/// リクエスト ID をレスポンス／ログで識別するためのヘッダ名。
pub const REQUEST_ID_HEADER: &str = "x-request-id";

/// API エラーの種別。フロント側の i18n キー (`error.api.<kind>`) と一対一対応する。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorKind {
    /// 入力検証失敗 (400)
    BadRequest,
    /// 未認証 (401)
    Auth,
    /// 認可拒否 (403)
    Forbidden,
    /// 対象が存在しない (404)
    NotFound,
    /// 他者が先に更新した／状態遷移不正 (409)
    Conflict,
    /// レート制限 (429)
    RateLimited,
    /// サーバ内部エラー (500)
    Server,
}

impl ErrorKind {
    pub fn status(self) -> StatusCode {
        match self {
            Self::BadRequest => StatusCode::BAD_REQUEST,
            Self::Auth => StatusCode::UNAUTHORIZED,
            Self::Forbidden => StatusCode::FORBIDDEN,
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::Conflict => StatusCode::CONFLICT,
            Self::RateLimited => StatusCode::TOO_MANY_REQUESTS,
            Self::Server => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    /// `type` URI（RFC 7807）。MVP ではアンカー的な相対 URI を使う。
    pub fn type_uri(self) -> &'static str {
        match self {
            Self::BadRequest => "/errors/bad-request",
            Self::Auth => "/errors/auth",
            Self::Forbidden => "/errors/forbidden",
            Self::NotFound => "/errors/not-found",
            Self::Conflict => "/errors/conflict",
            Self::RateLimited => "/errors/rate-limited",
            Self::Server => "/errors/server",
        }
    }

    /// 既定の `title`（言語非依存）。詳細はフロントが i18n で出す。
    pub fn title(self) -> &'static str {
        match self {
            Self::BadRequest => "Bad Request",
            Self::Auth => "Authentication Required",
            Self::Forbidden => "Forbidden",
            Self::NotFound => "Not Found",
            Self::Conflict => "Conflict",
            Self::RateLimited => "Too Many Requests",
            Self::Server => "Internal Server Error",
        }
    }
}

/// 統一エラー型。ハンドラから `Result<T, ApiError>` で返す。
#[derive(Debug, Clone)]
pub struct ApiError {
    pub kind: ErrorKind,
    /// マシン可読の細分コード（例: "task_not_found", "invalid_state_transition"）
    pub code: &'static str,
    /// 開発者向けの英語デバッグ詳細。エンドユーザーには見せない前提
    pub detail: Option<String>,
    /// 同じ操作を再実行できるか（429 や 5xx で true）
    pub retriable: bool,
    /// `Retry-After` ヘッダに乗せる秒数（任意）
    pub retry_after_seconds: Option<u32>,
    /// リクエスト ID（middleware で付与され、IntoResponse で書き戻す）
    pub request_id: Option<String>,
}

impl ApiError {
    pub fn new(kind: ErrorKind, code: &'static str) -> Self {
        let retriable = matches!(kind, ErrorKind::Server | ErrorKind::RateLimited);
        Self {
            kind,
            code,
            detail: None,
            retriable,
            retry_after_seconds: None,
            request_id: None,
        }
    }

    pub fn bad_request(code: &'static str) -> Self { Self::new(ErrorKind::BadRequest, code) }
    pub fn auth(code: &'static str) -> Self { Self::new(ErrorKind::Auth, code) }
    pub fn forbidden(code: &'static str) -> Self { Self::new(ErrorKind::Forbidden, code) }
    pub fn not_found(code: &'static str) -> Self { Self::new(ErrorKind::NotFound, code) }
    pub fn conflict(code: &'static str) -> Self { Self::new(ErrorKind::Conflict, code) }
    pub fn server(code: &'static str) -> Self { Self::new(ErrorKind::Server, code) }
    pub fn rate_limited(code: &'static str, retry_after: u32) -> Self {
        let mut e = Self::new(ErrorKind::RateLimited, code);
        e.retry_after_seconds = Some(retry_after);
        e
    }

    pub fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    pub fn with_request_id(mut self, request_id: impl Into<String>) -> Self {
        self.request_id = Some(request_id.into());
        self
    }
}

#[derive(Serialize)]
struct ProblemDetails<'a> {
    /// RFC 7807 の type URI
    #[serde(rename = "type")]
    type_uri: &'a str,
    title: &'a str,
    status: u16,
    /// マシン可読コード（フロントの i18n キーへ写像する）
    code: &'a str,
    /// 詳細（任意・英語デバッグ）
    #[serde(skip_serializing_if = "Option::is_none")]
    detail: Option<&'a str>,
    /// リトライ可能性
    retriable: bool,
    /// リクエスト ID（サポート対応で参照する）
    #[serde(skip_serializing_if = "Option::is_none")]
    request_id: Option<&'a str>,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let body = ProblemDetails {
            type_uri: self.kind.type_uri(),
            title: self.kind.title(),
            status: self.kind.status().as_u16(),
            code: self.code,
            detail: self.detail.as_deref(),
            retriable: self.retriable,
            request_id: self.request_id.as_deref(),
        };
        let mut headers = HeaderMap::new();
        // RFC 7807 で規定された MIME
        headers.insert(
            header::CONTENT_TYPE,
            "application/problem+json".parse().expect("static content-type"),
        );
        if let Some(secs) = self.retry_after_seconds {
            if let Ok(v) = secs.to_string().parse() {
                headers.insert(header::RETRY_AFTER, v);
            }
        }
        if let Some(ref rid) = self.request_id {
            if let Ok(v) = rid.parse() {
                headers.insert(REQUEST_ID_HEADER, v);
            }
        }
        (self.kind.status(), headers, Json(body)).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::to_bytes;
    use axum::http::StatusCode;

    #[test]
    fn kind_status_mapping_is_canonical() {
        assert_eq!(ErrorKind::BadRequest.status(), StatusCode::BAD_REQUEST);
        assert_eq!(ErrorKind::Auth.status(), StatusCode::UNAUTHORIZED);
        assert_eq!(ErrorKind::Forbidden.status(), StatusCode::FORBIDDEN);
        assert_eq!(ErrorKind::NotFound.status(), StatusCode::NOT_FOUND);
        assert_eq!(ErrorKind::Conflict.status(), StatusCode::CONFLICT);
        assert_eq!(ErrorKind::RateLimited.status(), StatusCode::TOO_MANY_REQUESTS);
        assert_eq!(ErrorKind::Server.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn server_and_rate_limited_are_retriable_by_default() {
        assert!(ApiError::server("x").retriable);
        assert!(ApiError::rate_limited("x", 1).retriable);
        assert!(!ApiError::not_found("x").retriable);
        assert!(!ApiError::conflict("x").retriable);
    }

    #[tokio::test]
    async fn into_response_emits_problem_json_with_request_id() {
        let err = ApiError::not_found("task_not_found")
            .with_request_id("rid-123");
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
        assert_eq!(
            resp.headers().get(axum::http::header::CONTENT_TYPE).unwrap(),
            "application/problem+json"
        );
        assert_eq!(resp.headers().get(REQUEST_ID_HEADER).unwrap(), "rid-123");
        let body = to_bytes(resp.into_body(), 4096).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["status"], 404);
        assert_eq!(json["code"], "task_not_found");
        assert_eq!(json["type"], "/errors/not-found");
        assert_eq!(json["request_id"], "rid-123");
        assert_eq!(json["retriable"], false);
    }

    #[tokio::test]
    async fn rate_limited_response_carries_retry_after() {
        let err = ApiError::rate_limited("throttled", 5);
        let resp = err.into_response();
        assert_eq!(
            resp.headers().get(axum::http::header::RETRY_AFTER).unwrap(),
            "5"
        );
    }
}
