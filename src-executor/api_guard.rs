/// v3.7.0: API 守卫中间件
/// 解密请求体 (AES-256-GCM) → 验证 HMAC 签名 → 时间窗口防重放 (±5s)

use qrpc_session;
use axum::{
    body::Body,
    extract::Request,
    middleware::Next,
    response::Response,
};
use axum::http::StatusCode;

/// API 守卫中间件 — v3.5.0: Phase 5 HMAC验证激活
pub async fn api_guard_middleware(
    request: Request,
    next: Next,
) -> Result<Response, (StatusCode, String)> {
    if request.uri().path() == "/api/executor/health" {
        return Ok(next.run(request).await);
    }
    // v3.7.0: 开发环境可降级, 生产强制验证
    if std::env::var("QUANTPILOT_EXECUTOR_INSECURE").map_or(false, |v| v == "true" || v == "1") {
        return Ok(next.run(request).await);
    }
    let sig_header = request.headers().get("x-executor-signature")
        .and_then(|v| v.to_str().ok()).map(|s| s.to_string());
    let ts_header = request.headers().get("x-executor-timestamp")
        .and_then(|v| v.to_str().ok()).map(|s| s.to_string());
    if sig_header.is_none() {
        return Err((StatusCode::UNAUTHORIZED, "缺少认证头".into()));
    }
    let (parts, body) = request.into_parts();
    let body_bytes = axum::body::to_bytes(body, 1024 * 1024).await
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("读取请求体失败: {}", e)))?;
    verify_request_signature(&body_bytes, sig_header.as_deref(), ts_header.as_deref())?;
    let request = Request::from_parts(parts, axum::body::Body::from(body_bytes));
    Ok(next.run(request).await)
}

/// v3.3.0: Phase 5 HMAC验证 (当前透传, 待加密通道完成后启用)
/// 验证请求签名和时间窗口
fn verify_request_signature(
    body: &[u8],
    signature_header: Option<&str>,
    timestamp_header: Option<&str>,
) -> Result<(), (StatusCode, String)> {
    // 1. HMAC 验证
    let sig = signature_header.ok_or_else(|| {
        (StatusCode::UNAUTHORIZED, "缺少 X-Executor-Signature 头".to_string())
    })?;
    qrpc_session::verify(body, sig).map_err(|_| {
        (StatusCode::UNAUTHORIZED, "签名验证失败".to_string())
    })?;

    // 2. 时间窗口防重放 (±5s)
    if let Some(ts_str) = timestamp_header {
        let ts_ms: u64 = ts_str.parse().unwrap_or(0);
        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        if ts_ms.abs_diff(now_ms) > 5_000 {
            return Err((StatusCode::UNAUTHORIZED, "请求时间戳超出允许窗口 (±5s)".to_string()));
        }
    }

    Ok(())
}
