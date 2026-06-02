# v4.16.0 backend.storage_security.credential_api_handler_implementation.list_projection actual extraction complete

> Batch: BE-001KQ-02
> Node: `backend.storage_security.credential_api_handler_implementation.list_projection`
> Parent: `backend.storage_security.credential_api_handler_implementation`
> Stage: `extract_closeout`
> Movement: Code movement.

---

## Summary

BE-001KQ-02 completes actual extraction for `backend.storage_security.credential_api_handler_implementation.list_projection`.

Moved into `src/backend/storage_security/credential_api_handler_implementation/list_projection.rs`:

- `unscoped_services_for`
- `list_credentials`

Parent updates:

- `src/backend/storage_security/credential_api_handler_implementation.rs` now declares `mod list_projection`.
- `GET /api/credentials` route registration now uses `list_projection::list_credentials`.
- `set_credential`, `delete_credential`, `scoped_cv_key`, and route registration ownership remain in the parent file.

Preserved behavior:

- scoped prefix filtering and prefix stripping are unchanged.
- vault unavailable handling remains `503 SERVICE_UNAVAILABLE`.
- success response remains `{"services": services}`.
- no auth/vault internals, set/delete branches, audit logging, status-code mapping, or release-transition behavior moved.

## Boundary

**Real files**:
- `src/backend/storage_security/credential_api_handler_implementation/list_projection.rs`
- `src/backend/storage_security/credential_api_handler_implementation.rs`

**Markers**:
- `BE-001KQ-02`
- `actual_extraction_complete`
- `list_projection`
- `get_credentials_branch`
- `set_delete_deferred`
- `release_transition_guard`

**Next step**:
BE-001KQ-03 backend.storage_security.credential_api_handler_implementation.list_projection single_leaf_closeout

---

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p quantpilot credential --lib`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
