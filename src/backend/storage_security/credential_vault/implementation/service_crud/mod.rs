mod service_mutation_commit;
mod service_read_projection;

use super::{CredentialFields, CredentialVault};
use anyhow::Result;
use std::collections::BTreeMap;
use zeroize::Zeroizing;

pub(super) fn set_service(
    vault: &CredentialVault,
    service: &str,
    fields: CredentialFields,
) -> Result<()> {
    service_mutation_commit::set_service(vault, service, fields)
}

pub(super) fn get_service(
    vault: &CredentialVault,
    service: &str,
) -> Option<BTreeMap<String, Zeroizing<String>>> {
    service_read_projection::get_service(vault, service)
}

pub(super) fn delete_service(vault: &CredentialVault, service: &str) -> Result<()> {
    service_mutation_commit::delete_service(vault, service)
}

pub(super) fn list_services(vault: &CredentialVault) -> Vec<String> {
    service_read_projection::list_services(vault)
}
