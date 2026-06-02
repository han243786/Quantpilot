pub const MODULE_ID: &str = "backend.storage_security.credential_vault";

mod implementation;

pub use implementation::{CredentialFields, CredentialVault};
