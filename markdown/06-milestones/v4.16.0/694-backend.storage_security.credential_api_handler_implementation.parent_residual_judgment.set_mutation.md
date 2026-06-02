# v4.16.0 backend.storage_security.credential_api_handler_implementation parent residual judgment selects set_mutation

> Batch: BE-001KT-01
> Node: `backend.storage_security.credential_api_handler_implementation`
> Parent: `backend.storage_security`
> Stage: `parent_residual_judgment`
> Movement: No code movement.

---

## Summary

`backend.storage_security.credential_api_handler_implementation.set_mutation` is selected as the next child.

Closed children:

- `list_projection stop_split: true`
- `key_scope stop_split: true`

Remaining parent responsibilities:

- route registration
- parent `scoped_cv_key` bridge
- `set_credential`
- `delete_credential`

`set_mutation` is selected before `delete_mutation` because it owns the larger POST mutation branch:

- vault availability mapping
- service label validation
- fields object validation
- field value conversion and empty rejection
- scoped key handoff through the parent bridge
- vault `set_service`
- audit log
- `{"stored": service}` success response

`delete_mutation` remains deferred.

## leaf_split_decision_gate

leaf_split_base_gate

| Rule | Result | Evidence |
| --- | --- | --- |
| white_box_boundary_named | PASS | The selected child is `backend.storage_security.credential_api_handler_implementation.set_mutation`. |
| parent_child_communication_kept | PASS | The child will remain under the handler implementation parent and call the parent key-scope bridge. |
| equivalence_baseline_freezable | PASS | The POST branch has a concrete validation/storage/audit/response boundary. |

leaf_split_positive_trigger

| Trigger | Result | Evidence |
| --- | --- | --- |
| public_or_handler_boundary | PASS | The child owns the `POST /api/credentials` handler branch. |
| state_machine_phase | PASS | It runs vault availability, service validation, fields conversion, scoped key, vault mutation, audit, and JSON response phases. |
| strategy_branch | PASS | Set/create credential mutation is separate from delete mutation. |
| independent_failure_mode | PASS | Service and field validation/storage regress independently from delete not-found mapping. |
| reuse_pressure | PARTIAL | Reuse is limited, but extraction improves targeted proof and future route tests. |

leaf_split_stop_condition

| Stop condition | Result | Evidence |
| --- | --- | --- |
| micro_leaf_without_owner | NO | The selected child owns a real POST mutation branch. |
| communication_cost_rises | NO | Parent-mediated key bridge prevents sibling shortcut cost. |
| local_proof_missing | NO | BE-001KS-03 inherited passing key_scope and credential filtered proof. |
| line_count_only | NO | Selection is driven by handler branch ownership, not file size. |

leaf_split_decision_result

`select_child: backend.storage_security.credential_api_handler_implementation.set_mutation`

`backend.storage_security.credential_api_handler_implementation stop_split: false`.

BE-001KU-01 must freeze the POST branch before any movement. It must not move delete logic, list projection, key_scope child internals, route registration ownership, auth/vault internals, or release-transition shortcuts.

next_recursive_step

BE-001KU-01 backend.storage_security.credential_api_handler_implementation.set_mutation baseline_plan
## Boundary

**Real files**:
- `src/backend/storage_security/credential_api_handler_implementation.rs`

**Markers**:
- `BE-001KT-01`
- `select_set_mutation`
- `post_credentials_branch`
- `delete_mutation_deferred`
- `parent_key_bridge_required`
- `release_transition_guard`

**Next step**:
BE-001KU-01 backend.storage_security.credential_api_handler_implementation.set_mutation baseline_plan

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
