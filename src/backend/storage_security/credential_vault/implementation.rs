use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use zeroize::{Zeroize, Zeroizing};

mod crypto_codec;
mod machine_key_management;
mod service_crud;
mod vault_persistence_restore;

fn storage_root() -> String {
    std::env::var("QUANTPILOT_STORAGE_ROOT").unwrap_or_else(|_| "storage".into())
}
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
        vault_persistence_restore::load_from_storage_root(storage_root)
    }

    /// 按标签整体设置凭证字段，同一标签下的所有字段原子替换
    pub fn set_service(&self, service: &str, fields: CredentialFields) -> Result<()> {
        service_crud::set_service(self, service, fields)
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
        service_crud::get_service(self, service)
    }

    pub fn delete_service(&self, service: &str) -> Result<()> {
        service_crud::delete_service(self, service)
    }

    pub fn list_services(&self) -> Vec<String> {
        service_crud::list_services(self)
    }

    // ── 内部持久化: 原子写入 (tmp + rename + bak 回滚) ──────

    fn save_inner(&self, data: &VaultData) -> Result<()> {
        vault_persistence_restore::save_inner(&self.path, &self.machine_key, data)
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
