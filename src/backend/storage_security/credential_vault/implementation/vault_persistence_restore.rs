use super::crypto_codec::{decrypt_with_machine_key, encrypt_with_machine_key};
use super::machine_key_management::get_machine_key_for_path;
use super::{CredentialVault, VaultData};
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

pub(super) fn save_inner(path: &Path, machine_key: &[u8; 32], data: &VaultData) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string(data)?;
    let encrypted = encrypt_with_machine_key(&json, machine_key)?;

    let tmp = path.with_extension("tmp");
    let bak = path.with_extension("bak");

    let had_old = path.exists();
    if had_old {
        let _ = std::fs::remove_file(&bak);
        std::fs::rename(path, &bak)?;
    }

    if let Err(e) = std::fs::write(&tmp, &encrypted) {
        if had_old {
            let _ = std::fs::rename(&bak, path);
        }
        return Err(anyhow::anyhow!("凭证写入失败: {}", e));
    }
    if let Ok(f) = std::fs::File::open(&tmp) {
        let _ = f.sync_all();
    }

    if let Err(e) = std::fs::rename(&tmp, path) {
        if had_old {
            let _ = std::fs::rename(&bak, path);
        }
        let _ = std::fs::remove_file(&tmp);
        return Err(anyhow::anyhow!("凭证保存失败: {}", e));
    }
    if let Some(parent) = path.parent() {
        if let Ok(f) = std::fs::File::open(parent) {
            let _ = f.sync_all();
        }
    }

    let _ = std::fs::remove_file(&bak);
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600)).ok();
    }
    #[cfg(windows)]
    {
        let username = std::env::var("USERNAME").unwrap_or_default();
        let _ = std::process::Command::new("icacls")
            .args([
                path.to_str().unwrap_or(""),
                "/inheritance:r",
                "/grant",
                &format!("{}:F", username),
            ])
            .output();
    }
    Ok(())
}
