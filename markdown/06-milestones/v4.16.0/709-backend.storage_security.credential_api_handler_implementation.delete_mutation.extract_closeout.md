# v4.16.0 backend.storage_security.credential_api_handler_implementation.delete_mutation actual extraction complete

> Batch: BE-001LB-02
> Node: `backend.storage_security.credential_api_handler_implementation.delete_mutation`
> Parent: `backend.storage_security.credential_api_handler_implementation`
> Stage: `extract_closeout`
> Movement: Code movement.

---

## Summary

BE-001LB-02 completes actual extraction for `backend.storage_security.credential_api_handler_implementation.delete_mutation`.

Moved into `src/backend/storage_security/credential_api_handler_implementation/delete_mutation.rs`:

- DELETE service path validation
- vault unavailable mapping
- parent key bridge handoff through `super::scoped_cv_key`
- vault `delete_service`
- not-found/internal delete error mapping
- delete audit log
- `{"deleted": service}` success response

Parent updates:

- `src/backend/storage_security/credential_api_handler_implementation.rs` now declares `mod delete_mutation`.
- DELETE route registration now uses `delete_mutation::delete_credential`.
- route registration ownership and the parent `scoped_cv_key` bridge remain in the handler implementation parent.

Deferred residuals:

- route registration remains in the parent.
- `scoped_cv_key` bridge remains in the parent.
- list/set/key child internals are unchanged.
- auth/vault internals and release-transition behavior are unchanged.

## Boundary

**Real files**:
- `src/backend/storage_security/credential_api_handler_implementation.rs`
- `src/backend/storage_security/credential_api_handler_implementation/delete_mutation.rs`

**Markers**:
- `BE-001LB-02`
- `actual_extraction_complete`
- `delete_mutation`
- `delete_path_validation`
- `delete_parent_key_bridge_retained`
- `delete_not_found_mapping`
- `release_transition_guard`

**Next step**:
BE-001LB-03 backend.storage_security.credential_api_handler_implementation.delete_mutation single_leaf_closeout

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
