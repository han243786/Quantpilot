# v4.16.0 backend.storage_security.credential_api_handler_implementation.delete_mutation.delete_commit actual extraction complete

> Batch: BE-001LF-02
> Node: `backend.storage_security.credential_api_handler_implementation.delete_mutation.delete_commit`
> Parent: `backend.storage_security.credential_api_handler_implementation.delete_mutation`
> Stage: `extract_closeout`
> Movement: Code movement.

---

## Summary

BE-001LF-02 completes actual extraction for `backend.storage_security.credential_api_handler_implementation.delete_mutation.delete_commit`.

Moved into `src/backend/storage_security/credential_api_handler_implementation/delete_mutation/delete_commit.rs`:

- vault `delete_service` call
- not-found/internal delete error mapping
- delete audit log after successful delete commit
- `{"deleted": service}` success JSON response

Parent updates:

- `src/backend/storage_security/credential_api_handler_implementation/delete_mutation.rs` now declares `mod delete_commit`.
- `delete_credential` keeps service path validation, vault availability lookup, and parent `scoped_cv_key` bridge handoff.
- `delete_credential` delegates the post-key delete result phase to `delete_commit::commit_delete_credential`.

Deferred residuals:

- vault availability lookup remains in `delete_mutation.rs`.
- service path validation remains in the validation child.
- parent key bridge handoff remains in `delete_mutation.rs`.
- route registration and parent key bridge remain in the handler implementation parent.
- list/set/key child internals, auth/vault internals, and release-transition behavior are unchanged.

## Boundary

**Real files**:
- `src/backend/storage_security/credential_api_handler_implementation/delete_mutation.rs`
- `src/backend/storage_security/credential_api_handler_implementation/delete_mutation/delete_commit.rs`

**Markers**:
- `BE-001LF-02`
- `actual_extraction_complete`
- `delete_commit`
- `vault_delete_service_phase`
- `delete_not_found_mapping`
- `delete_audit_after_success_only`
- `success_response_shape_retained`
- `release_transition_guard`

**Next step**:
BE-001LF-03 backend.storage_security.credential_api_handler_implementation.delete_mutation.delete_commit single_leaf_closeout

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
