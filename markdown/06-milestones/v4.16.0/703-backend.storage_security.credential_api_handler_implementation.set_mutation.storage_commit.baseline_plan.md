# v4.16.0 backend.storage_security.credential_api_handler_implementation.set_mutation.storage_commit equivalence baseline and extraction plan

> Batch: BE-001KY-01
> Node: `backend.storage_security.credential_api_handler_implementation.set_mutation.storage_commit`
> Parent: `backend.storage_security.credential_api_handler_implementation.set_mutation`
> Stage: `baseline_plan`
> Movement: No code movement.

---

## Summary

`backend.storage_security.credential_api_handler_implementation.set_mutation.storage_commit` equivalence baseline and extraction plan are frozen.

This child owns only the POST set storage commit and result phase after validation and key scoping.

Frozen behavior:

- Call `vault.set_service(&scoped_key, fields)`.
- Map any storage error to `StatusCode::INTERNAL_SERVER_ERROR`.
- Preserve the existing storage error message format: `凭证存储失败: {error}`.
- Emit the existing audit log only after `set_service` succeeds.
- Preserve the existing audit log shape: `[audit] 用户 {user_id} 设置凭证 service={service}`.
- Return `Ok(Json(serde_json::json!({ "stored": service })))` on success.
- Do not add new fields to the success JSON.
- Do not audit failed storage commits.

Allowed BE-001KY-02 movement:

1. Create `src/backend/storage_security/credential_api_handler_implementation/set_mutation/storage_commit.rs`.
2. Move vault `set_service`, storage error mapping, audit logging, and success JSON construction into that child.
3. Add `mod storage_commit;` in `set_mutation.rs`.
4. Let `set_mutation.rs` call the child after validation and parent key bridge handoff.
5. Pass only the existing storage inputs/results needed by the child: vault reference, user id, scoped key, service, and fields.

Forbidden BE-001KY-02 movement:

- Do not move vault availability lookup.
- Do not move service/fields validation.
- Do not move parent `scoped_cv_key` bridge handoff.
- Do not move delete mutation.
- Do not move route registration, list/key child internals, auth/vault internals, or release-transition policy.
- Do not add sibling shortcuts between validation and storage children.

## Boundary

**Real files**:
- `src/backend/storage_security/credential_api_handler_implementation/set_mutation.rs`
- `src/backend/storage_security/credential_api_handler_implementation/set_mutation/storage_commit.rs` (planned)

**Markers**:
- `BE-001KY-01`
- `baseline_frozen`
- `storage_commit`
- `vault_set_service_phase`
- `audit_after_success_only`
- `success_response_shape_frozen`
- `release_transition_guard`

**Next step**:
BE-001KY-02 backend.storage_security.credential_api_handler_implementation.set_mutation.storage_commit extract_closeout

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
