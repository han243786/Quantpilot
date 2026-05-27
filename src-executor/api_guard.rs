use axum::{extract::Request, http::StatusCode, middleware::Next, response::Response};
/// v3.7.0: API 守卫中间件
/// 解密请求体 (AES-256-GCM) → 验证 HMAC 签名 → 时间窗口防重放 (±5s)
use std::collections::BTreeMap;
use std::sync::{Mutex, OnceLock};

/// API 守卫中间件 — v3.5.0: Phase 5 HMAC验证激活
pub async fn api_guard_middleware(
    request: Request,
    next: Next,
) -> Result<Response, (StatusCode, String)> {
    if request.uri().path() == "/api/executor/health" {
        return Ok(next.run(request).await);
    }
    // v3.7.0: 开发环境可降级, 生产强制验证
    if std::env::var("QUANTPILOT_EXECUTOR_INSECURE").is_ok_and(|v| v == "true" || v == "1") {
        return Ok(next.run(request).await);
    }
    let sig_header = request
        .headers()
        .get("x-executor-signature")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());
    let ts_header = request
        .headers()
        .get("x-executor-timestamp")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());
    if sig_header.is_none() {
        return Err((StatusCode::UNAUTHORIZED, "缺少认证头".into()));
    }
    let method = request.method().as_str().to_string();
    let path_and_query = request
        .uri()
        .path_and_query()
        .map(|v| v.as_str().to_string())
        .unwrap_or_else(|| request.uri().path().to_string());
    let (parts, body) = request.into_parts();
    let body_bytes = axum::body::to_bytes(body, 1024 * 1024)
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("读取请求体失败: {}", e)))?;
    verify_request_signature(
        &method,
        &path_and_query,
        &body_bytes,
        sig_header.as_deref(),
        ts_header.as_deref(),
    )?;
    let request = Request::from_parts(parts, axum::body::Body::from(body_bytes));
    Ok(next.run(request).await)
}

/// v3.3.0: Phase 5 HMAC验证 (当前透传, 待加密通道完成后启用)
/// 验证请求签名和时间窗口
fn verify_request_signature(
    method: &str,
    path_and_query: &str,
    body: &[u8],
    signature_header: Option<&str>,
    timestamp_header: Option<&str>,
) -> Result<(), (StatusCode, String)> {
    // 1. 时间戳必须参与签名，避免 body-only 请求跨端点重放。
    let ts_str = timestamp_header.ok_or_else(|| {
        (
            StatusCode::UNAUTHORIZED,
            "缺少 X-Executor-Timestamp 头".to_string(),
        )
    })?;
    let ts_ms: u64 = ts_str.parse().map_err(|_| {
        (
            StatusCode::UNAUTHORIZED,
            "X-Executor-Timestamp 格式无效".to_string(),
        )
    })?;

    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;
    if ts_ms.abs_diff(now_ms) > 5_000 {
        return Err((
            StatusCode::UNAUTHORIZED,
            "请求时间戳超出允许窗口 (±5s)".to_string(),
        ));
    }

    // 2. HMAC 验证覆盖 method + path/query + timestamp + body。
    let sig = signature_header.ok_or_else(|| {
        (
            StatusCode::UNAUTHORIZED,
            "缺少 X-Executor-Signature 头".to_string(),
        )
    })?;
    qrpc_session::verify_request(method, path_and_query, ts_ms, body, sig)
        .map_err(|_| (StatusCode::UNAUTHORIZED, "签名验证失败".to_string()))?;
    reject_replay(sig, ts_ms)?;

    Ok(())
}

fn reject_replay(signature: &str, timestamp_ms: u64) -> Result<(), (StatusCode, String)> {
    static SEEN_SIGNATURES: OnceLock<Mutex<BTreeMap<String, u64>>> = OnceLock::new();
    let seen = SEEN_SIGNATURES.get_or_init(|| Mutex::new(BTreeMap::new()));
    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;
    let mut seen = seen.lock().unwrap_or_else(|e| e.into_inner());
    seen.retain(|_, observed_at| now_ms.saturating_sub(*observed_at) <= 5_000);
    let replay_key = format!("{}:{}", timestamp_ms, signature.trim());
    if seen.contains_key(&replay_key) {
        return Err((
            StatusCode::UNAUTHORIZED,
            "请求签名已使用，疑似重放".to_string(),
        ));
    }
    seen.insert(replay_key, now_ms);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ensure_session_key() {
        let path = std::env::temp_dir().join(format!(
            "quantpilot-executor-api-guard-session-{}",
            std::process::id()
        ));
        let old = std::env::var_os("QUANTPILOT_SESSION_KEY_PATH");
        std::env::set_var("QUANTPILOT_SESSION_KEY_PATH", &path);
        let _ = qrpc_session::init_session_key();
        match old {
            Some(value) => std::env::set_var("QUANTPILOT_SESSION_KEY_PATH", value),
            None => std::env::remove_var("QUANTPILOT_SESSION_KEY_PATH"),
        }
        let _ = std::fs::remove_file(path);
    }

    fn now_ms() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64
    }

    #[test]
    fn request_signature_requires_timestamp() {
        ensure_session_key();
        let body = br#"{"ok":true}"#;
        let ts = now_ms();
        let sig = qrpc_session::sign_request("POST", "/api/executor/strategies", ts, body).unwrap();

        let result =
            verify_request_signature("POST", "/api/executor/strategies", body, Some(&sig), None);

        assert!(result.is_err());
    }

    #[test]
    fn request_signature_binds_path_and_rejects_replay() {
        ensure_session_key();
        let body = br#"{"ok":true}"#;
        let ts = now_ms();
        let sig = qrpc_session::sign_request("POST", "/api/executor/strategies", ts, body).unwrap();

        assert!(verify_request_signature(
            "POST",
            "/api/executor/mode",
            body,
            Some(&sig),
            Some(&ts.to_string()),
        )
        .is_err());
        assert!(verify_request_signature(
            "POST",
            "/api/executor/strategies",
            body,
            Some(&sig),
            Some(&ts.to_string()),
        )
        .is_ok());
        assert!(verify_request_signature(
            "POST",
            "/api/executor/strategies",
            body,
            Some(&sig),
            Some(&ts.to_string()),
        )
        .is_err());
    }
}
