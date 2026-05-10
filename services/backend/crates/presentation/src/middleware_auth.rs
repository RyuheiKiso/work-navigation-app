//! セッション検証ミドルウェア（HMAC-SHA256）
//!
//! 対応 §: ロードマップ §10.5 §11.4.1 ADR-0010

use axum::{
    body::Body,
    extract::State,
    http::{header, Request, StatusCode},
    middleware::Next,
    response::Response,
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use std::sync::Arc;
use wna_adapter::Hs256SessionFactory;

#[derive(Debug, Clone)]
pub struct AuthContext {
    pub user_id: String,
    pub issued_at: i64,
}

pub const SESSION_MAX_AGE_SECONDS: i64 = 8 * 60 * 60;

pub async fn require_session(
    State(factory): State<Arc<Hs256SessionFactory>>,
    mut req: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;
    let token = auth.strip_prefix("Bearer ").ok_or(StatusCode::UNAUTHORIZED)?;
    let mut parts = token.splitn(2, '.');
    let payload_b64 = parts.next().ok_or(StatusCode::UNAUTHORIZED)?;
    let sig_b64 = parts.next().ok_or(StatusCode::UNAUTHORIZED)?;
    let payload = URL_SAFE_NO_PAD
        .decode(payload_b64)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    let sig = URL_SAFE_NO_PAD
        .decode(sig_b64)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    if !factory.verify_signature(&payload, &sig) {
        return Err(StatusCode::UNAUTHORIZED);
    }
    let payload_str = std::str::from_utf8(&payload).map_err(|_| StatusCode::UNAUTHORIZED)?;
    let mut p = payload_str.splitn(2, '.');
    let user_id = p.next().ok_or(StatusCode::UNAUTHORIZED)?.to_string();
    let ts_str = p.next().ok_or(StatusCode::UNAUTHORIZED)?;
    let issued_at: i64 = ts_str.parse().map_err(|_| StatusCode::UNAUTHORIZED)?;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|_| StatusCode::UNAUTHORIZED)?
        .as_secs() as i64;
    if now - issued_at > SESSION_MAX_AGE_SECONDS {
        return Err(StatusCode::UNAUTHORIZED);
    }
    if user_id.is_empty() {
        return Err(StatusCode::UNAUTHORIZED);
    }
    req.extensions_mut().insert(AuthContext { user_id, issued_at });
    Ok(next.run(req).await)
}
