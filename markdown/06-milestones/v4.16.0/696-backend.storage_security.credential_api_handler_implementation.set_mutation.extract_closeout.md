# v4.16.0 backend.storage_security.credential_api_handler_implementation.set_mutation actual extraction complete

> Batch: BE-001KU-02
> Node: `backend.storage_security.credential_api_handler_implementation.set_mutation`
> Parent: `backend.storage_security.credential_api_handler_implementation`
> Stage: `extract_closeout`
> Movement: Code movement.

---

## Summary

BE-001KU-02 completes actual extraction for `backend.storage_security.credential_api_handler_implementation.set_mutation`.

Moved into `src/backend/storage_security/credential_api_handler_implementation/set_mutation.rs`:

- `set_credential`
- POST credential service validation
- POST fields conversion and empty rejection
- vault `set_service` call
- set audit log
- `{"stored": service}` response mapping

Parent updates:

- `src/backend/storage_security/credential_api_handler_implementation.rs` now declares `mod set_mutation`.
- `POST /api/credentials` route registration now uses `set_mutation::set_credential`.
- The set child calls `super::scoped_cv_key`, preserving the parent key bridge.

Deferred residuals:

- `delete_credential` remains in the parent.
- route registration ownership remains in the parent.
- `key_scope` and `list_projection` child internals are unchanged.
- No auth/vault internals, status/JSON/audit semantics, or release-transition behavior moved.

## Boundary

**Real files**:
- `src/backend/storage_security/credential_api_handler_implementation/set_mutation.rs`
- `src/backend/storage_security/credential_api_handler_implementation.rs`

**Markers**:
- `BE-001KU-02`
- `actual_extraction_complete`
- `set_mutation`
- `post_credentials_branch`
- `parent_key_bridge_retained`
- `delete_mutation_deferred`
- `release_transition_guard`

**Next step**:
BE-001KU-03 backend.storage_security.credential_api_handler_implementation.set_mutation single_leaf_closeout

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
