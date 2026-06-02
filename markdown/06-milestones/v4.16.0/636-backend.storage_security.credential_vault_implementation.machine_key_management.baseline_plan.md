# v4.16.0 backend.storage_security.credential_vault_implementation.machine_key_management equivalence baseline and extraction plan

> Batch: BE-001JP-01
> Node: `backend.storage_security.credential_vault_implementation.machine_key_management`
> Parent: `backend.storage_security.credential_vault_implementation`
> Stage: `baseline_plan`
> Movement: No code movement.

---

## Summary

`backend.storage_security.credential_vault_implementation.machine_key_management` equivalence baseline and extraction plan are frozen.

This child owns only the machine-key bootstrap path inside `src/backend/storage_security/credential_vault/implementation.rs`:

- `MACHINE_KEYS`: process-local cache keyed by absolute key file path.
- `MACHINE_KEY_INIT_LOCK`: initialization lock that prevents key creation races.
- `get_machine_key_for_path`: absolute path normalization, cache hit, double-check after init lock, key file read, new 32-byte random key generation, secret atomic key write, cache insert, and error propagation.
- `derive_key_from_machine_key`: legacy v1 SHA-256 based key derivation using host name and machine key hex.
- `derive_key_pbkdf2_from_machine_key`: v2 PBKDF2-HMAC-SHA256 derivation with 600,000 iterations and host-scoped salt.

It does not own AES-GCM encrypt/decrypt framing, nonce/tag handling, vault JSON schema, backup restore, atomic vault save, service CRUD, secret pattern extraction, root compatibility shim behavior, or release transition.

BE-001JP-02 may move this owner pocket into a child module under `src/backend/storage_security/credential_vault/` only if the parent continues to mediate access and existing public `CredentialVault` behavior remains equivalent.

## Equivalence Baseline

| Surface | Frozen behavior |
| --- | --- |
| Path input | `get_machine_key_for_path` receives the storage root key file path from `CredentialVault::load_from_storage_root`; path is normalized through `absolute_path` before cache lookup. |
| Cache behavior | Cache lookup happens before taking `MACHINE_KEY_INIT_LOCK`, then is checked again after taking the lock to avoid duplicate key creation. Poisoned mutexes recover through `into_inner()`. |
| Existing key file | Existing files must read exactly 32 bytes; malformed key length returns an error. |
| New key file | Missing key files generate 32 random bytes using `SystemRandom`, then persist through `crate::storage_lifecycle::atomic_write_secret_file`. |
| Derivation v1 | `derive_key_from_machine_key` keeps the `quantpilot-credential-vault-{host}-{hex}` seed and SHA-256 to AES-256-GCM key behavior. |
| Derivation v2 | `derive_key_pbkdf2_from_machine_key` keeps `quantpilot-vault-v2-{host}` salt, PBKDF2-HMAC-SHA256, 600,000 iterations, and AES-256-GCM key output. |
| Error shape | Existing `anyhow` failures and propagation remain equivalent; no new public error enum is introduced. |
| Parent mediation | Public callers still enter through `CredentialVault`; root `src/credential_vault.rs` remains a compatibility shim. |

## Extraction Plan

1. Create `src/backend/storage_security/credential_vault/machine_key_management.rs`.
2. Move `MACHINE_KEYS`, `MACHINE_KEY_INIT_LOCK`, `get_machine_key_for_path`, `derive_key_from_machine_key`, and `derive_key_pbkdf2_from_machine_key` into that child.
3. Keep functions `pub(super)` or `pub(crate)` only as needed by `implementation.rs`; do not expose a new public API.
4. Add `mod machine_key_management;` at the `credential_vault` parent level.
5. Update `implementation.rs` imports/calls to use the parent-owned child without changing encrypt/decrypt, persistence, CRUD, or tests.
6. Run `cargo fmt --check`, `cargo check -p quantpilot`, `cargo test -p quantpilot credential_vault --lib`, and `cargo test -p quantpilot credential --lib`.

## Boundary

**Real files**:
- `src/backend/storage_security/credential_vault/implementation.rs`
- `src/backend/storage_security/credential_vault.rs`
- `src/credential_vault.rs`

**Markers**:
- `BE-001JP-01`
- `machine_key_management baseline_frozen`
- `machine_key_management plan_frozen`
- `no code movement`
- `release_transition_guard`

**Next step**:
BE-001JP-02 backend.storage_security.credential_vault_implementation.machine_key_management extract_closeout

---

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p quantpilot credential_vault --lib`
- `cargo test -p quantpilot credential --lib`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
