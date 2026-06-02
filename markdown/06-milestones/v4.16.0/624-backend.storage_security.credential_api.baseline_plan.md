# v4.16.0 backend.storage_security.credential_api route facade baseline and plan

> Batch: BE-001JG-01
> Node: `backend.storage_security.credential_api`
> Parent: `backend.storage_security`
> Stage: `baseline_plan`
> Movement: no code movement.

---

## Summary

backend.storage_security.credential_api route facade baseline and plan

Frozen current boundary:

- `src/backend/storage_security/credential_api.rs` owns only `MODULE_ID` and `register_routes`.
- `register_routes(router)` delegates to `crate::credential_api::register_credential_routes(router)`.
- `src/credential_api.rs` remains the handler owner for list, set, delete, user scoping, validation, audit logging, and response semantics.

Allowed next movement:

- Confirm the existing backend child facade as extraction complete.
- Keep root credential handlers in place.

Forbidden next movement:

- Do not move `list_credentials`, `set_credential`, `delete_credential`, `scoped_cv_key`, or `unscoped_services_for`.
- Do not change service validation, user scoping, audit logging, status codes, vault calls, or JSON response shape.
- Do not alter `CredentialVault`, auth extraction, AppState, or route paths.

## Boundary

**Real files**:
- `src/backend/storage_security/credential_api.rs`
- `src/backend/storage_security.rs`
- `src/credential_api.rs`

**Markers**:
- `BE-001JG-01`
- `baseline_frozen`
- `route facade only`
- `credential handler paused`
- `release_transition_guard`

**Next step**:
BE-001JG-02 backend.storage_security.credential_api extract_closeout

---

## Gates

- `git diff --check`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
