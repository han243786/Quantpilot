# v4.16.0 backend.storage_security.credential_api_handler_implementation.key_scope equivalence baseline and extraction plan

> Batch: BE-001KS-01
> Node: `backend.storage_security.credential_api_handler_implementation.key_scope`
> Parent: `backend.storage_security.credential_api_handler_implementation`
> Stage: `baseline_plan`
> Movement: No code movement.

---

## Summary

`backend.storage_security.credential_api_handler_implementation.key_scope` equivalence baseline and extraction plan are frozen.

This child owns only:

- `scoped_cv_key(user_id, service)`
- the exact credential vault key format for credential API mutations

Frozen behavior:

- `scoped_cv_key` must accept `&UserId` and `&str`.
- It must return `format!("{}:{}", user_id.0, service)`.
- It must not trim, normalize, sanitize, lowercase, encode, or otherwise transform `service`.
- It must preserve negative/zero/positive numeric user ids according to `user_id.0` formatting.
- It is shared by set and delete mutation branches through the handler implementation parent.

Allowed BE-001KS-02 movement:

1. Create `src/backend/storage_security/credential_api_handler_implementation/key_scope.rs`.
2. Move `scoped_cv_key` into that child.
3. Add `mod key_scope;` in `credential_api_handler_implementation.rs`.
4. Keep a parent-local `fn scoped_cv_key(...)` bridge that delegates to `key_scope::scoped_cv_key(...)`, so future set/delete children can use the parent bridge without sibling shortcuts.
5. Add a minimal unit test proving `UserId(42)` and `binance` become `42:binance`.

Forbidden BE-001KS-02 movement:

- Do not move `set_credential`.
- Do not move `delete_credential`.
- Do not move route registration.
- Do not alter validation, vault calls, audit logging, status codes, JSON response shape, auth internals, vault internals, list projection, or release-transition policy.

## Boundary

**Real files**:
- `src/backend/storage_security/credential_api_handler_implementation.rs`
- `src/backend/storage_security/credential_api_handler_implementation/key_scope.rs` (planned)

**Markers**:
- `BE-001KS-01`
- `baseline_frozen`
- `key_scope`
- `shared_user_service_key`
- `parent_bridge_required`
- `release_transition_guard`

**Next step**:
BE-001KS-02 backend.storage_security.credential_api_handler_implementation.key_scope extract_closeout

---

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p quantpilot credential --lib`
- `cargo test -p quantpilot key_scope --lib`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
