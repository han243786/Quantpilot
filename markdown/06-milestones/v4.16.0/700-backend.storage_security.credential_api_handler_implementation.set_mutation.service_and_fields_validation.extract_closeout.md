# v4.16.0 backend.storage_security.credential_api_handler_implementation.set_mutation.service_and_fields_validation actual extraction complete

> Batch: BE-001KW-02
> Node: `backend.storage_security.credential_api_handler_implementation.set_mutation.service_and_fields_validation`
> Parent: `backend.storage_security.credential_api_handler_implementation.set_mutation`
> Stage: `extract_closeout`
> Movement: Code movement.

---

## Summary

BE-001KW-02 completes actual extraction for `backend.storage_security.credential_api_handler_implementation.set_mutation.service_and_fields_validation`.

Moved into `src/backend/storage_security/credential_api_handler_implementation/set_mutation/service_and_fields_validation.rs`:

- POST `service` extraction and validation.
- POST `fields` object validation.
- field value conversion via `as_str().unwrap_or_default().to_string()`.
- empty converted field rejection.
- `BTreeMap<String, String>` construction for storage handoff.

Parent updates:

- `src/backend/storage_security/credential_api_handler_implementation/set_mutation.rs` now declares `mod service_and_fields_validation`.
- `set_credential` delegates request validation/conversion to `service_and_fields_validation::validate_set_request`.
- The parent set mutation still owns vault availability lookup, parent `scoped_cv_key` bridge handoff, vault `set_service`, storage error mapping, audit logging, and `{"stored": service}` success response.

Deferred residuals:

- vault availability lookup remains in `set_mutation.rs`.
- parent key bridge handoff remains in `set_mutation.rs`.
- storage commit, storage error mapping, audit logging, and success response remain in `set_mutation.rs`.
- delete mutation, route registration, list/key child internals, auth/vault internals, and release-transition behavior are unchanged.

## Boundary

**Real files**:
- `src/backend/storage_security/credential_api_handler_implementation/set_mutation.rs`
- `src/backend/storage_security/credential_api_handler_implementation/set_mutation/service_and_fields_validation.rs`

**Markers**:
- `BE-001KW-02`
- `actual_extraction_complete`
- `service_and_fields_validation`
- `post_validation_phase`
- `parent_storage_commit_retained`
- `parent_key_bridge_retained`
- `release_transition_guard`

**Next step**:
BE-001KW-03 backend.storage_security.credential_api_handler_implementation.set_mutation.service_and_fields_validation single_leaf_closeout

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
