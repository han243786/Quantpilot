use super::CredentialVault;
use zeroize::Zeroizing;

pub(super) fn extract_secret_patterns(vault: &CredentialVault) -> Vec<Zeroizing<String>> {
    let data = vault.data.lock().unwrap_or_else(|e| e.into_inner());
    data.entries
        .values()
        .flat_map(|entry| entry.values().map(|v| Zeroizing::new(v.0.clone())))
        .filter(|v| v.len() >= 4)
        .collect()
}
