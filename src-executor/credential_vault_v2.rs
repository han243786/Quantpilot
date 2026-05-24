/// v3.4.0: 执行端独立凭证保险库 v2
/// PBKDF2 1,000,000 轮, 独立密钥/凭证文件, 与测试端完全隔离
use anyhow::{bail, Result};
use ring::aead::{Aad, LessSafeKey, Nonce, UnboundKey, AES_256_GCM};
use ring::pbkdf2;
use ring::rand::{SecureRandom, SystemRandom};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use zeroize::Zeroizing;

const PBKDF2_ITERATIONS: u32 = 1_000_000;
const NONCE_LEN: usize = 12;
const SALT_LEN: usize = 32;
const CREDENTIALS_FILE: &str = ".executor-credentials";
const MACHINE_KEY_FILE: &str = ".executor-machine-key";

#[derive(Debug, Clone)]
pub struct CredentialEntry(pub BTreeMap<String, Zeroizing<String>>);

impl serde::Serialize for CredentialEntry {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let map: BTreeMap<&str, &str> = self
            .0
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();
        map.serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for CredentialEntry {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let map: BTreeMap<String, String> = BTreeMap::deserialize(deserializer)?;
        Ok(CredentialEntry(
            map.into_iter()
                .map(|(k, v)| (k, Zeroizing::new(v)))
                .collect(),
        ))
    }
}

/// v3.1.0 D-3: deny_unknown_fields 纵深防御
#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VaultData {
    pub entries: BTreeMap<String, CredentialEntry>,
}

pub struct ExecutorCredentialVault {
    data: Mutex<VaultData>,
    storage_dir: PathBuf,
    key: [u8; 32],
}

impl ExecutorCredentialVault {
    pub fn load(storage_dir: &Path) -> Result<Self> {
        std::fs::create_dir_all(storage_dir)?;
        let machine_key = get_or_create_machine_key(storage_dir)?;
        let vault_path = storage_dir.join(CREDENTIALS_FILE);
        let bak_path = vault_path.with_extension("bak");
        // v3.2.2: 崩溃恢复 — vault不存在但bak存在时从bak恢复
        let data = if vault_path.exists() {
            decrypt_vault(&std::fs::read(&vault_path)?, &machine_key)?
        } else if bak_path.exists() {
            decrypt_vault(&std::fs::read(&bak_path)?, &machine_key)?
        } else {
            VaultData::default()
        };
        Ok(Self {
            data: Mutex::new(data),
            storage_dir: storage_dir.to_path_buf(),
            key: machine_key,
        })
    }

    pub fn set_service(&self, service: &str, fields: BTreeMap<String, String>) -> Result<()> {
        {
            let mut data = self.data.lock().map_err(|e| anyhow::anyhow!("锁: {}", e))?;
            data.entries.insert(
                service.to_string(),
                CredentialEntry(
                    fields
                        .into_iter()
                        .map(|(k, v)| (k, Zeroizing::new(v)))
                        .collect(),
                ),
            );
        }
        self.save()
    }

    pub fn get_service(&self, service: &str) -> Result<BTreeMap<String, String>> {
        let data = self.data.lock().map_err(|e| anyhow::anyhow!("锁: {}", e))?;
        data.entries
            .get(service)
            .map(|e| {
                e.0.iter()
                    .map(|(k, v)| (k.clone(), v.to_string()))
                    .collect()
            })
            .ok_or_else(|| anyhow::anyhow!("凭证 {} 不存在", service))
    }

    pub fn list_services(&self) -> Result<Vec<String>> {
        let data = self.data.lock().map_err(|e| anyhow::anyhow!("锁: {}", e))?;
        Ok(data.entries.keys().cloned().collect())
    }

    pub fn delete_service(&self, service: &str) -> Result<()> {
        {
            let mut data = self.data.lock().map_err(|e| anyhow::anyhow!("锁: {}", e))?;
            data.entries.remove(service);
        }
        self.save()
    }

    fn save(&self) -> Result<()> {
        let data = self.data.lock().map_err(|e| anyhow::anyhow!("锁: {}", e))?;
        let encrypted = encrypt_vault(&serde_json::to_vec(&*data)?, &self.key)?;
        let vault_path = self.storage_dir.join(CREDENTIALS_FILE);
        let bak = vault_path.with_extension("bak");
        let tmp = vault_path.with_extension("tmp");
        // v3.0.1 D-1: .bak 备份+回滚 — 先建bak再写tmp, 失败从bak恢复
        if vault_path.exists() {
            let _ = std::fs::remove_file(&bak);
            std::fs::rename(&vault_path, &bak)?;
        }
        let result = (|| -> std::io::Result<()> {
            {
                use std::io::Write;
                let mut file = std::fs::OpenOptions::new()
                    .create(true)
                    .write(true)
                    .truncate(true)
                    .open(&tmp)?;
                file.write_all(&encrypted)?;
                file.sync_all()?;
            }
            std::fs::rename(&tmp, &vault_path)?;
            sync_parent_directory(&vault_path)?;
            Ok(())
        })();
        match result {
            Ok(()) => {
                let _ = std::fs::remove_file(&bak);
                Ok(())
            }
            Err(e) => {
                // 失败时从 .bak 恢复
                if bak.exists() {
                    let _ = std::fs::rename(&bak, &vault_path);
                }
                Err(anyhow::anyhow!("凭证写入失败: {}", e))
            }
        }
    }
}

fn get_or_create_machine_key(storage_dir: &Path) -> Result<[u8; 32]> {
    let key_path = storage_dir.join(MACHINE_KEY_FILE);
    if key_path.exists() {
        let encoded = std::fs::read_to_string(&key_path)?;
        let bytes =
            base64::Engine::decode(&base64::engine::general_purpose::STANDARD, encoded.trim())?;
        if bytes.len() != 32 {
            bail!("机器密钥长度异常");
        }
        let mut key = [0u8; 32];
        key.copy_from_slice(&bytes);
        Ok(key)
    } else {
        let rng = SystemRandom::new();
        let mut key = [0u8; 32];
        rng.fill(&mut key)
            .map_err(|_| anyhow::anyhow!("生成密钥失败"))?;
        let encoded = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &key);
        quantpilot::storage_lifecycle::atomic_write_secret_file(&key_path, encoded.as_bytes())?;
        Ok(key)
    }
}

fn derive_aes_key(machine_key: &[u8; 32], salt: &[u8]) -> [u8; 32] {
    let mut aes_key = [0u8; 32];
    pbkdf2::derive(
        pbkdf2::PBKDF2_HMAC_SHA256,
        std::num::NonZeroU32::new(PBKDF2_ITERATIONS).unwrap(),
        salt,
        machine_key,
        &mut aes_key,
    );
    aes_key
}

fn encrypt_vault(plaintext: &[u8], machine_key: &[u8; 32]) -> Result<Vec<u8>> {
    let rng = SystemRandom::new();
    let mut salt = [0u8; SALT_LEN];
    rng.fill(&mut salt)
        .map_err(|_| anyhow::anyhow!("盐值生成失败"))?; // v3.0.2 A-1
    let aes_key = derive_aes_key(machine_key, &salt);
    let unbound =
        UnboundKey::new(&AES_256_GCM, &aes_key).map_err(|_| anyhow::anyhow!("密钥创建失败"))?; // v3.0.2 A-3
    let key = LessSafeKey::new(unbound);
    let mut nonce_bytes = [0u8; NONCE_LEN];
    rng.fill(&mut nonce_bytes)
        .map_err(|_| anyhow::anyhow!("临时数生成失败"))?; // v3.0.2 A-2
    let nonce = Nonce::assume_unique_for_key(nonce_bytes);
    let mut in_out = plaintext.to_vec();
    key.seal_in_place_append_tag(nonce, Aad::empty(), &mut in_out)
        .map_err(|_| anyhow::anyhow!("加密失败"))?;
    // v3.3.0: 预分配避免多次扩容
    let mut result = Vec::with_capacity(SALT_LEN + NONCE_LEN + in_out.len());
    result.extend_from_slice(&salt);
    result.extend_from_slice(&nonce_bytes);
    result.extend_from_slice(&in_out);
    Ok(result)
}

fn decrypt_vault(encrypted: &[u8], machine_key: &[u8; 32]) -> Result<VaultData> {
    if encrypted.len() < SALT_LEN + NONCE_LEN + 16 {
        bail!("密文过短");
    }
    let (salt, rest) = encrypted.split_at(SALT_LEN);
    let (nonce_bytes, ciphertext) = rest.split_at(NONCE_LEN);
    let aes_key = derive_aes_key(machine_key, salt);
    let unbound =
        UnboundKey::new(&AES_256_GCM, &aes_key).map_err(|_| anyhow::anyhow!("解密密钥创建失败"))?;
    let key = LessSafeKey::new(unbound);
    let nonce = Nonce::assume_unique_for_key(nonce_bytes.try_into().unwrap());
    let mut in_out = ciphertext.to_vec();
    let plaintext = key
        .open_in_place(nonce, Aad::empty(), &mut in_out)
        .map_err(|_| anyhow::anyhow!("解密失败: 密钥不匹配或数据损坏"))?;
    Ok(serde_json::from_slice(plaintext)?)
}

#[cfg(unix)]
fn sync_parent_directory(path: &Path) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::File::open(parent)?.sync_all()?;
    }
    Ok(())
}

#[cfg(not(unix))]
fn sync_parent_directory(_path: &Path) -> std::io::Result<()> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "quantpilot-executor-vault-{name}-{}",
            std::process::id()
        ))
    }

    #[test]
    fn credential_vault_roundtrips_and_deletes_service() {
        let dir = temp_dir("roundtrip");
        let _ = std::fs::remove_dir_all(&dir);
        let vault = ExecutorCredentialVault::load(&dir).unwrap();
        vault
            .set_service(
                "okx",
                BTreeMap::from([
                    ("api_key".to_string(), "key".to_string()),
                    ("secret".to_string(), "secret".to_string()),
                    ("passphrase".to_string(), "pass".to_string()),
                ]),
            )
            .unwrap();

        assert_eq!(vault.list_services().unwrap(), vec!["okx".to_string()]);
        let service = vault.get_service("okx").unwrap();
        assert_eq!(service.get("api_key").map(String::as_str), Some("key"));

        vault.delete_service("okx").unwrap();
        assert!(vault.list_services().unwrap().is_empty());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn vault_encrypt_decrypt_rejects_truncated_ciphertext() {
        let key = [7u8; 32];
        let plaintext = serde_json::to_vec(&VaultData::default()).unwrap();
        let encrypted = encrypt_vault(&plaintext, &key).unwrap();
        let decrypted = decrypt_vault(&encrypted, &key).unwrap();
        assert!(decrypted.entries.is_empty());
        assert!(decrypt_vault(&encrypted[..8], &key).is_err());
    }
}
