use anyhow::Result;
use machine_key_management::{
    derive_key_from_machine_key, derive_key_pbkdf2_from_machine_key, get_machine_key_for_path,
};
use ring::aead::{Aad, LessSafeKey, Nonce, UnboundKey};
use ring::rand::{SecureRandom, SystemRandom};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use zeroize::{Zeroize, Zeroizing};

mod machine_key_management;

fn storage_root() -> String {
    std::env::var("QUANTPILOT_STORAGE_ROOT").unwrap_or_else(|_| "storage".into())
}
const NONCE_LEN: usize = 12;
const TAG_LEN: usize = 16;

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

// ── 加解密 ────────────────────────────────────────────────

/// 使用 PBKDF2 派生密钥加密，输出前 prepend 1 字节版本头 [2]。
fn encrypt_with_machine_key(plaintext: &str, machine_key: &[u8; 32]) -> Result<Vec<u8>> {
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
    let mut result = vec![2u8]; // version=2: PBKDF2
    result.extend(nonce_bytes);
    result.extend(data);
    Ok(result)
}

fn decrypt_with_machine_key(
    ciphertext: &[u8],
    machine_key: &[u8; 32],
) -> Result<Zeroizing<String>> {
    if ciphertext.is_empty() {
        anyhow::bail!("凭证数据为空");
    }

    // 读取第一个字节判断版本
    let version = ciphertext[0];

    // 根据版本选择密钥派生函数和数据偏移
    let (key, offset): (UnboundKey, usize) = match version {
        2 => (derive_key_pbkdf2_from_machine_key(machine_key)?, 1), // v2: PBKDF2, 跳过版本头
        1 => (derive_key_from_machine_key(machine_key)?, 1),        // v1: SHA-256, 跳过版本头
        _ => (derive_key_from_machine_key(machine_key)?, 0),        // 无版本头(旧文件): SHA-256
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

// ── Vault 类型 ────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct VaultData {
    entries: BTreeMap<String, BTreeMap<String, SecretString>>,
}

pub type CredentialFields = BTreeMap<String, String>;

pub struct CredentialVault {
    path: PathBuf,
    machine_key: [u8; 32],
    data: Mutex<VaultData>,
}

// ── Vault API ─────────────────────────────────────────────

impl CredentialVault {
    pub fn load() -> Result<Self> {
        Self::load_from_storage_root(storage_root())
    }

    pub(crate) fn load_from_storage_root<P: AsRef<Path>>(storage_root: P) -> Result<Self> {
        let storage_root = storage_root.as_ref();
        // 触发机器密钥初始化（可能失败，不再静默降级）
        let machine_key_path = storage_root.join(".machine_key");
        let machine_key = get_machine_key_for_path(&machine_key_path)?;

        let path = storage_root.join(".credentials");
        // v2.1.0: 崩溃恢复 — 若 .bak 残留且主文件不存在, 从 bak 恢复
        let bak = path.with_extension("bak");
        if !path.exists() && bak.exists() {
            eprintln!("[vault] 检测到 .bak 残留文件，正在恢复...");
            std::fs::rename(&bak, &path).map_err(|e| {
                anyhow::anyhow!(
                    "凭证备份恢复失败: {}，请手动检查 {} 和 {}",
                    e,
                    path.display(),
                    bak.display()
                )
            })?;
        }
        let data = if path.exists() {
            let encrypted = std::fs::read(&path)?;
            let decrypted = decrypt_with_machine_key(&encrypted, &machine_key)
                .map_err(|_| anyhow::anyhow!("凭证文件损坏或密钥不匹配, 请重新设置凭证"))?;
            // v2.1.x: JSON损坏时返回错误，不再静默清空凭证数据
            serde_json::from_str(&decrypted).map_err(|e| {
                anyhow::anyhow!(
                    "凭证JSON解析失败: {}，请重新设置凭证 (备份文件: {})",
                    e,
                    bak.display()
                )
            })?
        } else {
            // 首次启动: 创建空 vault 并持久化, 避免 API 返回 503
            let data = VaultData::default();
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let json = serde_json::to_string(&data)?;
            let encrypted = encrypt_with_machine_key(&json, &machine_key)?;
            crate::storage_lifecycle::atomic_write_secret_file(&path, &encrypted)?;
            data
        };
        Ok(Self {
            path,
            machine_key,
            data: Mutex::new(data),
        })
    }

    /// 按标签整体设置凭证字段，同一标签下的所有字段原子替换
    pub fn set_service(&self, service: &str, fields: CredentialFields) -> Result<()> {
        if fields.is_empty() {
            anyhow::bail!("凭证字段不能为空");
        }
        let mut data = self.data.lock().unwrap_or_else(|e| e.into_inner());
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
    pub fn get_service(&self, service: &str) -> Option<BTreeMap<String, Zeroizing<String>>> {
        // v1.1.11: 返回 Zeroizing 强制调用方在 Drop 时清零明文
        let data = self.data.lock().unwrap_or_else(|e| e.into_inner());
        data.entries.get(service).map(|entry| {
            entry
                .iter()
                .map(|(k, v)| (k.clone(), Zeroizing::new(v.0.clone())))
                .collect()
        })
    }

    pub fn delete_service(&self, service: &str) -> Result<()> {
        let mut data = self.data.lock().unwrap_or_else(|e| e.into_inner());
        if data.entries.remove(service).is_none() {
            anyhow::bail!("标签 '{}' 不存在", service);
        }
        self.save_inner(&data)
    }

    pub fn list_services(&self) -> Vec<String> {
        let data = self.data.lock().unwrap_or_else(|e| e.into_inner());
        data.entries.keys().cloned().collect()
    }

    // ── 内部持久化: 原子写入 (tmp + rename + bak 回滚) ──────

    fn save_inner(&self, data: &VaultData) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string(data)?;
        let encrypted = encrypt_with_machine_key(&json, &self.machine_key)?;

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
        // fsync tmp 文件确保密文落盘后再 rename
        if let Ok(f) = std::fs::File::open(&tmp) {
            let _ = f.sync_all();
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
        // fsync 父目录确保 rename 落盘
        if let Some(parent) = self.path.parent() {
            if let Ok(f) = std::fs::File::open(parent) {
                let _ = f.sync_all();
            }
        }

        // 成功：清理 bak，设置安全权限
        let _ = std::fs::remove_file(&bak);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&self.path, std::fs::Permissions::from_mode(0o600)).ok();
        }
        #[cfg(windows)]
        {
            let username = std::env::var("USERNAME").unwrap_or_default();
            let _ = std::process::Command::new("icacls")
                .args([
                    self.path.to_str().unwrap_or(""),
                    "/inheritance:r",
                    "/grant",
                    &format!("{}:F", username),
                ])
                .output();
        }
        Ok(())
    }
}

// ── 脱敏模式注册 ──────────────────────────────────────────

impl CredentialVault {
    /// 提取所有已存储凭证的字段值，供 safe_log 脱敏模块使用
    pub fn extract_secret_patterns(&self) -> Vec<Zeroizing<String>> {
        let data = self.data.lock().unwrap_or_else(|e| e.into_inner());
        data.entries
            .values()
            .flat_map(|entry| entry.values().map(|v| Zeroizing::new(v.0.clone())))
            .filter(|v| v.len() >= 4) // v2.5.0: 阈值 8→4, 防止短 API key 在日志中明文出现
            .collect()
    }
}

// ── 单元测试 ──

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::sync::{Mutex, OnceLock as SyncOnceLock};

    /// 全局 vault 测试锁：串行化 CWD 敏感的 vault 测试
    static VAULT_TEST_LOCK: SyncOnceLock<Mutex<()>> = SyncOnceLock::new();

    fn vault_lock() -> &'static Mutex<()> {
        VAULT_TEST_LOCK.get_or_init(|| Mutex::new(()))
    }

    struct VaultTestEnv {
        temp_dir: PathBuf,
    }

    impl VaultTestEnv {
        fn new() -> Self {
            let temp_dir =
                std::env::temp_dir().join(format!("quantpilot_vault_test_{}", std::process::id()));

            // 清除旧数据
            let _ = std::fs::remove_dir_all(&temp_dir);
            std::fs::create_dir_all(&temp_dir).unwrap();

            // 创建 storage 目录和机器密钥文件（仅首轮初始化时有效）
            let storage_dir = temp_dir.join("storage");
            std::fs::create_dir_all(&storage_dir).unwrap();

            let machine_key: [u8; 32] = [0xAB; 32];
            std::fs::write(storage_dir.join(".machine_key"), machine_key).unwrap();

            Self { temp_dir }
        }

        fn storage_dir(&self) -> PathBuf {
            self.temp_dir.join("storage")
        }

        fn load_vault(&self) -> CredentialVault {
            CredentialVault::load_from_storage_root(self.storage_dir()).unwrap()
        }

        /// 确保凭证文件不存在（模拟首次启动）
        fn clean_credentials(&self) {
            let creds = self.temp_dir.join("storage/.credentials");
            let _ = std::fs::remove_file(&creds);
            let _ = std::fs::remove_file(self.temp_dir.join("storage/.credentials.tmp"));
            let _ = std::fs::remove_file(self.temp_dir.join("storage/.credentials.bak"));
        }
    }

    impl Drop for VaultTestEnv {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.temp_dir);
        }
    }

    /// 运行一个 vault 测试用例（串行化）
    fn run_vault_test<F>(f: F)
    where
        F: FnOnce(),
    {
        let guard = vault_lock().lock().unwrap_or_else(|e| e.into_inner());
        f();
        drop(guard);
    }

    // ── CredentialVault::load() ──

    #[test]
    fn test_load_fresh_vault_creates_file() {
        run_vault_test(|| {
            let env = VaultTestEnv::new();
            env.clean_credentials();
            assert!(
                !env.temp_dir.join("storage/.credentials").exists(),
                "测试前置条件：凭证文件不应存在"
            );

            let vault = env.load_vault();

            // 加载成功后应创建凭证文件
            assert!(env.temp_dir.join("storage/.credentials").exists());
            assert!(vault.list_services().is_empty());
        });
    }

    #[test]
    fn test_load_existing_vault() {
        run_vault_test(|| {
            let env = VaultTestEnv::new();
            env.clean_credentials();

            // 首次创建 vault 并写入一个服务
            let vault = env.load_vault();
            let mut fields = CredentialFields::new();
            fields.insert("api_key".to_string(), "test_secret_value".to_string());
            vault.set_service("test_service", fields).unwrap();

            // 重新加载 vault（模拟重启）
            let reloaded = env.load_vault();
            let services = reloaded.list_services();
            assert_eq!(services, vec!["test_service"]);
        });
    }

    // ── set_service + get_service ──

    #[test]
    fn test_set_and_get_service_roundtrip() {
        run_vault_test(|| {
            let env = VaultTestEnv::new();
            env.clean_credentials();

            let vault = env.load_vault();
            let mut fields = CredentialFields::new();
            fields.insert("key1".to_string(), "value1".to_string());
            fields.insert("key2".to_string(), "value2".to_string());

            vault.set_service("my_service", fields).unwrap();

            let result = vault.get_service("my_service");
            assert!(result.is_some());
            let entry = result.unwrap();
            assert_eq!(entry.get("key1").map(|z| z.as_str()), Some("value1"));
            assert_eq!(entry.get("key2").map(|z| z.as_str()), Some("value2"));
        });
    }

    #[test]
    fn test_set_service_rejects_empty_fields() {
        run_vault_test(|| {
            let env = VaultTestEnv::new();
            env.clean_credentials();

            let vault = env.load_vault();
            let fields = CredentialFields::new();
            let result = vault.set_service("empty_service", fields);
            assert!(result.is_err());
        });
    }

    #[test]
    fn test_set_service_overwrites_existing() {
        run_vault_test(|| {
            let env = VaultTestEnv::new();
            env.clean_credentials();

            let vault = env.load_vault();

            let mut fields1 = CredentialFields::new();
            fields1.insert("old".to_string(), "old_value".to_string());
            vault.set_service("svc", fields1).unwrap();

            let mut fields2 = CredentialFields::new();
            fields2.insert("new".to_string(), "new_value".to_string());
            vault.set_service("svc", fields2).unwrap();

            let result = vault.get_service("svc").unwrap();
            assert_eq!(result.get("old"), None);
            assert_eq!(result.get("new").map(|z| z.as_str()), Some("new_value"));
        });
    }

    #[test]
    fn test_get_service_nonexistent() {
        run_vault_test(|| {
            let env = VaultTestEnv::new();
            env.clean_credentials();

            let vault = env.load_vault();
            let result = vault.get_service("nonexistent");
            assert!(result.is_none());
        });
    }

    // ── delete_service ──

    #[test]
    fn test_delete_service_removes_service() {
        run_vault_test(|| {
            let env = VaultTestEnv::new();
            env.clean_credentials();

            let vault = env.load_vault();

            let mut fields = CredentialFields::new();
            fields.insert("key".to_string(), "value".to_string());
            vault.set_service("to_delete", fields).unwrap();
            assert!(vault.get_service("to_delete").is_some());

            vault.delete_service("to_delete").unwrap();
            assert!(vault.get_service("to_delete").is_none());
        });
    }

    #[test]
    fn test_delete_service_nonexistent_returns_error() {
        run_vault_test(|| {
            let env = VaultTestEnv::new();
            env.clean_credentials();

            let vault = env.load_vault();
            let result = vault.delete_service("does_not_exist");
            assert!(result.is_err());
        });
    }

    #[test]
    fn test_delete_service_persists_after_reload() {
        run_vault_test(|| {
            let env = VaultTestEnv::new();
            env.clean_credentials();

            let vault1 = env.load_vault();
            let mut fields = CredentialFields::new();
            fields.insert("key".to_string(), "keep".to_string());
            vault1.set_service("keep_me", fields).unwrap();

            let mut fields2 = CredentialFields::new();
            fields2.insert("key".to_string(), "remove".to_string());
            vault1.set_service("remove_me", fields2).unwrap();
            vault1.delete_service("remove_me").unwrap();
            drop(vault1);

            let vault2 = env.load_vault();
            let services = vault2.list_services();
            assert_eq!(services, vec!["keep_me"]);
        });
    }

    // ── list_services ──

    #[test]
    fn test_list_services_empty_vault() {
        run_vault_test(|| {
            let env = VaultTestEnv::new();
            env.clean_credentials();

            let vault = env.load_vault();
            let services = vault.list_services();
            assert!(services.is_empty());
        });
    }

    #[test]
    fn test_list_services_returns_all_services() {
        run_vault_test(|| {
            let env = VaultTestEnv::new();
            env.clean_credentials();

            let vault = env.load_vault();

            let mut f1 = CredentialFields::new();
            f1.insert("k".to_string(), "v".to_string());
            vault.set_service("svc_a", f1).unwrap();

            let mut f2 = CredentialFields::new();
            f2.insert("k".to_string(), "v".to_string());
            vault.set_service("svc_b", f2).unwrap();

            let mut f3 = CredentialFields::new();
            f3.insert("k".to_string(), "v".to_string());
            vault.set_service("svc_c", f3).unwrap();

            let mut services = vault.list_services();
            services.sort();
            assert_eq!(services, vec!["svc_a", "svc_b", "svc_c"]);
        });
    }

    // ── extract_secret_patterns ──

    #[test]
    fn test_extract_secret_patterns_returns_values() {
        run_vault_test(|| {
            let env = VaultTestEnv::new();
            env.clean_credentials();

            let vault = env.load_vault();

            let mut fields = CredentialFields::new();
            fields.insert("api_key".to_string(), "my_long_api_key_12345".to_string());
            vault.set_service("exchange", fields).unwrap();

            let patterns = vault.extract_secret_patterns();
            assert!(!patterns.is_empty());
            // 返回值应 >= 8 字符
            for p in &patterns {
                assert!(p.len() >= 8);
            }
            assert!(patterns
                .iter()
                .any(|p| p.as_str() == "my_long_api_key_12345"));
        });
    }

    #[test]
    fn test_extract_secret_patterns_skips_short_values() {
        run_vault_test(|| {
            let env = VaultTestEnv::new();
            env.clean_credentials();

            let vault = env.load_vault();

            let mut fields = CredentialFields::new();
            fields.insert("short".to_string(), "abc".to_string()); // 3 chars, < 8
            vault.set_service("test", fields).unwrap();

            let patterns = vault.extract_secret_patterns();
            // 只有 >= 8 的值才会返回
            assert!(patterns.is_empty());
        });
    }

    #[test]
    fn test_extract_secret_patterns_returns_zeroizing() {
        run_vault_test(|| {
            let env = VaultTestEnv::new();
            env.clean_credentials();

            let vault = env.load_vault();

            let mut fields = CredentialFields::new();
            fields.insert("secret".to_string(), "very_long_secret_value".to_string());
            vault.set_service("test", fields).unwrap();

            let patterns = vault.extract_secret_patterns();
            for p in &patterns {
                // Zeroizing 包装验证：类型应为 Zeroizing<String>
                let _: &Zeroizing<String> = p;
            }
        });
    }
}
