//! v3.7.0: 进程间加密通道
//! 启动时生成临时 32 字节会话密钥, 通过共享内存/临时文件交换
//! 所有 API 请求: AES-256-GCM 加密 + HMAC-SHA256 签名
//! 密钥仅存于内存, 进程退出即销毁

use anyhow::{bail, Result};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use ring::aead::{Aad, LessSafeKey, Nonce, UnboundKey, AES_256_GCM};
use ring::hmac;
use ring::rand::{SecureRandom, SystemRandom};
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

const NONCE_LEN: usize = 12;
const TAG_LEN: usize = 16;

static SESSION_KEY: OnceLock<[u8; 32]> = OnceLock::new();
static HMAC_KEY: OnceLock<hmac::Key> = OnceLock::new();
static SESSION_INSTALL_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

fn session_key_path() -> PathBuf {
    if let Some(path) = std::env::var_os("QUANTPILOT_SESSION_KEY_PATH") {
        return PathBuf::from(path);
    }
    std::env::var_os("QUANTPILOT_STORAGE_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("storage"))
        .join(".executor-session-key")
}

fn write_session_key_file(key_bytes: &[u8; 32]) -> Result<()> {
    let path = session_key_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let tmp_path = path.with_extension(format!("tmp-{}", std::process::id()));
    std::fs::write(&tmp_path, BASE64.encode(key_bytes))?;
    if let Err(rename_err) = std::fs::rename(&tmp_path, &path) {
        std::fs::copy(&tmp_path, &path).map_err(|copy_err| {
            anyhow::anyhow!("写入会话密钥失败: rename={}, copy={}", rename_err, copy_err)
        })?;
        let _ = std::fs::remove_file(&tmp_path);
    }
    Ok(())
}

fn install_session_key(key_bytes: [u8; 32], source: &str) -> Result<()> {
    if let Some(existing) = SESSION_KEY.get() {
        if existing == &key_bytes {
            return Ok(());
        }
        bail!("会话密钥已初始化且与 {} 不一致", source);
    }

    SESSION_KEY
        .set(key_bytes)
        .map_err(|_| anyhow::anyhow!("会话密钥已初始化"))?;

    let hmac_key = hmac::Key::new(hmac::HMAC_SHA256, &key_bytes);
    HMAC_KEY
        .set(hmac_key)
        .map_err(|_| anyhow::anyhow!("HMAC 密钥已初始化"))?;

    Ok(())
}

/// 生成新的会话密钥并写入共享文件供测试端读取
pub fn init_session_key() -> Result<()> {
    let _guard = SESSION_INSTALL_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    if let Some(existing) = SESSION_KEY.get() {
        write_session_key_file(existing)?;
        return Ok(());
    }

    let rng = SystemRandom::new();
    let mut key_bytes = [0u8; 32];
    rng.fill(&mut key_bytes)
        .map_err(|_| anyhow::anyhow!("生成会话密钥失败"))?;

    install_session_key(key_bytes, "新生成密钥")?;

    // 写入当前工作区 storage/ 下的会话文件，避免跨项目共享全局临时文件。
    write_session_key_file(&key_bytes)?;

    Ok(())
}

/// 从共享文件加载会话密钥 (测试端调用)
pub fn load_session_key() -> Result<()> {
    let _guard = SESSION_INSTALL_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    let key_path = session_key_path();
    let encoded = std::fs::read_to_string(&key_path)?;
    let key_bytes = BASE64.decode(encoded.trim())?;
    if key_bytes.len() != 32 {
        bail!("会话密钥长度不正确: {}", key_bytes.len());
    }

    let mut key = [0u8; 32];
    key.copy_from_slice(&key_bytes);

    install_session_key(key, "共享文件")?;

    // 立即删除共享文件
    let _ = std::fs::remove_file(&key_path);

    Ok(())
}

/// 读取会话密钥 (内部使用)
fn get_key() -> Result<&'static [u8; 32]> {
    SESSION_KEY
        .get()
        .ok_or_else(|| anyhow::anyhow!("会话密钥未初始化"))
}

/// AES-256-GCM 加密 plaintext → base64(ciphertext + tag)
pub fn encrypt(plaintext: &[u8]) -> Result<String> {
    let key_bytes = get_key()?;
    let unbound_key = UnboundKey::new(&AES_256_GCM, key_bytes)
        .map_err(|_| anyhow::anyhow!("创建 AES 密钥失败"))?;
    let key = LessSafeKey::new(unbound_key);

    let rng = SystemRandom::new();
    let mut nonce_bytes = [0u8; NONCE_LEN];
    rng.fill(&mut nonce_bytes)
        .map_err(|_| anyhow::anyhow!("生成临时数失败"))?;
    let nonce = Nonce::assume_unique_for_key(nonce_bytes);

    let mut in_out = plaintext.to_vec();
    key.seal_in_place_append_tag(nonce, Aad::empty(), &mut in_out)
        .map_err(|_| anyhow::anyhow!("AES 加密失败"))?;

    // 前置 nonce: [nonce(12) | ciphertext | tag(16)]
    let mut result = nonce_bytes.to_vec();
    result.extend_from_slice(&in_out);
    Ok(BASE64.encode(&result))
}

/// AES-256-GCM 解密 base64(nonce + ciphertext + tag) → plaintext
pub fn decrypt(encoded: &str) -> Result<Vec<u8>> {
    let key_bytes = get_key()?;
    let unbound_key = UnboundKey::new(&AES_256_GCM, key_bytes)
        .map_err(|_| anyhow::anyhow!("创建 AES 密钥失败"))?;
    let key = LessSafeKey::new(unbound_key);

    let data = BASE64.decode(encoded.trim())?;
    if data.len() < NONCE_LEN + TAG_LEN {
        bail!("密文长度不足");
    }

    let (nonce_bytes, ciphertext) = data.split_at(NONCE_LEN);
    let nonce = Nonce::assume_unique_for_key(nonce_bytes.try_into().unwrap());

    let mut in_out = ciphertext.to_vec();
    let plaintext = key
        .open_in_place(nonce, Aad::empty(), &mut in_out)
        .map_err(|_| anyhow::anyhow!("AES 解密失败: 密钥不匹配或数据损坏"))?;

    Ok(plaintext.to_vec())
}

/// HMAC-SHA256 签名 body
pub fn sign(body: &[u8]) -> Result<String> {
    let hmac_key = HMAC_KEY
        .get()
        .ok_or_else(|| anyhow::anyhow!("HMAC 密钥未初始化"))?;
    let tag = hmac::sign(hmac_key, body);
    Ok(BASE64.encode(tag.as_ref()))
}

fn request_signing_payload(
    method: &str,
    path_and_query: &str,
    timestamp_ms: u64,
    body: &[u8],
) -> Vec<u8> {
    let mut payload = Vec::with_capacity(method.len() + path_and_query.len() + body.len() + 48);
    payload.extend_from_slice(method.to_ascii_uppercase().as_bytes());
    payload.push(b'\n');
    payload.extend_from_slice(path_and_query.as_bytes());
    payload.push(b'\n');
    payload.extend_from_slice(timestamp_ms.to_string().as_bytes());
    payload.push(b'\n');
    payload.extend_from_slice(body);
    payload
}

/// HMAC-SHA256 签名 method/path/timestamp/body，避免 body-only 签名被跨端点重放。
pub fn sign_request(
    method: &str,
    path_and_query: &str,
    timestamp_ms: u64,
    body: &[u8],
) -> Result<String> {
    sign(&request_signing_payload(
        method,
        path_and_query,
        timestamp_ms,
        body,
    ))
}

/// 验证 method/path/timestamp/body 签名。
pub fn verify_request(
    method: &str,
    path_and_query: &str,
    timestamp_ms: u64,
    body: &[u8],
    signature_b64: &str,
) -> Result<()> {
    let payload = request_signing_payload(method, path_and_query, timestamp_ms, body);
    verify(&payload, signature_b64)
}

/// HMAC-SHA256 验证签名
pub fn verify(body: &[u8], signature_b64: &str) -> Result<()> {
    let expected = sign(body)?;
    if expected.trim() != signature_b64.trim() {
        bail!("HMAC 签名验证失败");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    static TEST_SESSION_KEY_READY: std::sync::OnceLock<()> = std::sync::OnceLock::new();

    fn ensure_key() {
        TEST_SESSION_KEY_READY.get_or_init(|| {
            let key_path = std::env::temp_dir()
                .join(format!("quantpilot-session-test-{}", std::process::id()))
                .join(".executor-session-key");
            std::env::set_var("QUANTPILOT_SESSION_KEY_PATH", &key_path);
            init_session_key().ok();
            let _ = std::fs::remove_file(&key_path);
            if let Some(parent) = key_path.parent() {
                let _ = std::fs::remove_dir_all(parent);
            }
        });
    }

    #[test]
    fn encrypt_decrypt_roundtrip() {
        ensure_key();
        let plaintext = b"{\"strategy_id\":\"test-001\"}";
        let encrypted = encrypt(plaintext).unwrap();
        let decrypted = decrypt(&encrypted).unwrap();
        assert_eq!(plaintext, decrypted.as_slice());
    }

    #[test]
    fn hmac_sign_verify_roundtrip() {
        ensure_key();
        let body = b"POST /api/executor/strategies";
        let sig = sign(body).unwrap();
        assert!(verify(body, &sig).is_ok());
    }

    #[test]
    fn request_signature_binds_route_and_timestamp() {
        ensure_key();
        let body = br#"{"strategy_id":"exec-1"}"#;
        let sig =
            sign_request("POST", "/api/executor/strategies", 1_730_000_000_000, body).unwrap();
        assert!(verify_request(
            "POST",
            "/api/executor/strategies",
            1_730_000_000_000,
            body,
            &sig
        )
        .is_ok());
        assert!(
            verify_request("POST", "/api/executor/mode", 1_730_000_000_000, body, &sig).is_err()
        );
        assert!(verify_request(
            "POST",
            "/api/executor/strategies",
            1_730_000_000_001,
            body,
            &sig
        )
        .is_err());
    }

    #[test]
    fn wrong_signature_fails() {
        ensure_key();
        let body = b"test";
        let sig = sign(b"different").unwrap();
        assert!(verify(body, &sig).is_err());
    }

    #[test]
    fn encrypt_decrypt_empty_payload() {
        ensure_key();
        let encrypted = encrypt(b"").unwrap();
        assert_eq!(decrypt(&encrypted).unwrap(), b"");
    }

    #[test]
    fn decrypt_corrupted_ciphertext_fails() {
        ensure_key();
        let encrypted = encrypt(b"test data").unwrap();
        let mut bytes =
            base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &encrypted).unwrap();
        if bytes.len() > 15 {
            bytes[15] ^= 0xFF;
        }
        let corrupted = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &bytes);
        assert!(decrypt(&corrupted).is_err());
    }
}
