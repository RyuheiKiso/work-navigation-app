//! リクエスト ID + tracing span ミドルウェア。
//!
//! 対応 §: ロードマップ §11.4 §31.2 §16
//!
//! クライアントが `X-Request-Id` を送ってきた場合はそれを使い、
//! 無ければ UUIDv7 を発行して両方向に伝搬させる。
//! 構造化ログ (tracing) に request_id / method / path を載せ、
//! 後段の障害追跡で 1 リクエストに紐づく全イベントを横断可能にする。

use axum::extract::Request;
use axum::http::{HeaderName, HeaderValue};
use axum::middleware::Next;
use axum::response::Response;
use tracing::Instrument;
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

    let method = req.method().clone();
    let path = req.uri().path().to_owned();

    // request_id を付与した span 配下で next.run を走らせる。
    // ハンドラ内 tracing::info! などはこの span を継承する。
    let span = tracing::info_span!(
        "http.request",
        request_id = %id,
        http.method = %method,
        http.path = %path,
    );

    let mut res = next.run(req).instrument(span).await;
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
