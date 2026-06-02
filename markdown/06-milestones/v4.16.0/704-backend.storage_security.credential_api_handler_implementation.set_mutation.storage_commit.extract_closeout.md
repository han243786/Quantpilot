# v4.16.0 backend.storage_security.credential_api_handler_implementation.set_mutation.storage_commit actual extraction complete

> Batch: BE-001KY-02
> Node: `backend.storage_security.credential_api_handler_implementation.set_mutation.storage_commit`
> Parent: `backend.storage_security.credential_api_handler_implementation.set_mutation`
> Stage: `extract_closeout`
> Movement: Code movement.

---

## Summary

BE-001KY-02 completes actual extraction for `backend.storage_security.credential_api_handler_implementation.set_mutation.storage_commit`.

Moved into `src/backend/storage_security/credential_api_handler_implementation/set_mutation/storage_commit.rs`:

- vault `set_service` call
- storage error mapping to `StatusCode::INTERNAL_SERVER_ERROR`
- set audit log after successful storage commit
- `{"stored": service}` success JSON response

Parent updates:

- `src/backend/storage_security/credential_api_handler_implementation/set_mutation.rs` now declares `mod storage_commit`.
- `set_credential` keeps vault availability lookup, service/fields validation, and parent `scoped_cv_key` bridge handoff.
- `set_credential` delegates the post-key storage result phase to `storage_commit::commit_set_credential`.

Deferred residuals:

- vault availability lookup remains in the parent.
- service/fields validation remains in the validation child.
- parent key bridge handoff remains in the parent.
- delete mutation, route registration, list/key child internals, auth/vault internals, and release-transition behavior are unchanged.

## Boundary

**Real files**:
- `src/backend/storage_security/credential_api_handler_implementation/set_mutation.rs`
- `src/backend/storage_security/credential_api_handler_implementation/set_mutation/storage_commit.rs`

**Markers**:
- `BE-001KY-02`
- `actual_extraction_complete`
- `storage_commit`
- `vault_set_service_phase`
- `audit_after_success_only`
- `success_response_shape_retained`
- `release_transition_guard`

**Next step**:
BE-001KY-03 backend.storage_security.credential_api_handler_implementation.set_mutation.storage_commit single_leaf_closeout

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
