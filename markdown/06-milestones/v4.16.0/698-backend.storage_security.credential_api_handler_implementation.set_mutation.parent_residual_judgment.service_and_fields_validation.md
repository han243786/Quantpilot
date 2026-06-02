# v4.16.0 backend.storage_security.credential_api_handler_implementation.set_mutation parent residual judgment selects service_and_fields_validation

> Batch: BE-001KV-01
> Node: `backend.storage_security.credential_api_handler_implementation.set_mutation`
> Parent: `backend.storage_security.credential_api_handler_implementation`
> Stage: `parent_residual_judgment`
> Movement: No code movement.

---

## Summary

`backend.storage_security.credential_api_handler_implementation.set_mutation.service_and_fields_validation` is selected as the next child.

This child owns only the input side of `POST /api/credentials`:

- service label extraction and validation
- fields object validation
- field value string conversion
- empty field rejection
- construction of `BTreeMap<String, String>`

It is selected before `storage_commit` because validation controls whether a request is allowed to reach credential storage at all. The remaining parent residual after this child is the commit/audit/response path:

- vault availability lookup
- parent key bridge handoff
- vault `set_service`
- storage error mapping
- audit log
- `{"stored": service}` response

## leaf_split_decision_gate

leaf_split_base_gate

| Rule | Result | Evidence |
| --- | --- | --- |
| white_box_boundary_named | PASS | The selected child is `backend.storage_security.credential_api_handler_implementation.set_mutation.service_and_fields_validation`. |
| parent_child_communication_kept | PASS | The child remains under `set_mutation`; storage commit remains parent residual. |
| equivalence_baseline_freezable | PASS | The validation rules are concrete and can be frozen before movement. |

leaf_split_positive_trigger

| Trigger | Result | Evidence |
| --- | --- | --- |
| public_or_handler_boundary | PARTIAL | The child is private but guards the public POST credential handler. |
| state_machine_phase | PASS | It owns the request validation/conversion phase before storage mutation. |
| strategy_branch | PASS | Service validation and fields conversion/rejection are distinct from vault commit. |
| independent_failure_mode | PASS | Validation regressions can allow unsafe labels or reject valid fields independently from storage/audit behavior. |
| reuse_pressure | PARTIAL | Reuse is secondary, but extraction improves review and future validation tests. |

leaf_split_stop_condition

| Stop condition | Result | Evidence |
| --- | --- | --- |
| micro_leaf_without_owner | NO | The selected child owns real request input validation and conversion. |
| communication_cost_rises | NO | It can return validated data to the parent without sibling shortcuts. |
| local_proof_missing | NO | BE-001KU-03 inherited passing `cargo check`, `key_scope`, and `credential` filtered proof. |
| line_count_only | NO | Selection is based on security validation ownership, not file length. |

leaf_split_decision_result

`select_child: backend.storage_security.credential_api_handler_implementation.set_mutation.service_and_fields_validation`

`backend.storage_security.credential_api_handler_implementation.set_mutation stop_split: false`.

BE-001KW-01 must freeze exact validation and conversion behavior before movement. It must not move vault commit, audit logging, success response mapping, parent key bridge, delete logic, route registration, or release-transition shortcuts.

next_recursive_step

BE-001KW-01 backend.storage_security.credential_api_handler_implementation.set_mutation.service_and_fields_validation baseline_plan
## Boundary

**Real files**:
- `src/backend/storage_security/credential_api_handler_implementation/set_mutation.rs`

**Markers**:
- `BE-001KV-01`
- `select_service_and_fields_validation`
- `post_credentials_validation_phase`
- `storage_commit_deferred`
- `release_transition_guard`

**Next step**:
BE-001KW-01 backend.storage_security.credential_api_handler_implementation.set_mutation.service_and_fields_validation baseline_plan

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
