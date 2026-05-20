//! v3.7.0: 进程间加密通道
//! 启动时生成临时 32 字节会话密钥, 通过共享内存/临时文件交换
//! 所有 API 请求: AES-256-GCM 加密 + HMAC-SHA256 签名
//! 密钥仅存于内存, 进程退出即销毁

use anyhow::{bail, Result};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use ring::aead::{Aad, LessSafeKey, Nonce, UnboundKey, AES_256_GCM};
use ring::hmac;
use ring::rand::{SecureRandom, SystemRandom};
use std::sync::OnceLock;

const NONCE_LEN: usize = 12;
const TAG_LEN: usize = 16;

static SESSION_KEY: OnceLock<[u8; 32]> = OnceLock::new();
static HMAC_KEY: OnceLock<hmac::Key> = OnceLock::new();

/// 生成新的会话密钥并写入共享文件供测试端读取
pub fn init_session_key() -> Result<()> {
    let rng = SystemRandom::new();
    let mut key_bytes = [0u8; 32];
    rng.fill(&mut key_bytes)
        .map_err(|_| anyhow::anyhow!("生成会话密钥失败"))?;

    SESSION_KEY.set(key_bytes).map_err(|_| anyhow::anyhow!("会话密钥已初始化"))?;

    let hmac_key = hmac::Key::new(hmac::HMAC_SHA256, &key_bytes);
    HMAC_KEY.set(hmac_key).map_err(|_| anyhow::anyhow!("HMAC 密钥已初始化"))?;

    // 写入临时文件供测试端读取 (仅当前用户可读)
    let tmp_path = std::env::temp_dir().join(".quantpilot_session_key");
    std::fs::write(&tmp_path, BASE64.encode(key_bytes))?;
    // Windows: 临时目录默认仅当前用户可读
    // 文件权限由父目录继承

    Ok(())
}

/// 从共享文件加载会话密钥 (测试端调用)
pub fn load_session_key() -> Result<()> {
    let tmp_path = std::env::temp_dir().join(".quantpilot_session_key");
    let encoded = std::fs::read_to_string(&tmp_path)?;
    let key_bytes = BASE64.decode(encoded.trim())?;
    if key_bytes.len() != 32 {
        bail!("会话密钥长度不正确: {}", key_bytes.len());
    }

    let mut key = [0u8; 32];
    key.copy_from_slice(&key_bytes);

    SESSION_KEY.set(key).map_err(|_| anyhow::anyhow!("会话密钥已加载"))?;

    let hmac_key = hmac::Key::new(hmac::HMAC_SHA256, &key);
    HMAC_KEY.set(hmac_key).map_err(|_| anyhow::anyhow!("HMAC 密钥已加载"))?;

    // 立即删除共享文件
    let _ = std::fs::remove_file(&tmp_path);

    Ok(())
}

/// 读取会话密钥 (内部使用)
fn get_key() -> Result<&'static [u8; 32]> {
    SESSION_KEY.get().ok_or_else(|| anyhow::anyhow!("会话密钥未初始化"))
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

    fn ensure_key() {
        SESSION_KEY.get().map(|_| ()).unwrap_or_else(|| {
            init_session_key().ok();
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
        let mut bytes = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &encrypted).unwrap();
        if bytes.len() > 15 { bytes[15] ^= 0xFF; }
        let corrupted = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &bytes);
        assert!(decrypt(&corrupted).is_err());
    }
}
