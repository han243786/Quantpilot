use super::crypto_codec::{decrypt_with_machine_key, encrypt_with_machine_key};
use super::machine_key_management::get_machine_key_for_path;
use super::{CredentialVault, VaultData};
use anyhow::Result;
use std::path::Path;

mod atomic_save_commit;
mod load_restore_entry;

pub(super) fn load_from_storage_root<P: AsRef<Path>>(storage_root: P) -> Result<CredentialVault> {
    load_restore_entry::load_from_storage_root(storage_root)
}

pub(super) fn save_inner(path: &Path, machine_key: &[u8; 32], data: &VaultData) -> Result<()> {
    atomic_save_commit::save_inner(path, machine_key, data)
}
