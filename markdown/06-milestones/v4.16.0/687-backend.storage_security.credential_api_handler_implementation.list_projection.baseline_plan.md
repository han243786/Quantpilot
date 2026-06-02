# v4.16.0 backend.storage_security.credential_api_handler_implementation.list_projection equivalence baseline and extraction plan

> Batch: BE-001KQ-01
> Node: `backend.storage_security.credential_api_handler_implementation.list_projection`
> Parent: `backend.storage_security.credential_api_handler_implementation`
> Stage: `baseline_plan`
> Movement: No code movement.

---

## Summary

`backend.storage_security.credential_api_handler_implementation.list_projection` equivalence baseline and extraction plan are frozen.

This child owns only:

- `unscoped_services_for`
- `list_credentials`
- `GET /api/credentials`

Frozen behavior:

- `unscoped_services_for` must build the prefix with `format!("{}:", user_id.0)`.
- It must list all vault services, keep only keys starting with that prefix, strip exactly `prefix.len()` bytes, and collect the resulting service labels.
- `list_credentials` must accept `auth::UserId` and `State<AppState>`.
- When `state.credential_vault` is `Some`, it must return `Json(serde_json::json!({ "services": services }))`.
- When the vault is absent, it must return `StatusCode::SERVICE_UNAVAILABLE` with the existing message.

Allowed BE-001KQ-02 movement:

1. Create `src/backend/storage_security/credential_api_handler_implementation/list_projection.rs`.
2. Move `unscoped_services_for` and `list_credentials` into that child.
3. Add `mod list_projection;` in `credential_api_handler_implementation.rs`.
4. Update route registration so `GET /api/credentials` uses `list_projection::list_credentials`.

Forbidden BE-001KQ-02 movement:

- Do not move `scoped_cv_key`.
- Do not move or edit `set_credential`.
- Do not move or edit `delete_credential`.
- Do not change route paths, HTTP methods, validation, status codes, JSON response shape, auth extraction, vault internals, audit logging, or release-transition policy.

## Boundary

**Real files**:
- `src/backend/storage_security/credential_api_handler_implementation.rs`
- `src/backend/storage_security/credential_api_handler_implementation/list_projection.rs` (planned)

**Markers**:
- `BE-001KQ-01`
- `baseline_frozen`
- `list_projection`
- `get_credentials_branch`
- `set_delete_deferred`
- `release_transition_guard`

**Next step**:
BE-001KQ-02 backend.storage_security.credential_api_handler_implementation.list_projection extract_closeout

---

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p quantpilot credential --lib`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
