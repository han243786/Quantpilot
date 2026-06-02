# v4.16.0 backend.storage_security.credential_api_handler_implementation.delete_mutation.service_path_validation actual extraction complete

> Batch: BE-001LD-02
> Node: `backend.storage_security.credential_api_handler_implementation.delete_mutation.service_path_validation`
> Parent: `backend.storage_security.credential_api_handler_implementation.delete_mutation`
> Stage: `extract_closeout`
> Movement: Code movement.

---

## Summary

BE-001LD-02 completes actual extraction for `backend.storage_security.credential_api_handler_implementation.delete_mutation.service_path_validation`.

Moved into `src/backend/storage_security/credential_api_handler_implementation/delete_mutation/service_path_validation.rs`:

- DELETE service empty rejection
- length >64 rejection
- `/`, `\`, `..`, and `\0` rejection
- `StatusCode::BAD_REQUEST` invalid-label response
- original valid service string handoff

Parent updates:

- `src/backend/storage_security/credential_api_handler_implementation/delete_mutation.rs` now declares `mod service_path_validation`.
- `delete_credential` calls `service_path_validation::validate_service_path(service)` immediately after path extraction.
- vault availability, parent key bridge handoff, `vault.delete_service`, delete error mapping, audit logging, and success response remain in `delete_mutation.rs`.

Deferred residuals:

- delete commit/error/audit/response remain in the parent delete mutation.
- route registration and parent key bridge remain in the handler implementation parent.
- list/set/key child internals, auth/vault internals, and release-transition behavior are unchanged.

## Boundary

**Real files**:
- `src/backend/storage_security/credential_api_handler_implementation/delete_mutation.rs`
- `src/backend/storage_security/credential_api_handler_implementation/delete_mutation/service_path_validation.rs`

**Markers**:
- `BE-001LD-02`
- `actual_extraction_complete`
- `service_path_validation`
- `delete_path_service_gate`
- `invalid_label_bad_request`
- `delete_commit_deferred`
- `release_transition_guard`

**Next step**:
BE-001LD-03 backend.storage_security.credential_api_handler_implementation.delete_mutation.service_path_validation single_leaf_closeout

---

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p quantpilot key_scope --lib`
- `cargo test -p quantpilot credential --lib`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
