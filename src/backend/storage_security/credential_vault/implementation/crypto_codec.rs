use super::machine_key_management::{
    derive_key_from_machine_key, derive_key_pbkdf2_from_machine_key,
};
use anyhow::Result;
use ring::aead::{Aad, LessSafeKey, Nonce, UnboundKey};
use ring::rand::{SecureRandom, SystemRandom};
use zeroize::Zeroizing;

const NONCE_LEN: usize = 12;
const TAG_LEN: usize = 16;

pub(super) fn encrypt_with_machine_key(plaintext: &str, machine_key: &[u8; 32]) -> Result<Vec<u8>> {
    let key = derive_key_pbkdf2_from_machine_key(machine_key)?;
    let key = LessSafeKey::new(key);
    let rng = SystemRandom::new();
    let mut nonce_bytes = [0u8; NONCE_LEN];
    rng.fill(&mut nonce_bytes)
        .map_err(|_| anyhow::anyhow!("随机数生成失败"))?;
    let nonce = Nonce::assume_unique_for_key(nonce_bytes);
    let mut data = plaintext.as_bytes().to_vec();
    key.seal_in_place_append_tag(nonce, Aad::from(".credentials".as_bytes()), &mut data)
        .map_err(|_| anyhow::anyhow!("加密失败"))?;
    let mut result = vec![2u8];
    result.extend(nonce_bytes);
    result.extend(data);
    Ok(result)
}

pub(super) fn decrypt_with_machine_key(
    ciphertext: &[u8],
    machine_key: &[u8; 32],
) -> Result<Zeroizing<String>> {
    if ciphertext.is_empty() {
        anyhow::bail!("凭证数据为空");
    }

    let version = ciphertext[0];
    let (key, offset): (UnboundKey, usize) = match version {
        2 => (derive_key_pbkdf2_from_machine_key(machine_key)?, 1),
        1 => (derive_key_from_machine_key(machine_key)?, 1),
        _ => (derive_key_from_machine_key(machine_key)?, 0),
    };

    let payload = &ciphertext[offset..];
    if payload.len() < NONCE_LEN + TAG_LEN {
        anyhow::bail!("凭证数据损坏");
    }

    let key = LessSafeKey::new(key);
    let nonce_bytes: [u8; NONCE_LEN] = payload[..NONCE_LEN].try_into().unwrap();
    let nonce = Nonce::assume_unique_for_key(nonce_bytes);
    let mut data = payload[NONCE_LEN..].to_vec();
    let plaintext = key
        .open_in_place(nonce, Aad::from(".credentials".as_bytes()), &mut data)
        .map_err(|_| anyhow::anyhow!("凭证解密失败: 密钥不匹配或数据损坏"))?;
    let plaintext_len = plaintext.len();
    data.truncate(plaintext_len);
    Ok(Zeroizing::new(String::from_utf8(data).unwrap_or_default()))
}
