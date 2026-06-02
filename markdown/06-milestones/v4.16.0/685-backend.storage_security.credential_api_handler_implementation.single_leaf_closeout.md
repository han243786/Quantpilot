# v4.16.0 backend.storage_security.credential_api_handler_implementation single leaf closeout continues split

> Batch: BE-001KO-03
> Node: `backend.storage_security.credential_api_handler_implementation`
> Parent: `backend.storage_security`
> Stage: `single_leaf_closeout`
> Movement: No code movement.

---

## Summary

`backend.storage_security.credential_api_handler_implementation` remains open and must continue splitting.

The current child owns several real handler responsibilities:

- route registration for `GET/POST /api/credentials` and `DELETE /api/credentials/:service`
- credential key scoping and list projection
- set credential validation, field conversion, vault mutation, audit, and response mapping
- delete credential validation, vault mutation, not-found mapping, audit, and response mapping

These are not facade/import micro leaves. They are distinct security branches with independent failure modes, so `stop_split: false`.

Likely next child candidates:

- `backend.storage_security.credential_api_handler_implementation.list_projection`
- `backend.storage_security.credential_api_handler_implementation.set_mutation`
- `backend.storage_security.credential_api_handler_implementation.delete_mutation`

BE-001KP-01 must choose one child and freeze it before any movement.

## leaf_split_decision_gate

leaf_split_base_gate

| Rule | Result | Evidence |
| --- | --- | --- |
| white_box_boundary_named | PASS | The handler implementation is named and has visible internal candidate branches. |
| parent_child_communication_kept | PASS | The current parent bridge remains intact; no sibling shortcut was introduced. |
| equivalence_baseline_freezable | PASS | BE-001KO-02 passed `cargo check -p quantpilot` and `cargo test -p quantpilot credential --lib`. |

leaf_split_positive_trigger

| Trigger | Result | Evidence |
| --- | --- | --- |
| public_or_handler_boundary | PASS | The file owns three HTTP handlers and route registration. |
| state_machine_phase | PASS | Each handler follows request validation, vault handoff, error mapping, audit, and JSON response phases. |
| strategy_branch | PASS | list, set, and delete are separate security branches. |
| independent_failure_mode | PASS | List prefix projection, set field validation/storage, and delete not-found mapping can regress independently. |
| reuse_pressure | PARTIAL | Reuse is secondary, but smaller branches will improve targeted review and future handler tests. |

leaf_split_stop_condition

| Stop condition | Result | Evidence |
| --- | --- | --- |
| micro_leaf_without_owner | NO | Candidate children own real list/set/delete behavior. |
| communication_cost_rises | NO | Splitting by handler branch reduces mixed security responsibilities without requiring sibling shortcuts. |
| local_proof_missing | NO | BE-001KO-02 local proof exists. |
| line_count_only | NO | Continue decision is driven by handler branch ownership, not file length. |

leaf_split_decision_result

`stop_split_false`

`backend.storage_security.credential_api_handler_implementation stop_split: false`.

The next recursive step returns to this node as a parent residual judgment and must select one child before code movement.

next_recursive_step

BE-001KP-01 backend.storage_security.credential_api_handler_implementation parent_residual_judgment
## Boundary

**Real files**:
- `src/backend/storage_security/credential_api_handler_implementation.rs`

**Markers**:
- `BE-001KO-03`
- `stop_split_false`
- `handler_branch_split_required`
- `list_projection_candidate`
- `set_mutation_candidate`
- `delete_mutation_candidate`
- `release_transition_guard`

**Next step**:
BE-001KP-01 backend.storage_security.credential_api_handler_implementation parent_residual_judgment

---

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p quantpilot credential --lib`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
