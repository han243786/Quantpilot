# v4.16.0 backend.storage_security.credential_vault_implementation.vault_persistence_restore.atomic_save_commit equivalence baseline and extraction plan

> Batch: BE-001JX-01
> Node: `backend.storage_security.credential_vault_implementation.vault_persistence_restore.atomic_save_commit`
> Parent: `backend.storage_security.credential_vault_implementation.vault_persistence_restore`
> Stage: `baseline_plan`
> Movement: No code movement.

---

## Summary

`backend.storage_security.credential_vault_implementation.vault_persistence_restore.atomic_save_commit` equivalence baseline and extraction plan are frozen.

This child owns only the save/commit phase inside `src/backend/storage_security/credential_vault/implementation/vault_persistence_restore.rs`:

- parent directory creation before write
- `VaultData` JSON serialization
- encryption handoff through `encrypt_with_machine_key`
- `.tmp` and `.bak` path derivation
- existing primary backup into `.bak`
- stale backup removal before backup
- encrypted tmp write
- rollback from `.bak` on tmp write failure
- tmp file fsync best-effort
- tmp-to-primary rename
- rollback from `.bak` on rename failure
- tmp cleanup on rename failure
- parent directory fsync best-effort
- successful backup cleanup
- Unix `0o600` permission hardening
- Windows `icacls /inheritance:r /grant USERNAME:F` hardening

It does not own `load_restore_entry`, load/restore/create behavior, service CRUD map mutation, secret pattern extraction, machine-key internals, crypto internals, root shim behavior, or release transition.

BE-001JX-02 may move this save/commit owner pocket into a true child module under `src/backend/storage_security/credential_vault/implementation/vault_persistence_restore/` only if `vault_persistence_restore.rs` remains the parent child that mediates `save_inner` and existing `CredentialVault` behavior remains equivalent.

## Equivalence Baseline

| Surface | Frozen behavior |
| --- | --- |
| Entry input | `save_inner` receives the selected credential file path, current 32-byte machine key, and parent-owned `VaultData`. |
| Parent directory | If the credential path has a parent, the parent directory is created before serialization/write. |
| Serialization and encryption | `VaultData` is serialized with `serde_json::to_string` and encrypted through `encrypt_with_machine_key`; crypto internals remain owned by `crypto_codec`. |
| Temp/backup paths | Tmp path remains `path.with_extension("tmp")`; backup path remains `path.with_extension("bak")`. |
| Existing primary backup | If the primary exists, stale backup is best-effort removed, then primary is renamed to backup before tmp write. |
| Tmp write failure | Tmp write failure attempts to restore backup to primary when an old primary existed, then returns `凭证写入失败: {}`. |
| Tmp fsync | Opening tmp and `sync_all()` remain best-effort and do not alter success/failure shape. |
| Rename failure | Rename failure attempts to restore backup to primary when an old primary existed, removes tmp best-effort, then returns `凭证保存失败: {}`. |
| Parent fsync | Opening parent directory and `sync_all()` remain best-effort after successful rename. |
| Cleanup | Successful save removes backup best-effort. |
| Permission hardening | Unix keeps best-effort `0o600`; Windows keeps best-effort `icacls /inheritance:r /grant USERNAME:F`. |
| Parent mediation | `vault_persistence_restore.rs` remains the parent child and exports only `pub(super)` helpers back to `implementation.rs`. |

## Extraction Plan

1. Create `src/backend/storage_security/credential_vault/implementation/vault_persistence_restore/atomic_save_commit.rs`.
2. Move the body of `save_inner` into a `pub(super)` helper owned by that child.
3. Keep `vault_persistence_restore.rs` as the parent child: it declares `mod atomic_save_commit;` and delegates `save_inner` to the new child.
4. Keep `load_restore_entry.rs` closed and unchanged.
5. Keep `CredentialVault`, `VaultData`, public CRUD methods, secret pattern extraction, machine-key child, crypto child, root shim, and release transition unchanged.
6. Run `cargo fmt --check`, `cargo check -p quantpilot`, `cargo test -p quantpilot credential_vault --lib`, and `cargo test -p quantpilot credential --lib`.

## Boundary

**Real files**:
- `src/backend/storage_security/credential_vault/implementation/vault_persistence_restore.rs`
- `src/backend/storage_security/credential_vault/implementation/vault_persistence_restore/load_restore_entry.rs`
- `src/backend/storage_security/credential_vault/implementation.rs`
- `src/backend/storage_security/credential_vault/implementation/crypto_codec.rs`

**Markers**:
- `BE-001JX-01`
- `atomic_save_commit baseline_frozen`
- `atomic_save_commit plan_frozen`
- `load_restore_entry remains_closed`
- `no code movement`
- `release_transition_guard`

**Next step**:
BE-001JX-02 backend.storage_security.credential_vault_implementation.vault_persistence_restore.atomic_save_commit extract_closeout

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
