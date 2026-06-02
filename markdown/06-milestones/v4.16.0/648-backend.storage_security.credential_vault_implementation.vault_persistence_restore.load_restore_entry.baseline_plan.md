# v4.16.0 backend.storage_security.credential_vault_implementation.vault_persistence_restore.load_restore_entry equivalence baseline and extraction plan

> Batch: BE-001JV-01
> Node: `backend.storage_security.credential_vault_implementation.vault_persistence_restore.load_restore_entry`
> Parent: `backend.storage_security.credential_vault_implementation.vault_persistence_restore`
> Stage: `baseline_plan`
> Movement: No code movement.

---

## Summary

`backend.storage_security.credential_vault_implementation.vault_persistence_restore.load_restore_entry` equivalence baseline and extraction plan are frozen.

This child owns only the load/restore/create phase inside `src/backend/storage_security/credential_vault/implementation/vault_persistence_restore.rs`:

- storage-root path normalization through `AsRef<Path>`
- `.machine_key` path derivation and `get_machine_key_for_path` handoff
- `.credentials` and `.credentials.bak` path selection
- `.bak` restore when the primary credentials file is missing
- encrypted credentials file read
- decrypt handoff through `crypto_codec`
- JSON parse into `VaultData`
- first-run default `VaultData` creation
- first-run parent directory creation
- first-run encrypted write through `crate::storage_lifecycle::atomic_write_secret_file`
- `CredentialVault` construction with `path`, `machine_key`, and `Mutex<VaultData>`

It does not own `save_inner`, tmp/bak save rollback, fsync best-effort calls, backup cleanup, Unix/Windows permission hardening, service CRUD, secret pattern extraction, machine-key internals, crypto internals, root shim behavior, or release transition.

BE-001JV-02 may move this load/restore owner pocket into a true child module under `src/backend/storage_security/credential_vault/implementation/vault_persistence_restore/` only if `vault_persistence_restore.rs` remains the parent child that mediates all calls and existing `CredentialVault` behavior remains equivalent.

## Equivalence Baseline

| Surface | Frozen behavior |
| --- | --- |
| Entry input | `load_from_storage_root<P: AsRef<Path>>` accepts the same generic storage-root input and calls `storage_root.as_ref()` exactly once before path derivation. |
| Machine-key handoff | The load child derives `<storage_root>/.machine_key` and calls `get_machine_key_for_path`; cache/init/key-file semantics remain owned by `machine_key_management`. |
| Vault paths | Primary path remains `<storage_root>/.credentials`; backup path remains `.credentials.bak` via `path.with_extension("bak")`. |
| Backup restore | If primary is missing and backup exists, backup is renamed to primary before any read; restore failure returns an `anyhow` error with both path displays. |
| Existing read | Existing primary file is read as encrypted bytes, decrypted by `decrypt_with_machine_key`, and parsed by `serde_json::from_str`. |
| Corrupt handling | Decrypt failure maps to the reset-required vault error; JSON parse failure returns an error with backup path context and never silently clears credentials. |
| Fresh create | Missing primary file creates `VaultData::default()`, ensures the parent directory exists, serializes JSON, encrypts it, and writes encrypted bytes through `atomic_write_secret_file`. |
| Return shape | The helper returns a `CredentialVault` containing the selected primary path, loaded machine key, and `Mutex::new(data)`. |
| Parent mediation | `vault_persistence_restore.rs` remains the parent child and exports only `pub(super)` helpers back to `implementation.rs`. |

## Extraction Plan

1. Create `src/backend/storage_security/credential_vault/implementation/vault_persistence_restore/load_restore_entry.rs`.
2. Move the body of `load_from_storage_root` into a `pub(super)` helper owned by that child.
3. Keep `vault_persistence_restore.rs` as the parent child: it declares `mod load_restore_entry;` and delegates `load_from_storage_root` to the new child.
4. Keep `save_inner` and all tmp/bak save rollback, fsync, backup cleanup, and permission hardening in `vault_persistence_restore.rs` until a later `atomic_save_commit` baseline authorizes movement.
5. Keep `CredentialVault`, `VaultData`, public CRUD methods, secret pattern extraction, machine-key child, crypto child, root shim, and release transition unchanged.
6. Run `cargo fmt --check`, `cargo check -p quantpilot`, `cargo test -p quantpilot credential_vault --lib`, and `cargo test -p quantpilot credential --lib`.

## Boundary

**Real files**:
- `src/backend/storage_security/credential_vault/implementation/vault_persistence_restore.rs`
- `src/backend/storage_security/credential_vault/implementation.rs`
- `src/backend/storage_security/credential_vault/implementation/crypto_codec.rs`
- `src/backend/storage_security/credential_vault/implementation/machine_key_management.rs`

**Markers**:
- `BE-001JV-01`
- `load_restore_entry baseline_frozen`
- `load_restore_entry plan_frozen`
- `atomic_save_commit remains_residual`
- `no code movement`
- `release_transition_guard`

**Next step**:
BE-001JV-02 backend.storage_security.credential_vault_implementation.vault_persistence_restore.load_restore_entry extract_closeout

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
