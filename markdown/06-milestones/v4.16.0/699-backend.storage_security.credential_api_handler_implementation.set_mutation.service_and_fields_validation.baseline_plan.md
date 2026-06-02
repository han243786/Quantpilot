# v4.16.0 backend.storage_security.credential_api_handler_implementation.set_mutation.service_and_fields_validation equivalence baseline and extraction plan

> Batch: BE-001KW-01
> Node: `backend.storage_security.credential_api_handler_implementation.set_mutation.service_and_fields_validation`
> Parent: `backend.storage_security.credential_api_handler_implementation.set_mutation`
> Stage: `baseline_plan`
> Movement: No code movement.

---

## Summary

`backend.storage_security.credential_api_handler_implementation.set_mutation.service_and_fields_validation` equivalence baseline and extraction plan are frozen.

This child owns only the POST set request input validation and conversion phase.

Frozen behavior:

- Read `body["service"]` as a string.
- Reject service labels when `trim().is_empty()`, length is greater than 64, contains `/`, contains `\`, or contains `..`.
- Preserve the original service string after validation; do not trim, normalize, lowercase, encode, or sanitize it.
- Read `body["fields"]` as an object.
- Reject missing/non-object fields with `StatusCode::BAD_REQUEST` and the existing message.
- Convert each field value with `v.as_str().unwrap_or_default().to_string()`.
- Reject empty converted values with `StatusCode::BAD_REQUEST` and the existing per-field message.
- Insert cloned field names into `BTreeMap<String, String>`.

Allowed BE-001KW-02 movement:

1. Create `src/backend/storage_security/credential_api_handler_implementation/set_mutation/service_and_fields_validation.rs`.
2. Move service validation and fields conversion into that child.
3. Add `mod service_and_fields_validation;` in `set_mutation.rs`.
4. Add or use a child function that returns the validated service string and fields map.
5. Keep vault availability lookup, parent key bridge handoff, `vault.set_service`, storage error mapping, audit log, and success JSON response in `set_mutation.rs`.

Forbidden BE-001KW-02 movement:

- Do not move vault availability lookup.
- Do not move parent `scoped_cv_key` bridge handoff.
- Do not move `vault.set_service`.
- Do not move storage error mapping, audit logging, or success response.
- Do not move delete mutation, route registration, list/key child internals, auth/vault internals, or release-transition policy.

## Boundary

**Real files**:
- `src/backend/storage_security/credential_api_handler_implementation/set_mutation.rs`
- `src/backend/storage_security/credential_api_handler_implementation/set_mutation/service_and_fields_validation.rs` (planned)

**Markers**:
- `BE-001KW-01`
- `baseline_frozen`
- `service_and_fields_validation`
- `post_validation_phase`
- `storage_commit_deferred`
- `release_transition_guard`

**Next step**:
BE-001KW-02 backend.storage_security.credential_api_handler_implementation.set_mutation.service_and_fields_validation extract_closeout

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
