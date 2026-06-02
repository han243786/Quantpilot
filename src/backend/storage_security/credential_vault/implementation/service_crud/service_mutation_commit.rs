use super::super::{CredentialFields, CredentialVault, SecretString};
use anyhow::Result;
use std::collections::BTreeMap;

pub(super) fn set_service(
    vault: &CredentialVault,
    service: &str,
    fields: CredentialFields,
) -> Result<()> {
    if fields.is_empty() {
        anyhow::bail!("凭证字段不能为空");
    }
    let mut data = vault.data.lock().unwrap_or_else(|e| e.into_inner());
    let entry: BTreeMap<String, SecretString> = fields
        .into_iter()
        .map(|(k, v)| (k, SecretString(v)))
        .collect();
    data.entries.insert(service.to_string(), entry);
    vault.save_inner(&data)
}

pub(super) fn delete_service(vault: &CredentialVault, service: &str) -> Result<()> {
    let mut data = vault.data.lock().unwrap_or_else(|e| e.into_inner());
    if data.entries.remove(service).is_none() {
        anyhow::bail!("标签 '{}' 不存在", service);
    }
    vault.save_inner(&data)
}
