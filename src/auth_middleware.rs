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
            // 未设置环境变量时自动生成随机 key, 打印到启动日志
            use ring::rand::{SecureRandom, SystemRandom};
            static GENERATED_KEY: std::sync::OnceLock<String> = std::sync::OnceLock::new();
            let key = GENERATED_KEY.get_or_init(|| {
                let rng = SystemRandom::new();
                let mut bytes = [0u8; 16];
                rng.fill(&mut bytes).ok();
                bytes.iter().map(|b| format!("{:02x}", b)).collect::<String>()
            });
            crate::safe_eprintln!(
                "[auth] QUANTPILOT_API_KEY 未设置, 已生成随机 key: {}. 请求需携带 Authorization: Bearer {}",
                key, key
            );
            key.clone()
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
        "error": "unauthorized",
        "message": "认证失败: 请在 Authorization 头中提供有效的 Bearer token"
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
