use anyhow::{Context, Result};
use ring::aead::{Aad, LessSafeKey, Nonce, UnboundKey, AES_256_GCM};
use ring::rand::{SecureRandom, SystemRandom};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Mutex;
use zeroize::Zeroizing;

const CREDENTIALS_FILE: &str = "storage/.credentials";
const NONCE_LEN: usize = 12;
const TAG_LEN: usize = 16;

const MACHINE_KEY_FILE: &str = "storage/.machine_key";

fn get_or_create_machine_key() -> Result<[u8; 32]> {
    let path = std::path::Path::new(MACHINE_KEY_FILE);
    if path.exists() {
        let bytes = std::fs::read(path)?;
        bytes.try_into().map_err(|_| anyhow::anyhow!("机器密钥格式错误"))
    } else {
        let rng = SystemRandom::new();
        let mut key = [0u8; 32];
        rng.fill(&mut key).map_err(|_| anyhow::anyhow!("随机数生成失败"))?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, key)?;
        Ok(key)
    }
}

fn derive_key() -> Result<UnboundKey> {
    let host = hostname::get().unwrap_or_default();
    let machine_key = get_or_create_machine_key().unwrap_or_default();
    let machine_key_hex: String = machine_key.iter().map(|b| format!("{:02x}", b)).collect();
    let seed = format!(
        "quantpilot-credential-vault-{}-{}",
        host.to_string_lossy(),
        machine_key_hex
    );
    let hash = ring::digest::digest(&ring::digest::SHA256, seed.as_bytes());
    let key_bytes: [u8; 32] = hash.as_ref()[..32].try_into().unwrap();
    UnboundKey::new(&AES_256_GCM, &key_bytes)
        .map_err(|_| anyhow::anyhow!("密钥派生失败"))
}

fn encrypt(plaintext: &str) -> Result<Vec<u8>> {
    let key = derive_key()?;
    let key = LessSafeKey::new(key);
    let rng = SystemRandom::new();
    let mut nonce_bytes = [0u8; NONCE_LEN];
    rng.fill(&mut nonce_bytes)
        .map_err(|_| anyhow::anyhow!("随机数生成失败"))?;
    let nonce = Nonce::assume_unique_for_key(nonce_bytes);
    let mut data = plaintext.as_bytes().to_vec();
    key.seal_in_place_append_tag(nonce, Aad::empty(), &mut data)
        .map_err(|_| anyhow::anyhow!("加密失败"))?;
    let mut result = nonce_bytes.to_vec();
    result.extend(data);
    Ok(result)
}

fn decrypt(ciphertext: &[u8]) -> Result<Zeroizing<String>> {
    if ciphertext.len() < NONCE_LEN + TAG_LEN {
        anyhow::bail!("凭证数据损坏");
    }
    let key = derive_key()?;
    let key = LessSafeKey::new(key);
    let nonce_bytes: [u8; NONCE_LEN] = ciphertext[..NONCE_LEN].try_into().unwrap();
    let nonce = Nonce::assume_unique_for_key(nonce_bytes);
    let mut data = ciphertext[NONCE_LEN..].to_vec();
    let plaintext = key
        .open_in_place(nonce, Aad::empty(), &mut data)
        .map_err(|_| anyhow::anyhow!("凭证解密失败: 密钥不匹配或数据损坏"))?;
    let plaintext_len = plaintext.len();
    data.truncate(plaintext_len);
    Ok(Zeroizing::new(String::from_utf8(data).unwrap_or_default()))
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct VaultData {
    entries: BTreeMap<String, BTreeMap<String, String>>,
}

pub type CredentialFields = BTreeMap<String, String>;

pub struct CredentialVault {
    path: PathBuf,
    data: Mutex<VaultData>,
}

impl CredentialVault {
    pub fn load() -> Result<Self> {
        let path = PathBuf::from(CREDENTIALS_FILE);
        let data = if path.exists() {
            let encrypted = std::fs::read(&path)?;
            let decrypted = decrypt(&encrypted)
                .map_err(|_| anyhow::anyhow!("凭证文件损坏或密钥不匹配, 请重新设置凭证"))?;
            serde_json::from_str(&decrypted).context("凭证数据格式错误")?
        } else {
            VaultData::default()
        };
        Ok(Self { path, data: Mutex::new(data) })
    }

    /// 按标签整体设置凭证字段，同一标签下的所有字段原子替换
    pub fn set_service(&self, service: &str, fields: CredentialFields) -> Result<()> {
        if fields.is_empty() {
            anyhow::bail!("凭证字段不能为空");
        }
        let mut data = self.data.lock().unwrap();
        data.entries.insert(service.to_string(), fields);
        self.save_inner(&data)
    }

    /// 获取标签下的全部凭证字段
    pub fn get_service(&self, service: &str) -> Option<CredentialFields> {
        let data = self.data.lock().unwrap();
        data.entries.get(service).cloned()
    }

    pub fn delete_service(&self, service: &str) -> Result<()> {
        let mut data = self.data.lock().unwrap();
        if data.entries.remove(service).is_none() {
            anyhow::bail!("服务 '{}' 不存在", service);
        }
        self.save_inner(&data)
    }

    pub fn list_services(&self) -> Vec<String> {
        let data = self.data.lock().unwrap();
        data.entries.keys().cloned().collect()
    }

    fn save_inner(&self, data: &VaultData) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string(data)?;
        let encrypted = encrypt(&json)?;
        std::fs::write(&self.path, encrypted)?;
        Ok(())
    }
}
