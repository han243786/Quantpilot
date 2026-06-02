# v4.16.0 backend.storage_security.credential_vault re-export facade baseline and plan

> Batch: BE-001JJ-01
> Node: `backend.storage_security.credential_vault`
> Parent: `backend.storage_security`
> Stage: `baseline_plan`
> Movement: no code movement.

---

## Summary

backend.storage_security.credential_vault re-export facade baseline and plan

Frozen current boundary:

- `src/backend/storage_security/credential_vault.rs` owns only `MODULE_ID` and `pub use crate::credential_vault::CredentialVault`.
- `src/backend/storage_security.rs` re-exports `credential_vault::CredentialVault` through the parent facade.
- `src/credential_vault.rs` remains the implementation owner for encryption, machine-key initialization, PBKDF2, nonce/tag handling, backup restore, atomic secret writes, vault data schema, and service CRUD.

Allowed next movement:

- Confirm the existing backend child re-export facade as extraction complete.
- Keep root vault implementation in place.

Forbidden next movement:

- Do not move `CredentialVault`, `CredentialFields`, `VaultData`, `SecretString`, machine-key helpers, encryption/decryption helpers, or persistence code.
- Do not change key derivation, nonce/tag layout, backup restore, atomic write, storage root, JSON parsing, or zeroize behavior.
- Do not alter credential API handler behavior or AppState vault ownership.

## Boundary

**Real files**:
- `src/backend/storage_security/credential_vault.rs`
- `src/backend/storage_security.rs`
- `src/credential_vault.rs`

**Markers**:
- `BE-001JJ-01`
- `baseline_frozen`
- `type re-export facade`
- `vault implementation paused`
- `release_transition_guard`

**Next step**:
BE-001JJ-02 backend.storage_security.credential_vault extract_closeout

---

## Gates

- `git diff --check`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
