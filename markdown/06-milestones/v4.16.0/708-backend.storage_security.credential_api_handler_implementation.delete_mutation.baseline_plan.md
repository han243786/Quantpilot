# v4.16.0 backend.storage_security.credential_api_handler_implementation.delete_mutation equivalence baseline and extraction plan

> Batch: BE-001LB-01
> Node: `backend.storage_security.credential_api_handler_implementation.delete_mutation`
> Parent: `backend.storage_security.credential_api_handler_implementation`
> Stage: `baseline_plan`
> Movement: No code movement.

---

## Summary

`backend.storage_security.credential_api_handler_implementation.delete_mutation` equivalence baseline and extraction plan are frozen.

This child owns only the DELETE credential mutation branch.

Frozen behavior:

- Accept `Path(service): Path<String>` from `DELETE /api/credentials/:service`.
- Reject service labels when empty, length is greater than 64, contains `/`, contains `\`, contains `..`, or contains `\0`.
- Preserve the original service string; do not trim, normalize, lowercase, encode, or sanitize it.
- Map missing credential vault to `StatusCode::SERVICE_UNAVAILABLE` with the existing message.
- Call the parent `scoped_cv_key(&user_id, &service)` bridge before deletion.
- Call `vault.delete_service(&scoped_key)`.
- If delete error text contains `不存在`, map to `StatusCode::NOT_FOUND` with `标签 '{service}' 不存在`.
- Map all other delete errors to `StatusCode::INTERNAL_SERVER_ERROR` with `凭证删除失败: {error}`.
- Emit the existing audit log only after `delete_service` succeeds.
- Return `Ok(Json(serde_json::json!({ "deleted": service })))` on success.

Allowed BE-001LB-02 movement:

1. Create `src/backend/storage_security/credential_api_handler_implementation/delete_mutation.rs`.
2. Move `delete_credential` into that child.
3. Add `mod delete_mutation;` in the handler implementation parent.
4. Update DELETE route registration to use `delete_mutation::delete_credential`.
5. Let the delete child call `super::scoped_cv_key`, preserving the parent key bridge and avoiding sibling shortcuts.

Forbidden BE-001LB-02 movement:

- Do not move route registration ownership out of the handler implementation parent.
- Do not move the parent `scoped_cv_key` bridge function.
- Do not move list/set/key child internals.
- Do not move auth/vault internals.
- Do not change status codes, error messages, audit shape, or JSON response shape.
- Do not introduce release-transition behavior.

## Boundary

**Real files**:
- `src/backend/storage_security/credential_api_handler_implementation.rs`
- `src/backend/storage_security/credential_api_handler_implementation/delete_mutation.rs` (planned)

**Markers**:
- `BE-001LB-01`
- `baseline_frozen`
- `delete_mutation`
- `delete_path_validation`
- `delete_not_found_mapping`
- `delete_audit_after_success_only`
- `release_transition_guard`

**Next step**:
BE-001LB-02 backend.storage_security.credential_api_handler_implementation.delete_mutation extract_closeout

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
