# v4.16.0 backend.storage_security.credential_api_handler_implementation.delete_mutation.service_path_validation equivalence baseline and extraction plan

> Batch: BE-001LD-01
> Node: `backend.storage_security.credential_api_handler_implementation.delete_mutation.service_path_validation`
> Parent: `backend.storage_security.credential_api_handler_implementation.delete_mutation`
> Stage: `baseline_plan`
> Movement: No code movement.

---

## Summary

`backend.storage_security.credential_api_handler_implementation.delete_mutation.service_path_validation` equivalence baseline and extraction plan are frozen.

This child owns only the DELETE path service label validation gate.

Frozen behavior:

- Accept the original `service: String` from `Path(service): Path<String>`.
- Reject when `service.is_empty()`.
- Reject when `service.len() > 64`.
- Reject when `service.contains('/')`.
- Reject when `service.contains('\\')`.
- Reject when `service.contains("..")`.
- Reject when `service.contains('\0')`.
- Preserve the original valid service string; do not trim, normalize, lowercase, encode, or sanitize it.
- On invalid service, return `StatusCode::BAD_REQUEST` with `凭证标签无效`.

Allowed BE-001LD-02 movement:

1. Create `src/backend/storage_security/credential_api_handler_implementation/delete_mutation/service_path_validation.rs`.
2. Move only the DELETE service path validation condition and invalid-label error into that child.
3. Add `mod service_path_validation;` in `delete_mutation.rs`.
4. Let `delete_mutation.rs` call the child immediately after `Path(service)` extraction.
5. Let the child return the original valid `String` for the parent delete flow.

Forbidden BE-001LD-02 movement:

- Do not move vault availability lookup.
- Do not move parent key bridge handoff.
- Do not move `vault.delete_service`.
- Do not move not-found/internal error mapping, audit logging, or success response.
- Do not move route registration, list/set/key child internals, auth/vault internals, or release-transition policy.

## Boundary

**Real files**:
- `src/backend/storage_security/credential_api_handler_implementation/delete_mutation.rs`
- `src/backend/storage_security/credential_api_handler_implementation/delete_mutation/service_path_validation.rs` (planned)

**Markers**:
- `BE-001LD-01`
- `baseline_frozen`
- `service_path_validation`
- `delete_path_service_gate`
- `invalid_label_bad_request`
- `release_transition_guard`

**Next step**:
BE-001LD-02 backend.storage_security.credential_api_handler_implementation.delete_mutation.service_path_validation extract_closeout

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
