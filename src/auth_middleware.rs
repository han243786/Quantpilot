use super::*;

// ── 多用户认证中间件 (v2.0.0) ──
// 1. DEV 模式 → 跳过认证, 使用默认用户 (user_id=0)
// 2. JWT Bearer token → 提取真实 user_id
// 3. API Key Bearer token → 使用默认用户 (user_id=0, 向后兼容)
// 4. 白名单路径 (/api/health, /api/auth/) → 跳过认证

pub(super) async fn api_key_auth(
    mut request: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    // ── 开发模式跳过认证 ──
    if std::env::var("QUANTPILOT_DEV")
        .map(|v| v == "true" || v == "1")
        .unwrap_or(false)
    {
        request.extensions_mut().insert(auth::UserId(0));
        return next.run(request).await;
    }

    // ── 白名单路径 ──
    let path = request.uri().path();
    if path == "/api/health" || path.starts_with("/api/auth/") {
        return next.run(request).await;
    }

    // 非 /api/ 路径免认证（如静态文件）
    if !path.starts_with("/api/") {
        return next.run(request).await;
    }

    // ── 获取 API Key ──
    let api_key = match std::env::var("QUANTPILOT_API_KEY") {
        Ok(key) if !key.is_empty() => key,
        _ => {
            use ring::rand::{SecureRandom, SystemRandom};
            static GENERATED_KEY: std::sync::OnceLock<String> = std::sync::OnceLock::new();
            let key = GENERATED_KEY.get_or_init(|| {
                let rng = SystemRandom::new();
                let mut bytes = [0u8; 16];
                rng.fill(&mut bytes).ok();
                bytes.iter().map(|b| format!("{:02x}", b)).collect::<String>()
            });
            safe_eprintln!(
                "[auth] QUANTPILOT_API_KEY 未设置, 已生成随机 key。请求需携带 Authorization: Bearer <KEY>"
            );
            key.clone()
        }
    };

    // ── 解析 Authorization header ──
    let auth_header = request
        .headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .unwrap_or("");

    let Some(token) = auth_header.strip_prefix("Bearer ") else {
        return unauthorized_response();
    };
    let token = token.trim();

    // ── JWT 认证 (token 包含 '.' 则为 JWT) ──
    if token.contains('.') {
        match auth::verify_token(token) {
            Ok(user) => {
                request.extensions_mut().insert(auth::UserId(user.id));
                return next.run(request).await;
            }
            Err(e) => {
                safe_eprintln!("[auth] JWT 验证失败: {}", e);
                // JWT 验证失败时继续尝试 API Key 认证 (向后兼容)
            }
        }
    }

    // ── API Key 认证 (向后兼容, 使用默认用户) ──
    if token == api_key.trim() {
        request.extensions_mut().insert(auth::UserId(0));
        return next.run(request).await;
    }

    unauthorized_response()
}

fn unauthorized_response() -> axum::response::Response {
    let problem = serde_json::json!({
        "error": "unauthorized",
        "message": "认证失败: 请使用有效 token 或设置 QUANTPILOT_DEV=true 跳过认证（仅开发环境）"
    });

    let body = axum::body::Body::from(
        serde_json::to_string(&problem)
            .unwrap_or_else(|_| r#"{"error":"unauthorized"}"#.to_string()),
    );

    axum::response::Response::builder()
        .status(StatusCode::UNAUTHORIZED)
        .header(axum::http::header::CONTENT_TYPE, "application/json")
        .body(body)
        .unwrap_or_else(|_| {
            axum::response::Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(axum::body::Body::empty())
                .unwrap_or_else(|_| axum::response::Response::new(axum::body::Body::empty()))
        })
}

// ── 单元测试 ──

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::Request, middleware, routing::get, Router};
    use std::sync::{Mutex, OnceLock};
    use tower::ServiceExt;

    /// 全局 env var 锁：防止并行测试中 env var 竞争
    static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

    fn env_lock() -> &'static Mutex<()> {
        ENV_LOCK.get_or_init(|| Mutex::new(()))
    }

    /// 辅助：在设置 env var 的上下文中执行闭包，执行完毕后恢复
    fn with_env_var<F>(key: &str, value: &str, f: F)
    where
        F: FnOnce(),
    {
        let guard = env_lock().lock().unwrap_or_else(|e| e.into_inner());
        let old = std::env::var(key).ok();
        std::env::set_var(key, value);
        f();
        match old {
            Some(v) => std::env::set_var(key, v),
            None => std::env::remove_var(key),
        }
        drop(guard);
    }

    /// 辅助：在清除 env var 的上下文中执行闭包
    fn without_env_var<F>(key: &str, f: F)
    where
        F: FnOnce(),
    {
        let guard = env_lock().lock().unwrap_or_else(|e| e.into_inner());
        let old = std::env::var(key).ok();
        std::env::remove_var(key);
        f();
        if let Some(v) = old {
            std::env::set_var(key, v);
        }
        drop(guard);
    }

    /// 用于测试的路由 handler
    async fn test_handler(user_id: auth::UserId) -> impl axum::response::IntoResponse {
        format!("user_id={}", user_id.0)
    }

    /// 构建测试用 Router
    fn build_test_router() -> Router {
        Router::new()
            .route("/api/test", get(test_handler))
            .route("/api/auth/register", get(|| async { "register" }))
            .route("/api/health", get(|| async { "health" }))
            .layer(middleware::from_fn(api_key_auth))
    }

    fn run_request(app: Router, uri: &str) -> axum::response::Response {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            app.oneshot(
                Request::builder().uri(uri).body(Body::empty()).unwrap(),
            )
            .await
            .unwrap()
        })
    }

    fn run_request_with_header(
        app: Router,
        uri: &str,
        header_name: &str,
        header_value: &str,
    ) -> axum::response::Response {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            app.oneshot(
                Request::builder()
                    .uri(uri)
                    .header(header_name, header_value)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap()
        })
    }

    // ── DEV 模式 ──

    #[test]
    fn test_dev_mode_bypasses_auth() {
        with_env_var("QUANTPILOT_DEV", "true", || {
            let app = build_test_router();
            let resp = run_request(app, "/api/test");
            assert_eq!(resp.status(), 200);
        });
    }

    #[test]
    fn test_dev_mode_inserts_user_id_zero() {
        with_env_var("QUANTPILOT_DEV", "true", || {
            let app = build_test_router();
            let resp = run_request(app, "/api/test");
            let body = rt_block_on(async { axum::body::to_bytes(resp.into_body(), 1024).await })
                .unwrap();
            let body_str = String::from_utf8(body.to_vec()).unwrap();
            assert_eq!(body_str, "user_id=0");
        });
    }

    #[test]
    fn test_dev_mode_uses_one_not_just_true() {
        with_env_var("QUANTPILOT_DEV", "1", || {
            let app = build_test_router();
            let resp = run_request(app, "/api/test");
            assert_eq!(resp.status(), 200);
        });
    }

    // ── 白名单路径 ──

    #[test]
    fn test_health_path_is_whitelisted() {
        without_env_var("QUANTPILOT_DEV", || {
            let app = build_test_router();
            let resp = run_request(app, "/api/health");
            assert_eq!(resp.status(), 200)
        });
    }

    #[test]
    fn test_auth_register_path_is_whitelisted() {
        without_env_var("QUANTPILOT_DEV", || {
            let app = build_test_router();
            let resp = run_request(app, "/api/auth/register");
            assert_eq!(resp.status(), 200)
        });
    }

    #[test]
    fn test_auth_login_path_is_whitelisted() {
        without_env_var("QUANTPILOT_DEV", || {
            let app = build_test_router();
            let resp = run_request(app, "/api/auth/login");
            // 路由不存在但应被中间件放行（返回 404 而非 401）
            assert_ne!(resp.status(), 401);
        });
    }

    #[test]
    fn test_non_api_path_is_whitelisted() {
        without_env_var("QUANTPILOT_DEV", || {
            let app = build_test_router();
            let resp = run_request(app, "/index.html");
            assert_eq!(resp.status(), 200);
        });
    }

    // ── 缺少 Authorization header ──

    #[test]
    fn test_missing_authorization_returns_401() {
        without_env_var("QUANTPILOT_DEV", || {
            let app = build_test_router();
            let resp = run_request(app, "/api/test");
            assert_eq!(resp.status(), 401);
        });
    }

    #[test]
    fn test_missing_authorization_returns_json() {
        without_env_var("QUANTPILOT_DEV", || {
            let app = build_test_router();
            let resp = run_request(app, "/api/test");
            let content_type = resp
                .headers()
                .get(axum::http::header::CONTENT_TYPE)
                .and_then(|v| v.to_str().ok())
                .unwrap_or("");
            assert!(content_type.contains("application/json"));
        });
    }

    // ── 无效 Bearer token ──

    #[test]
    fn test_invalid_bearer_token_returns_401() {
        without_env_var("QUANTPILOT_DEV", || {
            let app = build_test_router();
            let resp = run_request_with_header(
                app,
                "/api/test",
                "Authorization",
                "Bearer invalid_token_here",
            );
            assert_eq!(resp.status(), 401);
        });
    }

    #[test]
    fn test_bearer_with_wrong_scheme_returns_401() {
        without_env_var("QUANTPILOT_DEV", || {
            let app = build_test_router();
            let resp = run_request_with_header(
                app,
                "/api/test",
                "Authorization",
                "Basic dXNlcjpwYXNz",
            );
            assert_eq!(resp.status(), 401);
        });
    }

    #[test]
    fn test_empty_bearer_token_returns_401() {
        without_env_var("QUANTPILOT_DEV", || {
            let app = build_test_router();
            let resp = run_request_with_header(
                app,
                "/api/test",
                "Authorization",
                "Bearer ",
            );
            assert_eq!(resp.status(), 401);
        });
    }

    // ── API Key 认证 ──

    #[test]
    fn test_api_key_auth_with_correct_key() {
        with_env_var("QUANTPILOT_API_KEY", "my_test_api_key_12345", || {
            let app = build_test_router();
            let resp = run_request_with_header(
                app,
                "/api/test",
                "Authorization",
                "Bearer my_test_api_key_12345",
            );
            assert_eq!(resp.status(), 200);
        });
    }

    #[test]
    fn test_api_key_auth_wrong_key_returns_401() {
        with_env_var("QUANTPILOT_API_KEY", "correct_key", || {
            let app = build_test_router();
            let resp = run_request_with_header(
                app,
                "/api/test",
                "Authorization",
                "Bearer wrong_key",
            );
            assert_eq!(resp.status(), 401);
        });
    }

    #[test]
    fn test_api_key_auth_uses_default_user() {
        with_env_var("QUANTPILOT_API_KEY", "test_key_for_user_check", || {
            let app = build_test_router();
            let resp = run_request_with_header(
                app,
                "/api/test",
                "Authorization",
                "Bearer test_key_for_user_check",
            );
            let body =
                rt_block_on(async { axum::body::to_bytes(resp.into_body(), 1024).await }).unwrap();
            let body_str = String::from_utf8(body.to_vec()).unwrap();
            assert_eq!(body_str, "user_id=0");
        });
    }
}

/// 辅助函数：在同步测试中运行异步块
fn rt_block_on<F: std::future::Future>(f: F) -> F::Output {
    tokio::runtime::Runtime::new().unwrap().block_on(f)
}
