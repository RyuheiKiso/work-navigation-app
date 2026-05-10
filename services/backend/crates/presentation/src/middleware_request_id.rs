//! リクエスト ID ミドルウェア。
//!
//! 対応 §: ロードマップ §11.4 §31.2 観測性
//!
//! クライアントが `X-Request-Id` を送ってきた場合はそれを使い、
//! 無ければ UUIDv7 を発行して両方向に伝搬させる。
//! ハンドラ側は `Extension<RequestId>` で取得でき、エラー応答にも乗る。

use axum::extract::Request;
use axum::http::{HeaderName, HeaderValue};
use axum::middleware::Next;
use axum::response::Response;
use uuid::Uuid;

use crate::api_error::REQUEST_ID_HEADER;

/// ハンドラから参照するためのリクエスト ID 型。
#[derive(Clone, Debug)]
pub struct RequestId(pub String);

pub async fn request_id(mut req: Request, next: Next) -> Response {
    let header_name: HeaderName = REQUEST_ID_HEADER.parse().expect("static header name");
    let provided = req
        .headers()
        .get(&header_name)
        .and_then(|v| v.to_str().ok())
        .filter(|s| is_valid(s))
        .map(|s| s.to_owned());

    let id = provided.unwrap_or_else(|| Uuid::now_v7().to_string());
    let request_id = RequestId(id.clone());

    // ハンドラから extract できるように extension に積む
    req.extensions_mut().insert(request_id);

    let mut res = next.run(req).await;
    if let Ok(v) = HeaderValue::from_str(&id) {
        res.headers_mut().insert(header_name, v);
    }
    res
}

/// X-Request-Id 偽装入力で攻撃面を増やさないため、可視 ASCII の最大 128 字に絞る。
fn is_valid(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 128
        && s.bytes().all(|b| b.is_ascii_graphic() || b == b'-' || b == b'_')
}
