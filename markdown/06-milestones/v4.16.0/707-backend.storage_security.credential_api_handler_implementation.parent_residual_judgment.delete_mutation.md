# v4.16.0 backend.storage_security.credential_api_handler_implementation parent residual judgment selects delete_mutation

> Batch: BE-001LA-01
> Node: `backend.storage_security.credential_api_handler_implementation`
> Parent: `backend.storage_security`
> Stage: `parent_residual_judgment`
> Movement: No code movement.

---

## Summary

`backend.storage_security.credential_api_handler_implementation` residual judgment selects `backend.storage_security.credential_api_handler_implementation.delete_mutation` as the next child.

Closed children:

- `backend.storage_security.credential_api_handler_implementation.list_projection`
- `backend.storage_security.credential_api_handler_implementation.key_scope`
- `backend.storage_security.credential_api_handler_implementation.set_mutation`

Remaining parent-held behavior:

- route registration for `/api/credentials`
- parent `scoped_cv_key` bridge
- `DELETE /api/credentials/:service` mutation branch

Selected next child:

- `backend.storage_security.credential_api_handler_implementation.delete_mutation`

The selected child should own the DELETE request lifecycle: service path validation, vault availability mapping, parent key bridge handoff, vault `delete_service`, not-found/internal error mapping, audit logging, and `{"deleted": service}` response. Route registration and the parent `scoped_cv_key` bridge must remain in the handler implementation parent.

## Selection Rationale

| Candidate | Decision | Reason |
| --- | --- | --- |
| `delete_mutation` | SELECTED | It is the last real credential CRUD mutation branch still owned by the handler parent. |
| route registration | DEFERRED | It is parent orchestration for GET/POST/DELETE and should not become a child while CRUD branches are being isolated. |
| `scoped_cv_key` bridge | KEEP_PARENT | The parent bridge prevents set/delete children from shortcutting to the `key_scope` sibling. |

## Boundary

**Selected child owns**:
- DELETE service path validation
- vault unavailable mapping
- parent key bridge handoff
- vault `delete_service`
- not-found/internal delete error mapping
- delete audit log
- `{"deleted": service}` response

**Parent keeps**:
- route registration
- `scoped_cv_key` bridge function
- list/set/delete child wiring
- auth/vault internals
- release-transition policy

**Next step**:
BE-001LB-01 backend.storage_security.credential_api_handler_implementation.delete_mutation baseline_plan

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
