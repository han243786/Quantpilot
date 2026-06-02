# v4.16.0 backend.storage_security.credential_api_handler_implementation.delete_mutation.delete_commit equivalence baseline and extraction plan

> Batch: BE-001LF-01
> Node: `backend.storage_security.credential_api_handler_implementation.delete_mutation.delete_commit`
> Parent: `backend.storage_security.credential_api_handler_implementation.delete_mutation`
> Stage: `baseline_plan`
> Movement: No code movement.

---

## Summary

`backend.storage_security.credential_api_handler_implementation.delete_mutation.delete_commit` equivalence baseline and extraction plan are frozen.

This child owns only the DELETE storage commit and result phase after validation and key scoping.

Frozen behavior:

- Call `vault.delete_service(&scoped_key)`.
- If delete error text contains `不存在`, map to `StatusCode::NOT_FOUND`.
- Preserve the existing not-found message format: `标签 '{service}' 不存在`.
- Map all other delete errors to `StatusCode::INTERNAL_SERVER_ERROR`.
- Preserve the existing internal error message format: `凭证删除失败: {error}`.
- Emit the existing audit log only after `delete_service` succeeds.
- Preserve the existing audit log shape: `[audit] 用户 {user_id} 删除凭证 service={service}`.
- Return `Ok(Json(serde_json::json!({ "deleted": service })))` on success.
- Do not audit failed delete commits.
- Do not add new fields to the success JSON.

Allowed BE-001LF-02 movement:

1. Create `src/backend/storage_security/credential_api_handler_implementation/delete_mutation/delete_commit.rs`.
2. Move vault `delete_service`, delete error mapping, audit logging, and success JSON construction into that child.
3. Add `mod delete_commit;` in `delete_mutation.rs`.
4. Let `delete_mutation.rs` call the child after validation, vault availability lookup, and parent key bridge handoff.
5. Pass only the existing delete inputs/results needed by the child: vault reference, user id, scoped key, and service.

Forbidden BE-001LF-02 movement:

- Do not move vault availability lookup.
- Do not move service path validation.
- Do not move parent `scoped_cv_key` bridge handoff.
- Do not move route registration.
- Do not move list/set/key child internals, auth/vault internals, or release-transition policy.
- Do not add sibling shortcuts between validation and commit children.

## Boundary

**Real files**:
- `src/backend/storage_security/credential_api_handler_implementation/delete_mutation.rs`
- `src/backend/storage_security/credential_api_handler_implementation/delete_mutation/delete_commit.rs` (planned)

**Markers**:
- `BE-001LF-01`
- `baseline_frozen`
- `delete_commit`
- `vault_delete_service_phase`
- `delete_not_found_mapping`
- `delete_audit_after_success_only`
- `success_response_shape_frozen`
- `release_transition_guard`

**Next step**:
BE-001LF-02 backend.storage_security.credential_api_handler_implementation.delete_mutation.delete_commit extract_closeout

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
