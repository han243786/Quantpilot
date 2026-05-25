// ── 多用户认证系统 (QuantPilot v2.0.0) ──
// SQLite + JWT + bcrypt, 向后兼容默认用户 (user_id=0)

use std::path::Path;
use std::sync::{Mutex, OnceLock};

use axum::{extract::State, http::StatusCode, response::IntoResponse, routing::post, Json, Router};
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
static JWT_SECRET_INIT_LOCK: Mutex<()> = Mutex::new(());

fn jwt_secret_bytes() -> Result<&'static [u8], String> {
    static JWT_SECRET: OnceLock<Vec<u8>> = OnceLock::new();
    if let Some(secret) = JWT_SECRET.get() {
        return Ok(secret);
    }
    let _guard = JWT_SECRET_INIT_LOCK
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    if let Some(secret) = JWT_SECRET.get() {
        return Ok(secret);
    }

    let env_key = std::env::var("QUANTPILOT_JWT_SECRET").unwrap_or_default();
    let secret = if !env_key.is_empty() {
        env_key.into_bytes()
    } else {
        let api_key = std::env::var("QUANTPILOT_API_KEY").unwrap_or_default();
        if !api_key.is_empty() {
            let hash = ring::digest::digest(&ring::digest::SHA256, api_key.as_bytes());
            hash.as_ref().to_vec()
        } else {
            let path = std::path::Path::new(JWT_SECRET_FILE);
            if let Ok(existing) = std::fs::read(path) {
                if existing.len() >= 32 {
                    existing
                } else {
                    generate_and_persist_jwt_secret(path)?
                }
            } else {
                generate_and_persist_jwt_secret(path)?
            }
        }
    };

    let _ = JWT_SECRET.set(secret);
    JWT_SECRET
        .get()
        .map(|secret| secret.as_slice())
        .ok_or_else(|| "JWT 密钥初始化失败".to_string())
}

fn generate_and_persist_jwt_secret(path: &Path) -> Result<Vec<u8>, String> {
    use ring::rand::SecureRandom;
    let rng = ring::rand::SystemRandom::new();
    let mut bytes = vec![0u8; 32];
    rng.fill(&mut bytes)
        .map_err(|_| "JWT 密钥生成失败: 系统熵池不足".to_string())?;
    crate::storage_lifecycle::atomic_write_secret_file(path, &bytes)
        .map_err(|error| format!("JWT 密钥持久化失败: {}", error))?;
    Ok(bytes)
}

// ── 数据库初始化 ──

pub fn init_db(path: &Path) -> Result<Connection, String> {
    let conn = Connection::open(path).map_err(|e| format!("无法打开 SQLite 数据库: {}", e))?;

    // WAL 模式提升并发性能
    conn.execute_batch("PRAGMA journal_mode=WAL;")
        .map_err(|e| format!("无法设置 WAL 模式: {}", e))?;

    // P2-1: 启用外键约束
    conn.execute_batch("PRAGMA foreign_keys = ON;")
        .map_err(|e| format!("无法设置外键约束: {}", e))?;

    // v3.5.1: 读取当前 schema 版本号, 为未来迁移提供基线
    // 未来迁移用 PRAGMA user_version 判断升级路径
    let current_version: i32 = conn
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .map_err(|e| format!("无法读取 schema 版本: {}", e))?;

    if current_version == 0 {
        conn.execute_batch("PRAGMA user_version = 1;")
            .map_err(|e| format!("无法设置 schema 版本: {}", e))?;
    }

    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS users (
            id       INTEGER PRIMARY KEY AUTOINCREMENT,
            username TEXT    NOT NULL UNIQUE,
            password_hash TEXT NOT NULL,
            created_at TEXT  NOT NULL DEFAULT (datetime('now'))
        );",
    )
    .map_err(|e| format!("无法创建 users 表: {}", e))?;

    // v3.5.0: 刷新令牌哈希表 (用于轮换检测与重放防御)
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS refresh_tokens (
            token_hash TEXT PRIMARY KEY,
            user_id    INTEGER NOT NULL,
            family_id  TEXT    NOT NULL,
            created_at INTEGER NOT NULL,
            revoked    INTEGER NOT NULL DEFAULT 0,
            FOREIGN KEY (user_id) REFERENCES users(id)
        );

        CREATE TABLE IF NOT EXISTS revoked_token_families (
            family_id  TEXT PRIMARY KEY,
            revoked_at INTEGER NOT NULL
        );",
    )
    .map_err(|e| format!("无法创建 refresh_tokens 表: {}", e))?;

    // P1-8: refresh_tokens 表索引，优化轮换与清理性能
    conn.execute_batch(
        "CREATE INDEX IF NOT EXISTS idx_refresh_tokens_family ON refresh_tokens(family_id);
         CREATE INDEX IF NOT EXISTS idx_refresh_tokens_user ON refresh_tokens(user_id);
         CREATE INDEX IF NOT EXISTS idx_refresh_tokens_created ON refresh_tokens(created_at);",
    )
    .map_err(|e| format!("无法创建 refresh_tokens 索引: {}", e))?;

    // P1-7: 启动时清理 30 天前的过期令牌 (TTL)
    conn.execute_batch(
        "DELETE FROM refresh_tokens WHERE created_at < unixepoch() - 2592000;
         DELETE FROM revoked_token_families WHERE revoked_at < unixepoch() - 2592000;",
    )
    .map_err(|e| format!("清理过期令牌失败: {}", e))?;

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

/// 从数据库查询用户凭证 (不验证密码)
fn query_user_credentials(
    conn: &Connection,
    username: &str,
) -> Result<(i64, String, String), String> {
    let mut stmt = conn
        .prepare("SELECT id, username, password_hash FROM users WHERE username = ?1")
        .map_err(|e| format!("数据库查询失败: {}", e))?;

    let result: Result<(i64, String, String), rusqlite::Error> = stmt
        .query_row(rusqlite::params![username], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?))
        });

    match result {
        Ok(data) => Ok(data),
        Err(rusqlite::Error::QueryReturnedNoRows) => Err("用户名或密码错误".to_string()),
        Err(e) => Err(format!("数据库查询失败: {}", e)),
    }
}

fn generate_jwt(id: i64, username: &str) -> Result<String, String> {
    let exp = chrono::Utc::now().timestamp() as usize + 86_400; // 24h
    let claims = Claims {
        sub: id,
        username: username.to_string(),
        exp,
    };
    jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret(jwt_secret_bytes()?),
    )
    .map_err(|e| format!("生成 token 失败: {}", e))
}

/// 同步版本: 在非异步上下文中使用 (测试/内部调用)
/// 注意: 此函数在调用线程中执行 bcrypt, 异步上下文中请使用 login_handler
pub fn login_user(conn: &Connection, username: &str, password: &str) -> Result<String, String> {
    let (id, uname, password_hash) = query_user_credentials(conn, username)?;

    let valid =
        bcrypt::verify(password, &password_hash).map_err(|e| format!("密码验证失败: {}", e))?;

    if !valid {
        return Err("用户名或密码错误".to_string());
    }

    generate_jwt(id, &uname)
}

// ── JWT 验证 ──

pub fn verify_token(token: &str) -> Result<User, String> {
    let token_data = jsonwebtoken::decode::<Claims>(
        token,
        &jsonwebtoken::DecodingKey::from_secret(jwt_secret_bytes()?),
        &{
            let mut validation = jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::HS256);
            validation.leeway = 0;
            validation
        },
    )
    .map_err(|e| match e.kind() {
        jsonwebtoken::errors::ErrorKind::ExpiredSignature => "token 已过期，请重新登录".to_string(),
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

    // 提前验证参数 (不需要锁)
    let username = req.username.trim().to_string();
    if username.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "bad_request",
                "message": "用户名不能为空"
            })),
        )
            .into_response();
    }
    if req.password.len() < 6 {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "bad_request",
                "message": "密码长度不能少于 6 位"
            })),
        )
            .into_response();
    }

    // bcrypt hash 在 spawn_blocking 中执行, 不阻塞 tokio 工作线程
    let password = req.password.clone();
    let password_hash =
        match tokio::task::spawn_blocking(move || bcrypt::hash(&password, bcrypt::DEFAULT_COST))
            .await
        {
            Ok(Ok(hash)) => hash,
            Ok(Err(e)) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({
                        "error": "internal_error",
                        "message": format!("密码加密失败: {}", e)
                    })),
                )
                    .into_response();
            }
            Err(_) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({
                        "error": "internal_error",
                        "message": "服务内部错误"
                    })),
                )
                    .into_response();
            }
        };

    // 数据库写入在 spawn_blocking 中执行, 不阻塞 tokio 工作线程
    let db_clone = db.clone();
    let username_for_db = username.clone();
    let insert_result = tokio::task::spawn_blocking(move || {
        let db = db_clone.lock().unwrap_or_else(|e| e.into_inner());
        db.execute(
            "INSERT INTO users (username, password_hash) VALUES (?1, ?2)",
            rusqlite::params![username_for_db, password_hash],
        )?;
        Ok::<i64, rusqlite::Error>(db.last_insert_rowid())
    })
    .await;

    match insert_result {
        Ok(Ok(id)) => {
            let user = User { id, username };
            (
                StatusCode::CREATED,
                Json(serde_json::json!({
                    "user": user,
                    "message": "注册成功"
                })),
            )
                .into_response()
        }
        Ok(Err(e)) => {
            let msg = if e.to_string().contains("UNIQUE") {
                "注册失败，请稍后重试".to_string()
            } else {
                format!("注册失败: {}", e)
            };
            (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": "registration_failed",
                    "message": msg
                })),
            )
                .into_response()
        }
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "error": "internal_error",
                "message": "服务内部错误"
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

    // Step 1: 查询用户凭证 (SQLite 在 spawn_blocking 中执行)
    let db_clone = db.clone();
    let username_for_lookup = req.username.clone();
    let credentials_result = tokio::task::spawn_blocking(move || {
        let db = db_clone.lock().unwrap_or_else(|e| e.into_inner());
        query_user_credentials(&db, &username_for_lookup)
    })
    .await;
    let (id, uname, password_hash) = match credentials_result {
        Ok(Ok(data)) => data,
        Ok(Err(msg)) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({
                    "error": "login_failed",
                    "message": msg
                })),
            )
                .into_response();
        }
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "internal_error",
                    "message": "服务内部错误"
                })),
            )
                .into_response();
        }
    };

    // Step 2: bcrypt 验证在 spawn_blocking 中执行, 不阻塞 tokio 工作线程
    let password = req.password.clone();
    let valid = match tokio::task::spawn_blocking(move || bcrypt::verify(&password, &password_hash))
        .await
    {
        Ok(Ok(ok)) => ok,
        Ok(Err(e)) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "internal_error",
                    "message": format!("密码验证失败: {}", e)
                })),
            )
                .into_response();
        }
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "internal_error",
                    "message": "服务内部错误"
                })),
            )
                .into_response();
        }
    };

    if !valid {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({
                "error": "login_failed",
                "message": "用户名或密码错误"
            })),
        )
            .into_response();
    }

    // Step 3: 生成 JWT + 刷新令牌 (spawn_blocking, 包含CPU密集+DB操作)
    let db_clone = db.clone();
    let uname_clone = uname.clone();
    let token_result = tokio::task::spawn_blocking(move || {
        let token = generate_jwt(id, &uname_clone)?;
        let db = db_clone.lock().unwrap_or_else(|e| e.into_inner());
        let (rt, _) =
            create_refresh_token(&db, id).unwrap_or_else(|_| (String::new(), String::new()));
        Ok::<_, String>((token, rt))
    })
    .await;

    match token_result {
        Ok(Ok((token, rt))) => {
            let user = User {
                id,
                username: uname,
            };
            (
                StatusCode::OK,
                Json(serde_json::json!({
                    "token": token,
                    "refresh_token": if rt.is_empty() { serde_json::Value::Null } else { serde_json::json!(rt) },
                    "user": user
                })),
            ).into_response()
        }
        Ok(Err(msg)) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "error": "jwt_error",
                "message": msg
            })),
        )
            .into_response(),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "error": "internal_error",
                "message": "服务内部错误"
            })),
        )
            .into_response(),
    }
}

// v3.5.0: 刷新令牌轮换 + 重放检测
// 请求体: {"access_token": "...", "refresh_token": "..."}
// 成功后返回新的 access_token + refresh_token, 旧 refresh_token 立即失效
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RefreshRequest {
    access_token: String,
    refresh_token: String,
}

async fn refresh_handler(
    State(state): State<AppState>,
    Json(req): Json<RefreshRequest>,
) -> impl IntoResponse {
    // Step 1: 验证 access_token (确认用户身份)
    let user = match verify_token(&req.access_token) {
        Ok(user) => user,
        Err(e) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({
                    "error": "token_invalid",
                    "message": e
                })),
            )
                .into_response();
        }
    };

    // Step 2: 获取数据库连接
    let db = match &state.db {
        Some(db) => db,
        None => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(serde_json::json!({
                    "error": "auth_unavailable",
                    "message": "认证服务暂不可用"
                })),
            )
                .into_response();
        }
    };

    // Step 3: 生成新的 access token (在轮换之前, 快速失败)
    let new_access_token = match login_user_by_id(&user.id, &user.username) {
        Ok(token) => token,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "jwt_error",
                    "message": e
                })),
            )
                .into_response();
        }
    };

    // v3.5.0: 刷新令牌轮换 (在 spawn_blocking 中执行 SQLite 操作, 不阻塞 tokio 工作线程)
    let old_hash = hash_token(&req.refresh_token);
    let db_arc = db.clone();
    let user_id = user.id;
    let rotate_result = tokio::task::spawn_blocking(move || {
        let db = db_arc.lock().unwrap_or_else(|e| e.into_inner());
        rotate_refresh_token(&db, user_id, &old_hash)
    })
    .await;

    match rotate_result {
        Ok(Ok((new_refresh_token, _new_hash, _family_id))) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "token": new_access_token,
                "refresh_token": new_refresh_token,
                "user": user
            })),
        )
            .into_response(),
        Ok(Err(msg)) => {
            let is_replay = msg.contains("重放");
            let status = if is_replay {
                StatusCode::GONE
            } else {
                StatusCode::UNAUTHORIZED
            };
            (
                status,
                Json(serde_json::json!({
                    "error": if is_replay { "token_replay" } else { "refresh_token_invalid" },
                    "message": msg
                })),
            )
                .into_response()
        }
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "error": "internal_error",
                "message": "服务内部错误"
            })),
        )
            .into_response(),
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
        &jsonwebtoken::EncodingKey::from_secret(jwt_secret_bytes()?),
    )
    .map_err(|e| format!("生成 token 失败: {}", e))
}

// ── v3.5.0: 刷新令牌轮换与重放检测 ──

fn generate_refresh_token() -> String {
    use ring::rand::SecureRandom;
    // v3.7.1: 全局缓存 SystemRandom, 避免每次调用创建新的 CSPRNG 句柄 (Windows BCryptOpenAlgorithmProvider)
    static RNG: OnceLock<ring::rand::SystemRandom> = OnceLock::new();
    let rng = RNG.get_or_init(|| ring::rand::SystemRandom::new());
    let mut bytes = [0u8; 32];
    rng.fill(&mut bytes).expect("刷新令牌生成失败");
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

fn hash_token(token: &str) -> String {
    let hash = ring::digest::digest(&ring::digest::SHA256, token.as_bytes());
    hash.as_ref().iter().map(|b| format!("{:02x}", b)).collect()
}

fn now_secs() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

fn create_refresh_token(conn: &Connection, user_id: i64) -> Result<(String, String), String> {
    let token = generate_refresh_token();
    let token_hash = hash_token(&token);
    // v3.7.1: 从 token_hash 前缀派生 family_id, 避免第二次 CSPRNG 调用
    let family_id = format!("fam_{}", &token_hash[..16]);
    let now = now_secs();
    conn.execute(
        "INSERT INTO refresh_tokens (token_hash, user_id, family_id, created_at, revoked) VALUES (?1, ?2, ?3, ?4, 0)",
        rusqlite::params![token_hash, user_id, family_id, now],
    )
    .map_err(|e| format!("刷新令牌存储失败: {}", e))?;
    Ok((token, family_id))
}

fn rotate_refresh_token(
    conn: &Connection,
    user_id: i64,
    old_token_hash: &str,
) -> Result<(String, String, String), String> {
    // 检查旧令牌是否存在且未被撤销
    let (family_id, revoked): (String, i32) = conn
        .query_row(
            "SELECT family_id, revoked FROM refresh_tokens WHERE token_hash = ?1 AND user_id = ?2",
            rusqlite::params![old_token_hash, user_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|e| {
            if matches!(e, rusqlite::Error::QueryReturnedNoRows) {
                "刷新令牌无效".to_string()
            } else {
                format!("令牌查询失败: {}", e)
            }
        })?;

    // 重放检测: 令牌已被撤销 → 撤销整个 family
    if revoked != 0 {
        let now = now_secs();
        let _ = conn.execute(
            "INSERT OR IGNORE INTO revoked_token_families (family_id, revoked_at) VALUES (?1, ?2)",
            rusqlite::params![family_id, now],
        );
        return Err("安全警告: 检测到令牌重放, 该设备的所有会话已失效, 请重新登录".to_string());
    }

    // P1-1: 事务保护 UPDATE + INSERT，防止崩溃导致用户被锁定
    conn.execute_batch("BEGIN IMMEDIATE;")
        .map_err(|e| format!("事务启动失败: {}", e))?;

    // 撤销旧令牌
    conn.execute(
        "UPDATE refresh_tokens SET revoked = 1 WHERE token_hash = ?1",
        rusqlite::params![old_token_hash],
    )
    .map_err(|e| {
        let _ = conn.execute_batch("ROLLBACK;");
        format!("令牌撤销失败: {}", e)
    })?;

    // 生成新令牌 (同一 family)
    let new_token = generate_refresh_token();
    let new_hash = hash_token(&new_token);
    let now = now_secs();
    conn.execute(
        "INSERT INTO refresh_tokens (token_hash, user_id, family_id, created_at, revoked) VALUES (?1, ?2, ?3, ?4, 0)",
        rusqlite::params![new_hash, user_id, family_id, now],
    )
    .map_err(|e| {
        let _ = conn.execute_batch("ROLLBACK;");
        format!("新令牌存储失败: {}", e)
    })?;

    conn.execute_batch("COMMIT;")
        .map_err(|e| format!("事务提交失败: {}", e))?;

    Ok((new_token, new_hash, family_id))
}

/// P1-7: 清理 30 天前的过期刷新令牌和已撤销令牌族
pub fn cleanup_expired_tokens(conn: &Connection) -> Result<(), String> {
    conn.execute_batch(
        "DELETE FROM refresh_tokens WHERE created_at < unixepoch() - 2592000;
         DELETE FROM revoked_token_families WHERE revoked_at < unixepoch() - 2592000;",
    )
    .map_err(|e| format!("清理过期令牌失败: {}", e))
}

/// 注册认证相关路由 (供 app_router 调用)
/// v2.2.1: 添加独立的登录速率限制
/// v2.3.0: 添加 JWT 刷新端点
pub(crate) fn register_auth_routes(router: Router<AppState>) -> Router<AppState> {
    let auth_router = Router::new()
        .route("/api/auth/register", post(register_handler))
        .route("/api/auth/login", post(login_handler))
        .route("/api/auth/refresh", post(refresh_handler))
        .layer(axum::middleware::from_fn(
            super::rate_limiter::auth_rate_limit_middleware,
        ));
    router.merge(auth_router)
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
            &EncodingKey::from_secret(jwt_secret_bytes().unwrap()),
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
        let first = jwt_secret_bytes().unwrap().to_vec();
        let second = jwt_secret_bytes().unwrap().to_vec();
        assert_eq!(first, second);
        assert!(!first.is_empty());
        // 随机生成的 secret 应为 32 字节
        assert_eq!(first.len(), 32);
    }

    // ── init_db ──

    #[test]
    fn test_init_db_creates_tables_and_default_user() {
        let dir = std::env::temp_dir().join(format!("quantpilot_auth_test_{}", std::process::id()));
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
            .query_row("SELECT id, username FROM users WHERE id = 0", [], |row| {
                Ok((row.get(0)?, row.get(1)?))
            })
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
