use super::crypto_codec::{decrypt_with_machine_key, encrypt_with_machine_key};
use super::machine_key_management::get_machine_key_for_path;
use super::{CredentialVault, VaultData};
use anyhow::Result;
use std::path::Path;

mod load_restore_entry;

pub(super) fn load_from_storage_root<P: AsRef<Path>>(storage_root: P) -> Result<CredentialVault> {
    load_restore_entry::load_from_storage_root(storage_root)
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
