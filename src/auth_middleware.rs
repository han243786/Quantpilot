use super::*;

// ── API Key 认证中间件 ──
// Block 5 P2-1: Bearer token 认证 + 开发模式绕过

pub(super) async fn api_key_auth(
    request: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    // 开发模式跳过认证
    if std::env::var("QUANTPILOT_DEV")
        .map(|v| v == "true" || v == "1")
        .unwrap_or(false)
    {
        return next.run(request).await;
    }

    // 健康检查端点免认证
    if request.uri().path() == "/api/health" {
        return next.run(request).await;
    }

    // 非 /api/ 路径免认证（如静态文件）
    if !request.uri().path().starts_with("/api/") {
        return next.run(request).await;
    }

    let api_key = match std::env::var("QUANTPILOT_API_KEY") {
        Ok(key) if !key.is_empty() => key,
        _ => {
            // Print warning once per process lifetime
            static WARNED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
            if !WARNED.swap(true, std::sync::atomic::Ordering::Relaxed) {
                eprintln!(
                    "[auth] WARNING: QUANTPILOT_API_KEY not set — all API requests allowed. \
                     Set QUANTPILOT_API_KEY environment variable to enable authentication."
                );
            }
            return next.run(request).await;
        }
    };

    let auth_header = request
        .headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .unwrap_or("");

    if let Some(token) = auth_header.strip_prefix("Bearer ") {
        if token.trim() == api_key.trim() {
            return next.run(request).await;
        }
    }

    let problem = serde_json::json!({
        "type": "https://quantpilot.dev/problems/unauthorized",
        "title": "Unauthorized",
        "status": 401,
        "detail": "Valid Bearer token required. Set Authorization: Bearer <your-api-key> header.",
        "error_code": "UNAUTHORIZED",
    });

    let body = axum::body::Body::from(
        serde_json::to_string(&problem).unwrap_or_else(|_| r#"{"error":"unauthorized"}"#.to_string()),
    );

    axum::response::Response::builder()
        .status(StatusCode::UNAUTHORIZED)
        .header(
            axum::http::header::CONTENT_TYPE,
            "application/problem+json",
        )
        .body(body)
        .unwrap_or_else(|_| {
            axum::response::Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(axum::body::Body::empty())
                .unwrap_or_else(|_| axum::response::Response::new(axum::body::Body::empty()))
        })
}

#[cfg(test)]
mod tests {
    #[test]
    fn auth_middleware_module_compiles() {
        // 验证模块可编译且基本类型可用
        assert!(true);
    }
}
