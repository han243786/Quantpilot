use anyhow::Result;
use ring::aead::{UnboundKey, AES_256_GCM};
use ring::rand::{SecureRandom, SystemRandom};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

static MACHINE_KEYS: OnceLock<Mutex<BTreeMap<PathBuf, [u8; 32]>>> = OnceLock::new();
static MACHINE_KEY_INIT_LOCK: Mutex<()> = Mutex::new(());

fn absolute_path(path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(path)
    }
}

pub(super) fn get_machine_key_for_path(path: &Path) -> Result<[u8; 32]> {
    let path = absolute_path(path);
    let keys = MACHINE_KEYS.get_or_init(|| Mutex::new(BTreeMap::new()));
    if let Some(key) = keys
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .get(&path)
        .copied()
    {
        return Ok(key);
    }
    let _guard = MACHINE_KEY_INIT_LOCK
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    if let Some(key) = keys
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .get(&path)
        .copied()
    {
        return Ok(key);
    }
    let key: [u8; 32] = if path.exists() {
        std::fs::read(&path)?
            .try_into()
            .map_err(|_| anyhow::anyhow!("机器密钥格式错误"))?
    } else {
        let rng = SystemRandom::new();
        let mut k = [0u8; 32];
        rng.fill(&mut k)
            .map_err(|_| anyhow::anyhow!("随机数生成失败"))?;
        crate::storage_lifecycle::atomic_write_secret_file(&path, &k)
            .map_err(|e| anyhow::anyhow!("无法保存机器密钥: {}", e))?;
        k
    };
    keys.lock()
        .unwrap_or_else(|error| error.into_inner())
        .insert(path, key);
    Ok(key)
}

pub(super) fn derive_key_from_machine_key(machine_key: &[u8; 32]) -> Result<UnboundKey> {
    let host = hostname::get().unwrap_or_default();
    let hex: String = machine_key.iter().map(|b| format!("{:02x}", b)).collect();
    let seed = format!(
        "quantpilot-credential-vault-{}-{}",
        host.to_string_lossy(),
        hex
    );
    let hash = ring::digest::digest(&ring::digest::SHA256, seed.as_bytes());
    let key_bytes: [u8; 32] = hash.as_ref()[..32].try_into().unwrap();
    UnboundKey::new(&AES_256_GCM, &key_bytes).map_err(|_| anyhow::anyhow!("密钥派生失败"))
}

pub(super) fn derive_key_pbkdf2_from_machine_key(machine_key: &[u8; 32]) -> Result<UnboundKey> {
    let host = hostname::get().unwrap_or_default();
    let salt = format!("quantpilot-vault-v2-{}", host.to_string_lossy());
    let mut key_bytes = [0u8; 32];
    ring::pbkdf2::derive(
        ring::pbkdf2::PBKDF2_HMAC_SHA256,
        std::num::NonZeroU32::new(600_000).unwrap(),
        salt.as_bytes(),
        machine_key,
        &mut key_bytes,
    );
    UnboundKey::new(&ring::aead::AES_256_GCM, &key_bytes)
        .map_err(|_| anyhow::anyhow!("PBKDF2密钥派生失败"))
}
