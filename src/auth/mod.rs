// ── 多用户认证系统 (QuantPilot v2.0.0) ──
// SQLite + JWT + bcrypt, 向后兼容默认用户 (user_id=0)

use std::path::Path;
use std::sync::OnceLock;

use axum::{extract::State, http::StatusCode, response::IntoResponse, Json, routing::post, Router};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};

// ── 核心类型 ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub username: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: i64,
    username: String,
    exp: usize,
}

/// 认证中间件使用的请求扩展: 提取 UserId 到 handler
#[derive(Debug, Clone, Copy)]
pub struct UserId(pub i64);

/// 从请求扩展中提取 UserId, 若不存在则返回默认用户 (id=0, 向后兼容)。
pub fn user_id_from_extensions(extensions: &axum::http::Extensions) -> UserId {
    extensions.get::<UserId>().copied().unwrap_or(UserId(0))
}

/// Axum 提取器: 允许 handler 直接通过 `user_id: auth::UserId` 获取当前用户。
#[axum::async_trait]
impl<S: Send + Sync> axum::extract::FromRequestParts<S> for UserId {
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        Ok(user_id_from_extensions(&parts.extensions))
    }
}

/// 构造带用户前缀的 BTreeMap 键: `{user_id}:{record_id}`。
/// 向后兼容: user_id=0 时键保持原样 (不加前缀), 与 v1.x 已持久化数据兼容。
pub fn scoped_key(user_id: &UserId, record_id: &str) -> String {
    if user_id.0 == 0 {
        record_id.to_string()
    } else {
        format!("{}:{}", user_id.0, record_id)
    }
}

// ── 请求/响应 DTO ──

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RegisterRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct AuthSuccessResponse {
    pub token: String,
    pub user: User,
}

#[derive(Debug, Serialize)]
pub struct AuthErrorResponse {
    pub error: String,
    pub message: String,
}

// ── JWT 密钥 (全局缓存, v2.1.1 持久化) ──

const JWT_SECRET_FILE: &str = "storage/.jwt_secret";

fn jwt_secret_bytes() -> &'static [u8] {
    static JWT_SECRET: OnceLock<Vec<u8>> = OnceLock::new();
    JWT_SECRET.get_or_init(|| {
        let env_key = std::env::var("QUANTPILOT_JWT_SECRET").unwrap_or_default();
        if !env_key.is_empty() {
            return env_key.into_bytes();
        }
        // 从 API_KEY 派生 (若存在)
        let api_key = std::env::var("QUANTPILOT_API_KEY").unwrap_or_default();
        if !api_key.is_empty() {
            let hash = ring::digest::digest(&ring::digest::SHA256, api_key.as_bytes());
            return hash.as_ref().to_vec();
        }
        // v2.1.1: 持久化JWT密钥到磁盘，确保重启后token仍然有效
        let path = std::path::Path::new(JWT_SECRET_FILE);
        if let Ok(existing) = std::fs::read(path) {
            if existing.len() >= 32 {
                return existing;
            }
        }
        // 生成随机 32 字节并持久化
        use ring::rand::SecureRandom;
        let rng = ring::rand::SystemRandom::new();
        let mut bytes = vec![0u8; 32];
        rng.fill(&mut bytes).expect("JWT 密钥生成失败: 系统熵池不足");
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let tmp = path.with_extension("tmp");
        if std::fs::write(&tmp, &bytes).is_ok() {
            let _ = std::fs::rename(&tmp, path);
        }
        bytes
    })
}

// ── 数据库初始化 ──

pub fn init_db(path: &Path) -> Result<Connection, String> {
    let conn =
        Connection::open(path).map_err(|e| format!("无法打开 SQLite 数据库: {}", e))?;

    // WAL 模式提升并发性能
    conn.execute_batch("PRAGMA journal_mode=WAL;")
        .map_err(|e| format!("无法设置 WAL 模式: {}", e))?;

    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS users (
            id       INTEGER PRIMARY KEY AUTOINCREMENT,
            username TEXT    NOT NULL UNIQUE,
            password_hash TEXT NOT NULL,
            created_at TEXT  NOT NULL DEFAULT (datetime('now'))
        );",
    )
    .map_err(|e| format!("无法创建 users 表: {}", e))?;

    // 确保默认用户存在 (user_id=0, 仅用于向后兼容, 不可登录)
    conn.execute(
        "INSERT OR IGNORE INTO users (id, username, password_hash) VALUES (0, '__default__', '')",
        [],
    )
    .map_err(|e| format!("无法创建默认用户: {}", e))?;

    Ok(conn)
}

// ── 注册 ──

pub fn register_user(conn: &Connection, username: &str, password: &str) -> Result<User, String> {
    let username = username.trim();
    if username.is_empty() {
        return Err("用户名不能为空".to_string());
    }
    if password.len() < 6 {
        return Err("密码长度不能少于 6 位".to_string());
    }
    if username.len() > 64 {
        return Err("用户名长度不能超过 64 个字符".to_string());
    }
    if password.len() > 128 {
        return Err("密码长度不能超过 128 个字符".to_string());
    }

    let password_hash =
        bcrypt::hash(password, bcrypt::DEFAULT_COST).map_err(|e| format!("密码加密失败: {}", e))?;

    conn.execute(
        "INSERT INTO users (username, password_hash) VALUES (?1, ?2)",
        rusqlite::params![username, password_hash],
    )
    .map_err(|e| {
        if e.to_string().contains("UNIQUE") {
            // 统一返回通用消息, 不泄露用户名存在性
            "注册失败，请稍后重试".to_string()
        } else {
            format!("注册失败: {}", e)
        }
    })?;

    let id = conn.last_insert_rowid();
    Ok(User {
        id,
        username: username.to_string(),
    })
}

// ── 登录 & JWT 生成 ──

pub fn login_user(conn: &Connection, username: &str, password: &str) -> Result<String, String> {
    let mut stmt = conn
        .prepare("SELECT id, username, password_hash FROM users WHERE username = ?1")
        .map_err(|e| format!("数据库查询失败: {}", e))?;

    let result: Result<(i64, String, String), rusqlite::Error> =
        stmt.query_row(rusqlite::params![username], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?))
        });

    let (id, uname, password_hash) = match result {
        Ok(data) => data,
        Err(rusqlite::Error::QueryReturnedNoRows) => {
            return Err("用户名或密码错误".to_string());
        }
        Err(e) => return Err(format!("数据库查询失败: {}", e)),
    };

    let valid =
        bcrypt::verify(password, &password_hash).map_err(|e| format!("密码验证失败: {}", e))?;

    if !valid {
        return Err("用户名或密码错误".to_string());
    }

    let exp = chrono::Utc::now().timestamp() as usize + 86_400; // 24h
    let claims = Claims {
        sub: id,
        username: uname,
        exp,
    };

    let token = jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret(jwt_secret_bytes()),
    )
    .map_err(|e| format!("生成 token 失败: {}", e))?;

    Ok(token)
}

// ── JWT 验证 ──

pub fn verify_token(token: &str) -> Result<User, String> {
    let token_data = jsonwebtoken::decode::<Claims>(
        token,
        &jsonwebtoken::DecodingKey::from_secret(jwt_secret_bytes()),
        &{
            let mut validation = jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::HS256);
            validation.leeway = 0;
            validation
        },
    )
    .map_err(|e| match e.kind() {
        jsonwebtoken::errors::ErrorKind::ExpiredSignature => {
            "token 已过期，请重新登录".to_string()
        }
        jsonwebtoken::errors::ErrorKind::InvalidToken => "无效的 token".to_string(),
        _ => format!("token 验证失败: {}", e),
    })?;

    Ok(User {
        id: token_data.claims.sub,
        username: token_data.claims.username,
    })
}

// ── Axum 路由 handlers ──

use super::AppState;

async fn register_handler(
    State(state): State<AppState>,
    Json(req): Json<RegisterRequest>,
) -> impl IntoResponse {
    let db = match &state.db {
        Some(db) => db,
        None => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(serde_json::json!({
                    "error": "auth_unavailable",
                    "message": "认证服务暂不可用 (数据库未加载)"
                })),
            )
                .into_response();
        }
    };

    let db = db.lock().unwrap_or_else(|e| e.into_inner());
    match register_user(&db, &req.username, &req.password) {
        Ok(user) => (
            StatusCode::CREATED,
            Json(serde_json::json!({
                "user": user,
                "message": "注册成功"
            })),
        )
            .into_response(),
        Err(msg) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "registration_failed",
                "message": msg
            })),
        )
            .into_response(),
    }
}

async fn login_handler(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> impl IntoResponse {
    let db = match &state.db {
        Some(db) => db,
        None => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(serde_json::json!({
                    "error": "auth_unavailable",
                    "message": "认证服务暂不可用 (数据库未加载)"
                })),
            )
                .into_response();
        }
    };

    let db = db.lock().unwrap_or_else(|e| e.into_inner());
    match login_user(&db, &req.username, &req.password) {
        Ok(token) => {
            // 从 claims 解码出用户信息
            let user = verify_token(&token).ok();
            (
                StatusCode::OK,
                Json(serde_json::json!({
                    "token": token,
                    "user": user
                })),
            )
                .into_response()
        }
        Err(msg) => (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({
                "error": "login_failed",
                "message": msg
            })),
        )
            .into_response(),
    }
}

// v2.3.0: JWT 令牌刷新端点
async fn refresh_handler(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
) -> impl IntoResponse {
    let token = headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .unwrap_or("");

    if token.is_empty() {
        return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({
            "error": "auth_required",
            "message": "请提供有效的 Bearer token"
        }))).into_response();
    }

    match verify_token(token) {
        Ok(user) => {
            let username = user.username.clone();
            let new_token = login_user_by_id(&user.id, &username).unwrap_or_else(|_| token.to_string());
            (StatusCode::OK, Json(serde_json::json!({
                "token": new_token,
                "user": user
            }))).into_response()
        }
        Err(e) => (StatusCode::UNAUTHORIZED, Json(serde_json::json!({
            "error": "token_invalid",
            "message": e
        }))).into_response(),
    }
}

/// 基于用户 ID 直接签发新 token (用于刷新)
fn login_user_by_id(user_id: &i64, username: &str) -> Result<String, String> {
    let exp = chrono::Utc::now().timestamp() as usize + 86_400; // 24h
    let claims = Claims {
        sub: *user_id,
        username: username.to_string(),
        exp,
    };
    jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret(jwt_secret_bytes()),
    )
    .map_err(|e| format!("生成 token 失败: {}", e))
}

/// 注册认证相关路由 (供 app_router 调用)
/// v2.2.1: 添加独立的登录速率限制
/// v2.3.0: 添加 JWT 刷新端点
pub(crate) fn register_auth_routes(router: Router<AppState>) -> Router<AppState> {
    router
        .route("/api/auth/register", post(register_handler))
        .route("/api/auth/login", post(login_handler))
        .route("/api/auth/refresh", post(refresh_handler))
        .layer(axum::middleware::from_fn(
            super::rate_limiter::auth_rate_limit_middleware,
        ))
}

// ── 单元测试 ──

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    /// 辅助：创建内存 SQLite 数据库并建表、插入默认用户
    fn setup_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS users (
                id       INTEGER PRIMARY KEY AUTOINCREMENT,
                username TEXT    NOT NULL UNIQUE,
                password_hash TEXT NOT NULL,
                created_at TEXT  NOT NULL DEFAULT (datetime('now'))
            );",
        )
        .unwrap();
        conn.execute(
            "INSERT OR IGNORE INTO users (id, username, password_hash) VALUES (0, '__default__', '')",
            [],
        )
        .unwrap();
        conn
    }

    // ── register_user ──

    #[test]
    fn test_register_valid_user() {
        let conn = setup_db();
        let result = register_user(&conn, "valid_user", "password123");
        assert!(result.is_ok());
        let user = result.unwrap();
        assert_eq!(user.username, "valid_user");
        assert!(user.id > 0);
    }

    #[test]
    fn test_register_empty_username() {
        let conn = setup_db();
        let result = register_user(&conn, "", "password123");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("不能为空"));
    }

    #[test]
    fn test_register_short_password() {
        let conn = setup_db();
        let result = register_user(&conn, "short_pw_user", "12345");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("不能少于 6 位"));
    }

    #[test]
    fn test_register_username_too_long() {
        let conn = setup_db();
        let long_name = "a".repeat(65);
        let result = register_user(&conn, &long_name, "password123");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("不能超过 64 个字符"));
    }

    #[test]
    fn test_register_password_too_long() {
        let conn = setup_db();
        let long_pass = "a".repeat(129);
        let result = register_user(&conn, "long_pw_user", &long_pass);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("不能超过 128 个字符"));
    }

    #[test]
    fn test_register_duplicate_username() {
        let conn = setup_db();
        register_user(&conn, "dup_user", "password123").unwrap();
        let result = register_user(&conn, "dup_user", "password456");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("注册失败"));
    }

    #[test]
    fn test_register_username_is_trimmed() {
        let conn = setup_db();
        let result = register_user(&conn, "  spaced_user  ", "password123");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().username, "spaced_user");
    }

    // ── login_user ──

    #[test]
    fn test_login_valid_returns_jwt() {
        let conn = setup_db();
        register_user(&conn, "login_valid", "password123").unwrap();
        let token = login_user(&conn, "login_valid", "password123");
        assert!(token.is_ok());
        let token_str = token.unwrap();
        assert!(!token_str.is_empty());
        // JWT 格式：三个由点分隔的 base64 段
        assert_eq!(token_str.matches('.').count(), 2);
    }

    #[test]
    fn test_login_wrong_password() {
        let conn = setup_db();
        register_user(&conn, "login_wrong", "password123").unwrap();
        let result = login_user(&conn, "login_wrong", "wrong_password");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("错误"));
    }

    #[test]
    fn test_login_nonexistent_user() {
        let conn = setup_db();
        let result = login_user(&conn, "nonexistent_user", "password123");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("错误"));
    }

    #[test]
    fn test_login_default_user_cannot_login() {
        let conn = setup_db();
        // 默认用户 password_hash 为空，任何密码均不匹配
        let result = login_user(&conn, "__default__", "");
        assert!(result.is_err());
    }

    // ── verify_token ──

    #[test]
    fn test_verify_valid_token() {
        let conn = setup_db();
        register_user(&conn, "verify_valid", "password123").unwrap();
        let token = login_user(&conn, "verify_valid", "password123").unwrap();
        let user = verify_token(&token);
        assert!(user.is_ok());
        assert_eq!(user.unwrap().username, "verify_valid");
    }

    #[test]
    fn test_verify_expired_token() {
        use jsonwebtoken::{encode, EncodingKey, Header};

        let claims = Claims {
            sub: 1,
            username: "expired_user".to_string(),
            exp: 1_000_000_000, // 2001-09-09, 已过期
        };
        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(jwt_secret_bytes()),
        )
        .unwrap();
        let result = verify_token(&token);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("已过期"));
    }

    #[test]
    fn test_verify_tampered_token() {
        let conn = setup_db();
        register_user(&conn, "verify_tamper", "password123").unwrap();
        let token = login_user(&conn, "verify_tamper", "password123").unwrap();
        // 篡改 token 的最后一个字符
        let tampered = format!("{}x", &token[..token.len() - 1]);
        let result = verify_token(&tampered);
        assert!(result.is_err());
    }

    #[test]
    fn test_verify_empty_token() {
        let result = verify_token("");
        assert!(result.is_err());
    }

    #[test]
    fn test_verify_garbage_token() {
        let result = verify_token("this.is.not.a.valid.jwt");
        assert!(result.is_err());
    }

    // ── jwt_secret_bytes ──

    #[test]
    fn test_jwt_secret_consistent_across_calls() {
        let first = jwt_secret_bytes().to_vec();
        let second = jwt_secret_bytes().to_vec();
        assert_eq!(first, second);
        assert!(!first.is_empty());
        // 随机生成的 secret 应为 32 字节
        assert_eq!(first.len(), 32);
    }

    // ── init_db ──

    #[test]
    fn test_init_db_creates_tables_and_default_user() {
        let dir = std::env::temp_dir().join(format!(
            "quantpilot_auth_test_{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&dir).ok();
        let db_path = dir.join("test_init.db");
        // 清理上次测试遗留文件
        let _ = std::fs::remove_file(&db_path);

        let conn = init_db(&db_path).unwrap();

        // 验证 users 表存在
        let table_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='users'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(table_count, 1);

        // 验证默认用户存在
        let (id, username): (i64, String) = conn
            .query_row(
                "SELECT id, username FROM users WHERE id = 0",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        assert_eq!(id, 0);
        assert_eq!(username, "__default__");

        // 验证总用户数
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM users", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 1);

        // 清理
        drop(conn);
        let _ = std::fs::remove_file(&db_path);
        let _ = std::fs::remove_file(dir.join("test_init.db-wal"));
        let _ = std::fs::remove_file(dir.join("test_init.db-shm"));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
