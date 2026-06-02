use super::super::{CredentialVault, VaultData};
use super::{decrypt_with_machine_key, encrypt_with_machine_key, get_machine_key_for_path};
use anyhow::Result;
use std::path::Path;
use std::sync::Mutex;

pub(super) fn load_from_storage_root<P: AsRef<Path>>(storage_root: P) -> Result<CredentialVault> {
    let storage_root = storage_root.as_ref();
    let machine_key_path = storage_root.join(".machine_key");
    let machine_key = get_machine_key_for_path(&machine_key_path)?;

    let path = storage_root.join(".credentials");
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
        serde_json::from_str(&decrypted).map_err(|e| {
            anyhow::anyhow!(
                "凭证JSON解析失败: {}，请重新设置凭证 (备份文件: {})",
                e,
                bak.display()
            )
        })?
    } else {
        let data = VaultData::default();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string(&data)?;
        let encrypted = encrypt_with_machine_key(&json, &machine_key)?;
        crate::storage_lifecycle::atomic_write_secret_file(&path, &encrypted)?;
        data
    };
    Ok(CredentialVault {
        path,
        machine_key,
        data: Mutex::new(data),
    })
}
