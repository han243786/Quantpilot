# v4.16.0 backend.storage_security.credential_api_handler_implementation parent residual judgment selects key_scope

> Batch: BE-001KR-01
> Node: `backend.storage_security.credential_api_handler_implementation`
> Parent: `backend.storage_security`
> Stage: `parent_residual_judgment`
> Movement: No code movement.

---

## Summary

`backend.storage_security.credential_api_handler_implementation.key_scope` is selected as the next child.

After closing `list_projection`, the parent still owns:

- route registration for POST and DELETE branches
- shared `scoped_cv_key`
- `set_credential`
- `delete_credential`

`scoped_cv_key` is selected before `set_mutation` and `delete_mutation` because it is the shared security boundary used by both mutation branches. Extracting it first lets the parent keep a mediated key-scope bridge for future children and avoids moving the `{user_id}:{service}` format into either mutation sibling.

Remaining parent residuals after this child:

- `set_mutation`
- `delete_mutation`
- route registration facade inside the handler implementation parent

## leaf_split_decision_gate

leaf_split_base_gate

| Rule | Result | Evidence |
| --- | --- | --- |
| white_box_boundary_named | PASS | The selected child is `backend.storage_security.credential_api_handler_implementation.key_scope`. |
| parent_child_communication_kept | PASS | The key-scope child will stay under the handler implementation parent; future set/delete children can call the parent bridge instead of a sibling. |
| equivalence_baseline_freezable | PASS | The boundary is one exact format: `{user_id}:{service}`. |

leaf_split_positive_trigger

| Trigger | Result | Evidence |
| --- | --- | --- |
| public_or_handler_boundary | PARTIAL | The helper is private, but it defines the credential key used by public POST and DELETE handlers. |
| state_machine_phase | PASS | Key scoping is a pre-vault mutation phase shared by set and delete. |
| strategy_branch | PASS | It branches credential ownership by user before both mutation strategies. |
| independent_failure_mode | PASS | A key-format regression can cross-contaminate users or break delete independently from validation and vault persistence. |
| reuse_pressure | PASS | Both set and delete mutation branches use the same scoped key format. |

leaf_split_stop_condition

| Stop condition | Result | Evidence |
| --- | --- | --- |
| micro_leaf_without_owner | NO | Although small, the child owns a shared user isolation key contract. |
| communication_cost_rises | NO | Parent-mediated key bridge reduces future sibling coupling for set/delete extraction. |
| local_proof_missing | NO | The next baseline can freeze the exact format before movement. |
| line_count_only | NO | Selection is driven by shared security ownership, not line count. |

leaf_split_decision_result

`select_child: backend.storage_security.credential_api_handler_implementation.key_scope`

`backend.storage_security.credential_api_handler_implementation stop_split: false`.

BE-001KS-01 must freeze the exact key format before any movement. It must not move set/delete handlers, list projection, route registration, auth internals, vault internals, or release-transition shortcuts.

next_recursive_step

BE-001KS-01 backend.storage_security.credential_api_handler_implementation.key_scope baseline_plan
## Boundary

**Real files**:
- `src/backend/storage_security/credential_api_handler_implementation.rs`

**Markers**:
- `BE-001KR-01`
- `select_key_scope`
- `shared_user_service_key`
- `set_mutation_deferred`
- `delete_mutation_deferred`
- `release_transition_guard`

**Next step**:
BE-001KS-01 backend.storage_security.credential_api_handler_implementation.key_scope baseline_plan

---

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p quantpilot credential --lib`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
