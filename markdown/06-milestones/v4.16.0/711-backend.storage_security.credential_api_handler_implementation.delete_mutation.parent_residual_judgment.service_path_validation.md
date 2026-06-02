# v4.16.0 backend.storage_security.credential_api_handler_implementation.delete_mutation parent residual judgment selects service_path_validation

> Batch: BE-001LC-01
> Node: `backend.storage_security.credential_api_handler_implementation.delete_mutation`
> Parent: `backend.storage_security.credential_api_handler_implementation`
> Stage: `parent_residual_judgment`
> Movement: No code movement.

---

## Summary

`backend.storage_security.credential_api_handler_implementation.delete_mutation` residual judgment selects `backend.storage_security.credential_api_handler_implementation.delete_mutation.service_path_validation` as the next child.

Remaining parent-held phases:

- service path validation
- vault availability mapping
- parent key bridge handoff
- vault `delete_service`
- not-found/internal delete error mapping
- audit logging
- `{"deleted": service}` response

Selected next child:

- `backend.storage_security.credential_api_handler_implementation.delete_mutation.service_path_validation`

The selected child should own only the DELETE path `service` validation gate. Vault availability, parent key bridge handoff, delete commit, error mapping, audit, and response remain in `delete_mutation` for the next phase.

## Selection Rationale

| Candidate | Decision | Reason |
| --- | --- | --- |
| `service_path_validation` | SELECTED | It is the safety gate before key scoping and vault deletion; invalid service labels must be rejected before any storage action. |
| `delete_commit` | DEFERRED | It depends on the validated service and scoped key, so it should be split after the validation gate is isolated. |
| vault availability | KEEP_PARENT | It is a short `AppState` precondition and not yet a reusable policy owner. |

## Boundary

**Selected child owns**:
- service empty rejection
- service length >64 rejection
- `/`, `\`, `..`, and `\0` rejection
- preserving the original valid service string
- `StatusCode::BAD_REQUEST` and existing invalid-label message

**Parent keeps**:
- vault unavailable mapping
- parent key bridge handoff
- vault `delete_service`
- not-found/internal delete error mapping
- delete audit log
- `{"deleted": service}` response
- route registration
- list/set/key child internals
- auth/vault internals
- release-transition policy

**Next step**:
BE-001LD-01 backend.storage_security.credential_api_handler_implementation.delete_mutation.service_path_validation baseline_plan

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
