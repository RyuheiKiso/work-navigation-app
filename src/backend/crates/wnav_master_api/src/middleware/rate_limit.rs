// レート制限ミドルウェア（wnav_master_api）
//
// 管理系 API への過剰リクエストを防ぐトークンバケット方式のレート制限。
// terminal-api より緩いデフォルト値（管理者・編集者向け）。

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::Instant,
};

use axum::{
    Extension,
    extract::Request,
    middleware::Next,
    response::Response,
};

use crate::error::AppError;

/// トークンバケット（1 バケット = 1 クライアントキー）
struct TokenBucket {
    tokens: f64,
    last_refill: Instant,
}

/// レートリミッター（管理系 API 用）
pub struct RateLimiter {
    buckets: Mutex<HashMap<String, TokenBucket>>,
    rpm: u32,
}

impl RateLimiter {
    pub fn new(rpm: u32) -> Arc<Self> {
        Arc::new(Self {
            buckets: Mutex::new(HashMap::new()),
            rpm,
        })
    }

    /// トークンを消費する。超過時は false を返す。
    fn consume(&self, key: &str) -> bool {
        let mut buckets = self.buckets.lock().unwrap_or_else(|e| e.into_inner());
        let bucket = buckets.entry(key.to_string()).or_insert_with(|| TokenBucket {
            tokens: self.rpm as f64,
            last_refill: Instant::now(),
        });
        let elapsed = bucket.last_refill.elapsed();
        let refill = elapsed.as_secs_f64() * (self.rpm as f64 / 60.0);
        bucket.tokens = (bucket.tokens + refill).min(self.rpm as f64);
        bucket.last_refill = Instant::now();
        if bucket.tokens >= 1.0 {
            bucket.tokens -= 1.0;
            true
        } else {
            false
        }
    }
}

/// master-api のレート制限ミドルウェア。
///
/// `Extension<Arc<RateLimiter>>` からリミッターを取得する。
/// Authorization ヘッダの Bearer トークン先頭 16 文字または IP アドレスをキーとする。
pub async fn rate_limit_middleware(
    Extension(limiter): Extension<std::sync::Arc<RateLimiter>>,
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    // Authorization ヘッダから Bearer トークンを抽出してキーに使う（IP より精密）
    let key = request
        .headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .map(|token| token.get(..16).unwrap_or(token).to_string())
        .or_else(|| {
            // フォールバック: X-Forwarded-For ヘッダ
            request
                .headers()
                .get("X-Forwarded-For")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.split(',').next().unwrap_or("unknown").trim().to_string())
        })
        .unwrap_or_else(|| "anonymous".to_string());

    if !limiter.consume(&key) {
        // AppError::RateLimited を返す（管理 API への過剰アクセス防止）
        return Err(AppError::RateLimited);
    }
    Ok(next.run(request).await)
}
