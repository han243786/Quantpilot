# v4.16.0 backend.storage_security.credential_api_handler_implementation actual extraction complete

> Batch: BE-001KO-02
> Node: `backend.storage_security.credential_api_handler_implementation`
> Parent: `backend.storage_security`
> Stage: `extract_closeout`
> Movement: Code movement.

---

## Summary

BE-001KO-02 completes actual extraction for `backend.storage_security.credential_api_handler_implementation`.

Moved handler owner:

- from `src/credential_api.rs`
- to `src/backend/storage_security/credential_api_handler_implementation.rs`

Parent mediation added:

- `src/backend/storage_security.rs` now declares `mod credential_api_handler_implementation`.
- `src/backend/storage_security.rs` owns the private `register_credential_handler_routes` bridge.
- `src/backend/storage_security/credential_api.rs` keeps the route facade and calls the storage-security parent bridge.

Root private module cleanup:

- `src/lib.rs` no longer declares `mod credential_api`.
- `src/credential_api.rs` was removed after the handler owner moved.

Preserved safety semantics:

- Route paths and HTTP methods remain `GET/POST /api/credentials` and `DELETE /api/credentials/:service`.
- `{user_id}:{service}` scoped credential keys are unchanged.
- list/set/delete validation, vault calls, audit logging, status-code mapping, and JSON response shape were not intentionally changed.
- The route facade does not call a handler sibling directly; it calls the storage-security parent bridge.

Deferred residuals:

- Handler-internal list/set/delete branches are not split in this step.
- Auth internals, `AppState`, vault internals, safe-log internals, rate limiting, backup, and storage lifecycle remain outside this movement.
- No release-transition shortcut was introduced.

## Boundary

**Real files**:
- `src/backend/storage_security/credential_api_handler_implementation.rs`
- `src/backend/storage_security.rs`
- `src/backend/storage_security/credential_api.rs`
- `src/lib.rs`
- `src/credential_api.rs` (removed)

**Markers**:
- `BE-001KO-02`
- `actual_extraction_complete`
- `credential_api_handler_implementation`
- `parent_mediated_route_bridge`
- `root_credential_api_removed`
- `release_transition_guard`

**Next step**:
BE-001KO-03 backend.storage_security.credential_api_handler_implementation single_leaf_closeout

---

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p quantpilot credential --lib`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
