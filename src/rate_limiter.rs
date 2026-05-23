use super::*;

// ── 速率限制中间件 ──
// Block 5 P2-2: 基于 IP 的令牌桶限流

#[derive(Clone)]
pub(super) struct RateLimiter {
    inner: Arc<std::sync::Mutex<RateLimiterInner>>,
    capacity: f64,
    refill_per_second: f64,
    limit_per_minute: u32,
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
        let max_rps = max_rps.max(1);
        Self {
            inner: Arc::new(std::sync::Mutex::new(RateLimiterInner {
                buckets: BTreeMap::new(),
            })),
            capacity: max_rps as f64,
            refill_per_second: max_rps as f64,
            limit_per_minute: max_rps.saturating_mul(60),
        }
    }

    pub(super) fn new_per_minute(max_per_minute: u32) -> Self {
        let max_per_minute = max_per_minute.max(1);
        Self {
            inner: Arc::new(std::sync::Mutex::new(RateLimiterInner {
                buckets: BTreeMap::new(),
            })),
            capacity: max_per_minute as f64,
            refill_per_second: max_per_minute as f64 / 60.0,
            limit_per_minute: max_per_minute,
        }
    }

    fn check(&self, client_ip: &str, now_ms: u64) -> Result<(), u64> {
        let mut inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        // v2.1.x: 每 500 次请求清理一次，降低内存峰值
        // v2.5.0 NOTE: retain O(n) 在持锁时执行, 高频场景可改为后台定期清理任务
        if inner.buckets.len() > 500 {
            inner
                .buckets
                .retain(|_, bucket| now_ms.saturating_sub(bucket.last_refill_ms) < 600_000);
        }
        let bucket = inner
            .buckets
            .entry(client_ip.to_string())
            .or_insert_with(|| TokenBucket {
                tokens: self.capacity,
                last_refill_ms: now_ms,
            });

        let elapsed_ms = now_ms.saturating_sub(bucket.last_refill_ms);
        let refill = (elapsed_ms as f64 / 1000.0) * self.refill_per_second;
        bucket.tokens = (bucket.tokens + refill).min(self.capacity);
        bucket.last_refill_ms = now_ms;

        if bucket.tokens >= 1.0 {
            bucket.tokens -= 1.0;
            Ok(())
        } else {
            let wait_ms = ((1.0 - bucket.tokens) / self.refill_per_second * 1000.0).ceil() as u64;
            Err(wait_ms.max(1000))
        }
    }
}

pub(super) async fn rate_limit_middleware(
    request: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    // v3.5.0: DEV 模式下跳过速率限制, 方便本地调试
    if std::env::var("QUANTPILOT_DEV").map_or(false, |v| v == "true" || v == "1") {
        return next.run(request).await;
    }

    let max_rps = std::env::var("QUANTPILOT_RATE_LIMIT_RPS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(100)
        .max(1); // v2.4.0 P1-E2: 拒绝 max_rps=0 导致全部请求 429
    let limiter = global_rate_limiter_for_rps(max_rps);

    // v2.3.4: 仅使用真实 TCP 地址。X-Forwarded-For 可被客户端伪造，
    // 仅在明确配置反向代理模式且设置了 QUANTPILOT_TRUSTED_PROXY 时使用
    let client_ip = request
        .extensions()
        .get::<axum::extract::ConnectInfo<std::net::SocketAddr>>()
        .map(|ci| ci.0.ip().to_string())
        .unwrap_or_else(|| {
            // 仅在 DEV 模式或可信代理模式下回退到 X-Forwarded-For
            if std::env::var("QUANTPILOT_DEV").map_or(false, |v| v == "true" || v == "1")
                || std::env::var("QUANTPILOT_TRUSTED_PROXY")
                    .map_or(false, |v| v == "true" || v == "1")
            {
                request
                    .headers()
                    .get("x-forwarded-for")
                    .and_then(|v| v.to_str().ok())
                    .map(|s| s.split(',').next().unwrap_or("unknown").trim())
                    .unwrap_or("unknown")
                    .to_string()
            } else {
                "unknown".to_string()
            }
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
                "detail": format!("请求过于频繁，请在 {} 秒后重试", retry_after_ms / 1000),
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
                        .unwrap_or_else(
                            |_| axum::response::Response::new(axum::body::Body::empty()),
                        )
                })
        }
    }
}

fn global_rate_limiter_for_rps(max_rps: u32) -> RateLimiter {
    static LIMITERS: std::sync::OnceLock<std::sync::Mutex<BTreeMap<u32, RateLimiter>>> =
        std::sync::OnceLock::new();
    let limiters = LIMITERS.get_or_init(|| std::sync::Mutex::new(BTreeMap::new()));
    let mut limiters = limiters.lock().unwrap_or_else(|e| e.into_inner());
    limiters
        .entry(max_rps.max(1))
        .or_insert_with(|| RateLimiter::new(max_rps))
        .clone()
}

// v2.2.1: 认证端点独立速率限制 (防暴力破解, ~6次/分钟/IP)
pub(super) async fn auth_rate_limit_middleware(
    request: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    // v3.5.0: DEV 模式下跳过速率限制, 方便本地调试
    if std::env::var("QUANTPILOT_DEV").map_or(false, |v| v == "true" || v == "1") {
        return next.run(request).await;
    }

    static AUTH_LIMITER: std::sync::OnceLock<RateLimiter> = std::sync::OnceLock::new();
    let limiter = AUTH_LIMITER.get_or_init(|| RateLimiter::new_per_minute(6));

    let client_ip = request
        .extensions()
        .get::<axum::extract::ConnectInfo<std::net::SocketAddr>>()
        .map(|ci| ci.0.ip().to_string())
        .unwrap_or_else(|| {
            // v2.4.0 P1-E4: auth 限流器与全局限流器一致的代理检测
            if std::env::var("QUANTPILOT_DEV").map_or(false, |v| v == "true" || v == "1")
                || std::env::var("QUANTPILOT_TRUSTED_PROXY")
                    .map_or(false, |v| v == "true" || v == "1")
            {
                request
                    .headers()
                    .get("x-forwarded-for")
                    .and_then(|v| v.to_str().ok())
                    .and_then(|v| v.split(',').next())
                    .map(|s| s.trim().to_string())
                    .unwrap_or_else(|| "unknown".to_string())
            } else {
                "unknown".to_string()
            }
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
                "title": "登录尝试过于频繁",
                "status": 429,
                "detail": format!("登录尝试过于频繁（限制: 约每分钟 {} 次）。请在 {} 秒后重试", limiter.limit_per_minute, retry_after_ms / 1000),
                "error_code": "AUTH_RATE_LIMITED",
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
                        .unwrap_or_else(
                            |_| axum::response::Response::new(axum::body::Body::empty()),
                        )
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

    #[test]
    fn global_rate_limiter_is_keyed_by_runtime_config() {
        let low = global_rate_limiter_for_rps(1);
        let high = global_rate_limiter_for_rps(2);
        let now = 1000000u64;
        assert!(low.check("config-keyed", now).is_ok());
        assert!(low.check("config-keyed", now).is_err());
        assert!(high.check("config-keyed", now).is_ok());
        assert!(high.check("config-keyed", now).is_ok());
    }
}
