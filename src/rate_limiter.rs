use super::*;

// ── 速率限制中间件 ──
// Block 5 P2-2: 基于 IP 的令牌桶限流

#[derive(Clone)]
pub(super) struct RateLimiter {
    inner: Arc<std::sync::Mutex<RateLimiterInner>>,
    max_rps: u32,
}

struct RateLimiterInner {
    buckets: BTreeMap<String, TokenBucket>,
}

#[derive(Clone)]
struct TokenBucket {
    tokens: f64,
    last_refill_ms: u64,
}

impl RateLimiter {
    pub(super) fn new(max_rps: u32) -> Self {
        Self {
            inner: Arc::new(std::sync::Mutex::new(RateLimiterInner {
                buckets: BTreeMap::new(),
            })),
            max_rps,
        }
    }

    fn check(&self, client_ip: &str, now_ms: u64) -> Result<(), u64> {
        let mut inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        // v2.1.x: 每 500 次请求清理一次，降低内存峰值
        if inner.buckets.len() > 500 {
            inner.buckets.retain(|_, bucket| {
                now_ms.saturating_sub(bucket.last_refill_ms) < 600_000
            });
        }
        let bucket = inner.buckets.entry(client_ip.to_string()).or_insert_with(|| {
            TokenBucket {
                tokens: self.max_rps as f64,
                last_refill_ms: now_ms,
            }
        });

        let elapsed_ms = now_ms.saturating_sub(bucket.last_refill_ms);
        let refill = (elapsed_ms as f64 / 1000.0) * self.max_rps as f64;
        bucket.tokens = (bucket.tokens + refill).min(self.max_rps as f64);
        bucket.last_refill_ms = now_ms;

        if bucket.tokens >= 1.0 {
            bucket.tokens -= 1.0;
            Ok(())
        } else {
            let wait_ms = ((1.0 - bucket.tokens) / self.max_rps as f64 * 1000.0).ceil() as u64;
            Err(wait_ms.max(1000))
        }
    }
}

pub(super) async fn rate_limit_middleware(
    request: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    // 从扩展中获取 RateLimiter（由 middleware layer 注入）
    // 简化实现: 直接读取环境变量配置，全局共享
    static LIMITER: std::sync::OnceLock<RateLimiter> = std::sync::OnceLock::new();
    let limiter = LIMITER.get_or_init(|| {
        let max_rps = std::env::var("QUANTPILOT_RATE_LIMIT_RPS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(100);
        RateLimiter::new(max_rps)
    });

    // v2.0.1: 优先使用真实 TCP 地址，x-forwarded-for 仅作反向代理时的补充
    let client_ip = request
        .extensions()
        .get::<axum::extract::ConnectInfo<std::net::SocketAddr>>()
        .map(|ci| ci.0.ip().to_string())
        .unwrap_or_else(|| {
            request
                .headers()
                .get("x-forwarded-for")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.split(',').next().unwrap_or("unknown").trim())
                .unwrap_or("unknown")
                .to_string()
        });

    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;

    match limiter.check(&client_ip, now_ms) {
        Ok(()) => next.run(request).await,
        Err(retry_after_ms) => {
            let problem = serde_json::json!({
                "type": "rate-limited",
                "title": "请求过于频繁",
                "status": 429,
                "detail": format!("请求过于频繁（当前限制: 每秒 {} 次）。请在 {} 秒后重试", limiter.max_rps, retry_after_ms / 1000),
                "error_code": "RATE_LIMITED",
                "retryable": true,
            });

            axum::response::Response::builder()
                .status(StatusCode::TOO_MANY_REQUESTS)
                .header(axum::http::header::CONTENT_TYPE, "application/problem+json")
                .header(
                    axum::http::header::RETRY_AFTER,
                    (retry_after_ms / 1000).to_string(),
                )
                .body(axum::body::Body::from(
                    serde_json::to_string(&problem).unwrap_or_default(),
                ))
                .unwrap_or_else(|_| {
                    axum::response::Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .body(axum::body::Body::empty())
                        .unwrap_or_else(|_| axum::response::Response::new(axum::body::Body::empty()))
                })
        }
    }
}

// v2.2.1: 认证端点独立速率限制 (防暴力破解, ~6次/分钟/IP)
pub(super) async fn auth_rate_limit_middleware(
    request: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    static AUTH_LIMITER: std::sync::OnceLock<RateLimiter> = std::sync::OnceLock::new();
    // RateLimiter API 以 RPS 为单位: 1 token/s = ~6次/分钟 (u32 最小值为 1)
    let limiter = AUTH_LIMITER.get_or_init(|| RateLimiter::new(1));

    let client_ip = request
        .extensions()
        .get::<axum::extract::ConnectInfo<std::net::SocketAddr>>()
        .map(|ci| ci.0.ip().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;

    match limiter.check(&client_ip, now_ms) {
        Ok(()) => next.run(request).await,
        Err(retry_after_ms) => {
            let problem = serde_json::json!({
                "type": "rate-limited",
                "title": "登录尝试过于频繁",
                "status": 429,
                "detail": format!("登录尝试过于频繁（限制: 约每分钟 {} 次）。请在 {} 秒后重试", limiter.max_rps * 6, retry_after_ms / 1000),
                "error_code": "AUTH_RATE_LIMITED",
                "retryable": true,
            });
            axum::response::Response::builder()
                .status(StatusCode::TOO_MANY_REQUESTS)
                .header(axum::http::header::CONTENT_TYPE, "application/problem+json")
                .header(axum::http::header::RETRY_AFTER, (retry_after_ms / 1000).to_string())
                .body(axum::body::Body::from(serde_json::to_string(&problem).unwrap_or_default()))
                .unwrap_or_else(|_| {
                    axum::response::Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .body(axum::body::Body::empty())
                        .unwrap_or_else(|_| axum::response::Response::new(axum::body::Body::empty()))
                })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rate_limiter_allows_requests_within_limit() {
        let limiter = RateLimiter::new(10);
        let now = 1000000u64;
        for _ in 0..10 {
            assert!(limiter.check("127.0.0.1", now).is_ok());
        }
        assert!(limiter.check("127.0.0.1", now).is_err());
    }

    #[test]
    fn rate_limiter_refills_over_time() {
        let limiter = RateLimiter::new(10);
        let now = 1000000u64;
        for _ in 0..10 {
            limiter.check("10.0.0.1", now).unwrap();
        }
        assert!(limiter.check("10.0.0.1", now).is_err());
        assert!(limiter.check("10.0.0.1", now + 1000).is_ok());
    }

    #[test]
    fn different_ips_have_separate_buckets() {
        let limiter = RateLimiter::new(5);
        let now = 1000000u64;
        for _ in 0..5 {
            limiter.check("192.168.1.1", now).unwrap();
        }
        assert!(limiter.check("192.168.1.1", now).is_err());
        assert!(limiter.check("192.168.1.2", now).is_ok());
    }
}
