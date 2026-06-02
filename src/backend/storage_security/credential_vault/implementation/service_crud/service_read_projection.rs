use super::super::CredentialVault;
use std::collections::BTreeMap;
use zeroize::Zeroizing;

pub(super) fn get_service(
    vault: &CredentialVault,
    service: &str,
) -> Option<BTreeMap<String, Zeroizing<String>>> {
    let data = vault.data.lock().unwrap_or_else(|e| e.into_inner());
    data.entries.get(service).map(|entry| {
        entry
            .iter()
            .map(|(k, v)| (k.clone(), Zeroizing::new(v.0.clone())))
            .collect()
    })
}

pub(super) fn list_services(vault: &CredentialVault) -> Vec<String> {
    let data = vault.data.lock().unwrap_or_else(|e| e.into_inner());
    data.entries.keys().cloned().collect()
}
