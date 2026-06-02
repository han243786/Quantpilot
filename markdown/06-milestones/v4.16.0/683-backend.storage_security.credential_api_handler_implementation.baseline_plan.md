# v4.16.0 backend.storage_security.credential_api_handler_implementation safety equivalence baseline and extraction plan

> Batch: BE-001KO-01
> Node: `backend.storage_security.credential_api_handler_implementation`
> Parent: `backend.storage_security`
> Stage: `baseline_plan`
> Movement: No code movement.

---

## Summary

`backend.storage_security.credential_api_handler_implementation` safety equivalence baseline and extraction plan are frozen.

Current handler owner:

- `src/credential_api.rs`

Current route registration semantics:

- `GET /api/credentials` is handled by `list_credentials`.
- `POST /api/credentials` is handled by `set_credential`.
- `DELETE /api/credentials/:service` is handled by `delete_credential`.

Frozen handler semantics:

- `scoped_cv_key(user_id, service)` must keep the exact `{user_id}:{service}` key format.
- `unscoped_services_for` must filter by the `{user_id}:` prefix and strip only that prefix from returned service names.
- `list_credentials` must return `{"services": services}` when `state.credential_vault` exists and `503 SERVICE_UNAVAILABLE` when it does not.
- `set_credential` must read `body["service"]` as a string, reject trim-empty values, reject length greater than 64, reject `/`, `\`, and `..`, and keep the current behavior of using the original service string after validation.
- `set_credential` must read `body["fields"]` as an object, convert non-string values to empty strings through `unwrap_or_default`, reject empty field values with `400 BAD_REQUEST`, and pass the resulting `BTreeMap<String, String>` to the vault.
- `set_credential` must map vault storage errors to `500 INTERNAL_SERVER_ERROR`, emit the existing audit log call, and return `{"stored": service}` on success.
- `delete_credential` must reject empty service labels, labels longer than 64, `/`, `\`, `..`, and NUL, then delete the scoped key.
- `delete_credential` must preserve the current delete error mapping: not-found-like vault errors become `404 NOT_FOUND`; other vault errors become `500 INTERNAL_SERVER_ERROR`.
- `delete_credential` must emit the existing audit log call and return `{"deleted": service}` on success.

Allowed BE-001KO-02 movement:

1. Create `src/backend/storage_security/credential_api_handler_implementation.rs`.
2. Move the current handler implementation from `src/credential_api.rs` into that child.
3. Add a private parent bridge in `src/backend/storage_security.rs` that delegates to the handler implementation child.
4. Change `src/backend/storage_security/credential_api.rs` so the route facade calls the storage-security parent bridge, not the handler sibling directly.
5. Remove the now-empty root `mod credential_api` wiring if no caller remains.

Forbidden BE-001KO-02 movement:

- Do not change route paths, HTTP methods, status codes, response JSON shape, audit log text, validation predicates, user scoping, or vault call order.
- Do not move auth internals, `AppState`, credential vault internals, safe-log internals, rate limiting, backup, or storage lifecycle.
- Do not introduce credential_api facade to handler sibling direct calls.
- Do not introduce release-transition shortcuts.

## Boundary

**Real files**:
- `src/credential_api.rs`
- `src/backend/storage_security.rs`
- `src/backend/storage_security/credential_api.rs`
- `src/lib.rs`

**Markers**:
- `BE-001KO-01`
- `baseline_frozen`
- `credential_api_handler_implementation`
- `parent_mediated_route_bridge`
- `handler_safety_semantics`
- `release_transition_guard`

**Next step**:
BE-001KO-02 backend.storage_security.credential_api_handler_implementation extract_closeout

---

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p quantpilot credential --lib`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
