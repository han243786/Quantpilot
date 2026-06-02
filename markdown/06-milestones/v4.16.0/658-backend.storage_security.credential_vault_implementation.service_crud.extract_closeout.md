# v4.16.0 backend.storage_security.credential_vault_implementation.service_crud actual extraction complete

> Batch: BE-001KA-02
> Node: `backend.storage_security.credential_vault_implementation.service_crud`
> Parent: `backend.storage_security.credential_vault_implementation`
> Stage: `extract_closeout`
> Movement: Code movement.

---

## Summary

BE-001KA-02 completes the actual `service_crud` extraction.

- Added `src/backend/storage_security/credential_vault/implementation/service_crud.rs`.
- `implementation.rs` now declares `mod service_crud;` and keeps public `CredentialVault` methods as facade methods.
- The new child owns `set_service`, `get_service`, `delete_service`, and `list_services` helper bodies, including empty-field validation, insert/overwrite, zeroizing read wrapping, missing-delete error, list collection, and save handoff.
- Parent-owned types, load/persistence children, secret pattern extraction, tests, root shim, and release transition remain unmoved.

## Boundary

**Real files**:
- `src/backend/storage_security/credential_vault/implementation/service_crud.rs`
- `src/backend/storage_security/credential_vault/implementation.rs`
- `src/backend/storage_security/credential_vault/implementation/vault_persistence_restore.rs`
- `src/backend/storage_security/credential_vault.rs`
- `src/credential_vault.rs`

**Markers**:
- `BE-001KA-02`
- `service_crud extraction_complete`
- `public facade retained`
- `secret_pattern_extraction remains_residual`
- `release_transition_guard`

**Next step**:
BE-001KA-03 backend.storage_security.credential_vault_implementation.service_crud single_leaf_closeout

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
