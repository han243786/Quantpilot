# v4.16.0 backend.storage_security.credential_api_handler_implementation.set_mutation equivalence baseline and extraction plan

> Batch: BE-001KU-01
> Node: `backend.storage_security.credential_api_handler_implementation.set_mutation`
> Parent: `backend.storage_security.credential_api_handler_implementation`
> Stage: `baseline_plan`
> Movement: No code movement.

---

## Summary

`backend.storage_security.credential_api_handler_implementation.set_mutation` equivalence baseline and extraction plan are frozen.

This child owns only:

- `set_credential`
- `POST /api/credentials`

Frozen behavior:

- Accepts `auth::UserId`, `State<AppState>`, and `Json(serde_json::Value)`.
- Reads `state.credential_vault` first and returns `StatusCode::SERVICE_UNAVAILABLE` with the existing message when absent.
- Reads `body["service"]` as a string.
- Rejects service labels when `trim().is_empty()`, length is greater than 64, contains `/`, contains `\`, or contains `..`.
- Keeps the original service string after validation; it must not trim or normalize before storage or response.
- Reads `body["fields"]` as an object and returns `StatusCode::BAD_REQUEST` with the existing message when absent.
- Converts each field value with `v.as_str().unwrap_or_default().to_string()`.
- Rejects empty converted values with `StatusCode::BAD_REQUEST` and the existing per-field message.
- Inserts cloned field names into `BTreeMap<String, String>`.
- Calls the parent `scoped_cv_key` bridge before vault mutation.
- Calls `vault.set_service(&scoped_key, fields)`.
- Maps vault storage errors to `StatusCode::INTERNAL_SERVER_ERROR` with the existing message.
- Emits the existing audit log text.
- Returns `Json(serde_json::json!({ "stored": service }))` on success.

Allowed BE-001KU-02 movement:

1. Create `src/backend/storage_security/credential_api_handler_implementation/set_mutation.rs`.
2. Move `set_credential` into that child.
3. Add `mod set_mutation;` in `credential_api_handler_implementation.rs`.
4. Update route registration so `POST /api/credentials` uses `set_mutation::set_credential`.
5. Let the child call `super::scoped_cv_key`; do not call `key_scope` sibling directly.

Forbidden BE-001KU-02 movement:

- Do not move `delete_credential`.
- Do not move route registration ownership.
- Do not move or alter `key_scope` or `list_projection` child internals.
- Do not alter validation predicates, conversion rules, vault call order, audit logging, status-code mapping, JSON response shape, auth/vault internals, or release-transition policy.

## Boundary

**Real files**:
- `src/backend/storage_security/credential_api_handler_implementation.rs`
- `src/backend/storage_security/credential_api_handler_implementation/set_mutation.rs` (planned)

**Markers**:
- `BE-001KU-01`
- `baseline_frozen`
- `set_mutation`
- `post_credentials_branch`
- `parent_key_bridge_required`
- `delete_mutation_deferred`
- `release_transition_guard`

**Next step**:
BE-001KU-02 backend.storage_security.credential_api_handler_implementation.set_mutation extract_closeout

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
