use anyhow::Result;
use ring::aead::{Aad, LessSafeKey, Nonce, UnboundKey, AES_256_GCM};
use ring::rand::{SecureRandom, SystemRandom};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use zeroize::{Zeroize, Zeroizing};

const CREDENTIALS_FILE: &str = "storage/.credentials";
const NONCE_LEN: usize = 12;
const TAG_LEN: usize = 16;
const MACHINE_KEY_FILE: &str = "storage/.machine_key";

// ── SecretString: Drop 时自动 Zeroize ──────────────────────

#[derive(Debug, Clone)]
struct SecretString(String);

impl Serialize for SecretString {
    fn serialize<S: serde::Serializer>(&self, s: S) -> std::result::Result<S::Ok, S::Error> {
        self.0.serialize(s)
    }
}

impl<'de> Deserialize<'de> for SecretString {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> std::result::Result<Self, D::Error> {
        String::deserialize(d).map(SecretString)
    }
}

impl Drop for SecretString {
    fn drop(&mut self) {
        self.0.zeroize();
    }
}

// ── 机器密钥: OnceLock 保护，消除 TOCTOU 竞态 ──────────────

static MACHINE_KEY: OnceLock<[u8; 32]> = OnceLock::new();

fn get_machine_key() -> Result<&'static [u8; 32]> {
    if let Some(key) = MACHINE_KEY.get() {
        return Ok(key);
    }
    let path = Path::new(MACHINE_KEY_FILE);
    let key: [u8; 32] = if path.exists() {
        std::fs::read(path)?
            .try_into()
            .map_err(|_| anyhow::anyhow!("机器密钥格式错误"))?
    } else {
        let rng = SystemRandom::new();
        let mut k = [0u8; 32];
        rng.fill(&mut k)
            .map_err(|_| anyhow::anyhow!("随机数生成失败"))?;
        k
    };
    // 只有一个线程能 set 成功：它负责持久化，其他线程复用它的 key
    match MACHINE_KEY.set(key) {
        Ok(()) => {
            if !path.exists() {
                if let Some(parent) = path.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                let tmp = path.with_extension("tmp");
                std::fs::write(&tmp, key)
                    .map_err(|e| anyhow::anyhow!("无法写入机器密钥: {}", e))?;
                std::fs::rename(&tmp, path)
                    .map_err(|e| anyhow::anyhow!("无法保存机器密钥: {}", e))?;
            }
            Ok(MACHINE_KEY.get().unwrap())
        }
        Err(_) => Ok(MACHINE_KEY.get().unwrap()),
    }
}

// ── 密钥派生 ──────────────────────────────────────────────

fn derive_key() -> Result<UnboundKey> {
    let host = hostname::get().unwrap_or_default();
    let machine_key = get_machine_key()?;
    let hex: String = machine_key.iter().map(|b| format!("{:02x}", b)).collect();
    let seed = format!(
        "quantpilot-credential-vault-{}-{}",
        host.to_string_lossy(),
        hex
    );
    let hash = ring::digest::digest(&ring::digest::SHA256, seed.as_bytes());
    let key_bytes: [u8; 32] = hash.as_ref()[..32].try_into().unwrap();
    UnboundKey::new(&AES_256_GCM, &key_bytes)
        .map_err(|_| anyhow::anyhow!("密钥派生失败"))
}

// ── 加解密 ────────────────────────────────────────────────

fn encrypt(plaintext: &str) -> Result<Vec<u8>> {
    let key = derive_key()?;
    let key = LessSafeKey::new(key);
    let rng = SystemRandom::new();
    let mut nonce_bytes = [0u8; NONCE_LEN];
    rng.fill(&mut nonce_bytes)
        .map_err(|_| anyhow::anyhow!("随机数生成失败"))?;
    let nonce = Nonce::assume_unique_for_key(nonce_bytes);
    let mut data = plaintext.as_bytes().to_vec();
    key.seal_in_place_append_tag(nonce, Aad::from(CREDENTIALS_FILE.as_bytes()), &mut data)
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
        .open_in_place(nonce, Aad::from(CREDENTIALS_FILE.as_bytes()), &mut data)
        .map_err(|_| anyhow::anyhow!("凭证解密失败: 密钥不匹配或数据损坏"))?;
    let plaintext_len = plaintext.len();
    data.truncate(plaintext_len);
    Ok(Zeroizing::new(String::from_utf8(data).unwrap_or_default()))
}

// ── Vault 类型 ────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct VaultData {
    entries: BTreeMap<String, BTreeMap<String, SecretString>>,
}

pub type CredentialFields = BTreeMap<String, String>;

pub struct CredentialVault {
    path: PathBuf,
    data: Mutex<VaultData>,
}

// ── Vault API ─────────────────────────────────────────────

impl CredentialVault {
    pub fn load() -> Result<Self> {
        // 触发机器密钥初始化（可能失败，不再静默降级）
        get_machine_key()?;

        let path = PathBuf::from(CREDENTIALS_FILE);
        let data = if path.exists() {
            let encrypted = std::fs::read(&path)?;
            let decrypted = decrypt(&encrypted)
                .map_err(|_| anyhow::anyhow!("凭证文件损坏或密钥不匹配, 请重新设置凭证"))?;
            serde_json::from_str(&decrypted).unwrap_or_default()
        } else {
            // 首次启动: 创建空 vault 并持久化, 避免 API 返回 503
            let data = VaultData::default();
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let json = serde_json::to_string(&data)?;
            let encrypted = encrypt(&json)?;
            std::fs::write(&path, encrypted)?;
            data
        };
        Ok(Self {
            path,
            data: Mutex::new(data),
        })
    }

    /// 按标签整体设置凭证字段，同一标签下的所有字段原子替换
    pub fn set_service(&self, service: &str, fields: CredentialFields) -> Result<()> {
        if fields.is_empty() {
            anyhow::bail!("凭证字段不能为空");
        }
        let mut data = self.data.lock().unwrap();
        let entry: BTreeMap<String, SecretString> = fields
            .into_iter()
            .map(|(k, v)| (k, SecretString(v)))
            .collect();
        data.entries.insert(service.to_string(), entry);
        self.save_inner(&data)
    }

    /// 获取标签下的全部凭证字段。
    ///
    /// # 安全
    ///
    /// 返回值为 `BTreeMap<String, String>`，其中 value 是从锁内 `SecretString`
    /// clone 出来的明文副本。锁内原始 `SecretString` 在 `Mutex::unlock` 时已
    /// 被 Drop 清零，但调用方持有的 clone 副本需要自行管理生命周期：
    ///
    /// - 尽快用完，用完后让变量离开作用域
    /// - 若需长期持有，用 `Zeroizing::new(value)` 包裹
    /// - 参考 `test_runner.rs:load_exchange_credentials()` 的调用模式
    pub fn get_service(&self, service: &str) -> Option<CredentialFields> {
        let data = self.data.lock().unwrap();
        data.entries.get(service).map(|entry| {
            entry
                .iter()
                .map(|(k, v)| (k.clone(), v.0.clone()))
                .collect()
        })
    }

    pub fn delete_service(&self, service: &str) -> Result<()> {
        let mut data = self.data.lock().unwrap();
        if data.entries.remove(service).is_none() {
            anyhow::bail!("标签 '{}' 不存在", service);
        }
        self.save_inner(&data)
    }

    pub fn list_services(&self) -> Vec<String> {
        let data = self.data.lock().unwrap();
        data.entries.keys().cloned().collect()
    }

    // ── 内部持久化: 原子写入 (tmp + rename + bak 回滚) ──────

    fn save_inner(&self, data: &VaultData) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string(data)?;
        let encrypted = encrypt(&json)?;

        let tmp = self.path.with_extension("tmp");
        let bak = self.path.with_extension("bak");

        // 保留旧文件的备份用于回滚
        let had_old = self.path.exists();
        if had_old {
            let _ = std::fs::remove_file(&bak);
            std::fs::rename(&self.path, &bak)?;
        }

        // 写临时文件
        if let Err(e) = std::fs::write(&tmp, &encrypted) {
            // 写入失败：从 bak 恢复
            if had_old {
                let _ = std::fs::rename(&bak, &self.path);
            }
            return Err(anyhow::anyhow!("凭证写入失败: {}", e));
        }

        // 原子替换
        if let Err(e) = std::fs::rename(&tmp, &self.path) {
            // rename 失败：从 bak 恢复
            if had_old {
                let _ = std::fs::rename(&bak, &self.path);
            }
            let _ = std::fs::remove_file(&tmp);
            return Err(anyhow::anyhow!("凭证保存失败: {}", e));
        }

        // 成功：清理 bak
        let _ = std::fs::remove_file(&bak);
        Ok(())
    }
}

// ── 脱敏模式注册 ──────────────────────────────────────────

impl CredentialVault {
    /// 提取所有已存储凭证的字段值，供 safe_log 脱敏模块使用
    pub fn extract_secret_patterns(&self) -> Vec<String> {
        let data = self.data.lock().unwrap();
        data.entries
            .values()
            .flat_map(|entry| entry.values().map(|v| v.0.clone()))
            .filter(|v| v.len() >= 8) // 跳过过短的值（可能是占位符）
            .collect()
    }
}
