use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Mutex;
use zeroize::Zeroize;

pub(super) fn storage_root() -> String {
    std::env::var("QUANTPILOT_STORAGE_ROOT").unwrap_or_else(|_| "storage".into())
}

#[derive(Debug, Clone)]
pub(super) struct SecretString(pub(super) String);

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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub(super) struct VaultData {
    pub(super) entries: BTreeMap<String, BTreeMap<String, SecretString>>,
}

pub type CredentialFields = BTreeMap<String, String>;

pub struct CredentialVault {
    pub(super) path: PathBuf,
    pub(super) machine_key: [u8; 32],
    pub(super) data: Mutex<VaultData>,
}
