# v4.16.0 backend.storage_security.credential_api_handler_implementation.key_scope actual extraction complete

> Batch: BE-001KS-02
> Node: `backend.storage_security.credential_api_handler_implementation.key_scope`
> Parent: `backend.storage_security.credential_api_handler_implementation`
> Stage: `extract_closeout`
> Movement: Code movement.

---

## Summary

BE-001KS-02 completes actual extraction for `backend.storage_security.credential_api_handler_implementation.key_scope`.

Moved into `src/backend/storage_security/credential_api_handler_implementation/key_scope.rs`:

- exact `{user_id}:{service}` credential key formatting
- `scoped_cv_key(&UserId, &str)`
- a minimal unit test for `UserId(42)` + `binance` => `42:binance`

Parent updates:

- `src/backend/storage_security/credential_api_handler_implementation.rs` now declares `mod key_scope`.
- The parent keeps a local `scoped_cv_key` bridge that delegates to `key_scope::scoped_cv_key`.
- `set_credential` and `delete_credential` continue to call the parent bridge, not the key_scope child directly.

Preserved behavior:

- The key format remains `format!("{}:{}", user_id.0, service)`.
- No trimming, normalization, sanitization, lowercasing, encoding, validation, vault call, audit logging, status-code mapping, JSON response shape, set/delete movement, list projection movement, or release-transition behavior changed.

## Boundary

**Real files**:
- `src/backend/storage_security/credential_api_handler_implementation/key_scope.rs`
- `src/backend/storage_security/credential_api_handler_implementation.rs`

**Markers**:
- `BE-001KS-02`
- `actual_extraction_complete`
- `key_scope`
- `parent_bridge_retained`
- `key_scope_format_test`
- `release_transition_guard`

**Next step**:
BE-001KS-03 backend.storage_security.credential_api_handler_implementation.key_scope single_leaf_closeout

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
