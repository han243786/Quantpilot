# v4.16.0 backend.storage_security.credential_vault_implementation.vault_persistence_restore equivalence baseline and extraction plan

> Batch: BE-001JT-01
> Node: `backend.storage_security.credential_vault_implementation.vault_persistence_restore`
> Parent: `backend.storage_security.credential_vault_implementation`
> Stage: `baseline_plan`
> Movement: No code movement.

---

## Summary

`backend.storage_security.credential_vault_implementation.vault_persistence_restore` equivalence baseline and extraction plan are frozen.

This child owns only the persistence and recovery lifecycle inside `src/backend/storage_security/credential_vault/implementation.rs`:

- `CredentialVault::load`
- `CredentialVault::load_from_storage_root`
- `.machine_key` handoff through `get_machine_key_for_path`
- `.credentials` path selection
- `.bak` restore when the primary credentials file is missing
- encrypted credentials file read/decode
- JSON parse into `VaultData`
- first-run empty vault creation and initial encrypted write
- `save_inner`
- tmp/bak/rename rollback
- file and parent directory sync best-effort calls
- Unix `0o600` permission hardening and Windows `icacls` hardening

It may call the already closed `machine_key_management` and `crypto_codec` children, but it does not own their internals. It also does not own service field validation, service map mutation, list/get/delete semantics, secret pattern extraction, root compatibility shim behavior, or release transition.

BE-001JT-02 may move this owner pocket into a true child module under `src/backend/storage_security/credential_vault/implementation/` only if `implementation.rs` remains the parent owner for public `CredentialVault` methods and existing behavior remains equivalent.

## Equivalence Baseline

| Surface | Frozen behavior |
| --- | --- |
| Public load entry | `CredentialVault::load()` delegates to `load_from_storage_root(storage_root())`; `storage_root()` keeps `QUANTPILOT_STORAGE_ROOT` fallback to `storage`. |
| Storage paths | `load_from_storage_root` uses `<storage_root>/.machine_key` and `<storage_root>/.credentials`; backup path remains `.credentials.bak`. |
| Machine-key handoff | The child may call `get_machine_key_for_path`, but machine-key cache/init/file semantics remain owned by `machine_key_management`. |
| Backup restore | If primary credentials file is missing and `.bak` exists, `.bak` is renamed to the primary path; restore failure returns an `anyhow` error with path context. |
| Existing vault read | Existing primary file is read as encrypted bytes, decrypted through `crypto_codec`, then parsed with `serde_json::from_str`. |
| Corrupt vault handling | Decrypt failure maps to a reset-required error; JSON parse failure returns an error and does not silently clear credentials. |
| Fresh vault creation | Missing primary file creates default `VaultData`, ensures parent directory exists, serializes JSON, encrypts it, and writes through `crate::storage_lifecycle::atomic_write_secret_file`. |
| Save lifecycle | `save_inner` ensures parent directory, serializes data, encrypts with current `machine_key`, writes tmp, fsyncs tmp best-effort, renames tmp to primary, fsyncs parent best-effort, deletes backup, then hardens permissions. |
| Rollback behavior | If old primary exists, it is renamed to `.bak`; write/rename failures attempt to restore `.bak` and remove tmp on rename failure. |
| Permission hardening | Unix keeps best-effort `0o600`; Windows keeps best-effort `icacls /inheritance:r /grant USERNAME:F`. |

## Extraction Plan

1. Create `src/backend/storage_security/credential_vault/implementation/vault_persistence_restore.rs`.
2. Move `load_from_storage_root` support logic and `save_inner` into that child as parent-only helpers.
3. Keep `CredentialVault::load`, `set_service`, `delete_service`, `list_services`, `get_service`, and `extract_secret_patterns` public methods in `implementation.rs` unless a later child baseline authorizes movement.
4. If helper extraction needs access to `CredentialVault` fields or `VaultData`, use `pub(super)` helper functions and keep parent-owned type definitions in `implementation.rs`.
5. Do not change `machine_key_management`, `crypto_codec`, service CRUD, secret pattern extraction, root shim, or release transition.
6. Run `cargo fmt --check`, `cargo check -p quantpilot`, `cargo test -p quantpilot credential_vault --lib`, and `cargo test -p quantpilot credential --lib`.

## Boundary

**Real files**:
- `src/backend/storage_security/credential_vault/implementation.rs`
- `src/backend/storage_security/credential_vault/implementation/crypto_codec.rs`
- `src/backend/storage_security/credential_vault/implementation/machine_key_management.rs`
- `src/backend/storage_security/credential_vault.rs`
- `src/credential_vault.rs`

**Markers**:
- `BE-001JT-01`
- `vault_persistence_restore baseline_frozen`
- `vault_persistence_restore plan_frozen`
- `no code movement`
- `release_transition_guard`

**Next step**:
BE-001JT-02 backend.storage_security.credential_vault_implementation.vault_persistence_restore extract_closeout

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
