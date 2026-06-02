# v4.16.0 backend.storage_security.credential_vault_implementation safety baseline and extraction plan

> Batch: BE-001JM-01
> Node: `backend.storage_security.credential_vault_implementation`
> Parent: `backend.storage_security`
> Stage: `baseline_plan`
> Movement: no code movement.

---

## Summary

backend.storage_security.credential_vault_implementation safety baseline and extraction plan

Frozen implementation boundary:

- `src/credential_vault.rs` currently owns vault implementation and tests.
- Public surface: `CredentialFields`, `CredentialVault::load`, `CredentialVault::set_service`, `CredentialVault::get_service`, `CredentialVault::delete_service`, `CredentialVault::list_services`, and `CredentialVault::extract_secret_patterns`.
- Crate-visible surface: `CredentialVault::load_from_storage_root`.
- Private safety helpers: `storage_root`, `absolute_path`, `SecretString`, `get_machine_key_for_path`, `derive_key_from_machine_key`, `derive_key_pbkdf2_from_machine_key`, `encrypt_with_machine_key`, `decrypt_with_machine_key`, `VaultData`, and `save_inner`.

Allowed next movement:

- Move the vault implementation under `backend.storage_security` only as an equivalent owner extraction.
- Keep `src/credential_vault.rs` as a compatibility shim if root callers still import `crate::credential_vault`.
- Preserve all public type names, method names, visibility, tests, and call sites.

Forbidden next movement:

- Do not change encryption algorithm selection, PBKDF2 iterations, version-byte handling, nonce/tag layout, AAD, key cache locking, machine-key file path, backup restore behavior, atomic write behavior, permission hardening, JSON parse error behavior, `Zeroizing` handling, or service CRUD semantics.
- Do not change `src/credential_api.rs` handler behavior, `AppState` vault ownership, storage lifecycle helpers, safe-log redaction, auth, quota, or backup semantics.
- Do not introduce sibling horizontal links; callers must continue through root compatibility or `backend.storage_security`.

Required proof if code moves in BE-001JM-02:

- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p quantpilot credential_vault --lib`
- `cargo test -p quantpilot credential --lib`
- governance gates listed below

## Boundary

**Real files**:
- `src/credential_vault.rs`
- `src/backend/storage_security/credential_vault.rs`
- `src/storage_lifecycle.rs`
- `src/backup.rs`

**Markers**:
- `BE-001JM-01`
- `safety_baseline_frozen`
- `vault implementation owner`
- `crypto persistence restore crud protected`
- `release_transition_guard`

**Next step**:
BE-001JM-02 backend.storage_security.credential_vault_implementation extract_closeout

---

## Gates

- `git diff --check`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
