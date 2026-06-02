use anyhow::Result;
use std::collections::BTreeMap;
use std::path::Path;
use zeroize::Zeroizing;

mod crypto_codec;
mod machine_key_management;
mod secret_pattern_extraction;
mod service_crud;
mod type_surface;
mod vault_persistence_restore;

use type_surface::{storage_root, SecretString, VaultData};
pub use type_surface::{CredentialFields, CredentialVault};

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
        secret_pattern_extraction::extract_secret_patterns(self)
    }
}

// ── 单元测试 ──

#[cfg(test)]
mod tests;
