# v4.16.0 backend.storage_security.credential_api_handler_implementation parent residual judgment selects list_projection

> Batch: BE-001KP-01
> Node: `backend.storage_security.credential_api_handler_implementation`
> Parent: `backend.storage_security`
> Stage: `parent_residual_judgment`
> Movement: No code movement.

---

## Summary

`backend.storage_security.credential_api_handler_implementation.list_projection` is selected as the next child.

This child owns only the GET/read side of the credential API handler implementation:

- `unscoped_services_for`
- `list_credentials`
- `GET /api/credentials` response projection from scoped vault keys to unscoped service labels
- vault unavailable mapping for the list path

This is selected first because it is the smallest independent branch and does not need `scoped_cv_key`, set validation, field conversion, delete validation, audit mutation logs, or delete not-found mapping.

Remaining parent residuals:

- `set_mutation`
- `delete_mutation`
- route registration facade inside the handler implementation parent
- shared imports and parent-owned branch routing

## leaf_split_decision_gate

leaf_split_base_gate

| Rule | Result | Evidence |
| --- | --- | --- |
| white_box_boundary_named | PASS | The selected child is `backend.storage_security.credential_api_handler_implementation.list_projection`. |
| parent_child_communication_kept | PASS | The child will remain under the handler implementation parent; set/delete siblings are not touched. |
| equivalence_baseline_freezable | PASS | The list branch has a compact boundary: prefix filter, prefix strip, vault unavailable mapping, and `{"services": services}` JSON. |

leaf_split_positive_trigger

| Trigger | Result | Evidence |
| --- | --- | --- |
| public_or_handler_boundary | PASS | The child owns the `GET /api/credentials` handler branch. |
| state_machine_phase | PASS | The branch performs user extraction, vault presence check, scoped list projection, and JSON response. |
| strategy_branch | PASS | List/read projection is separate from set and delete mutation branches. |
| independent_failure_mode | PASS | Prefix filtering/stripping can regress independently from mutation validation and delete error mapping. |
| reuse_pressure | PARTIAL | Reuse is limited, but extracting it improves targeted proof and future route tests. |

leaf_split_stop_condition

| Stop condition | Result | Evidence |
| --- | --- | --- |
| micro_leaf_without_owner | NO | The selected child owns a real GET/read projection branch. |
| communication_cost_rises | NO | A single child function pair can be delegated from the parent without sibling shortcuts. |
| local_proof_missing | NO | BE-001KO-03 passed `cargo check` and credential filtered tests after handler extraction. |
| line_count_only | NO | Selection is based on handler branch ownership, not file length. |

leaf_split_decision_result

`select_child: backend.storage_security.credential_api_handler_implementation.list_projection`

`backend.storage_security.credential_api_handler_implementation stop_split: false`.

BE-001KQ-01 must freeze the list branch before any movement. It must not move set/delete logic, `scoped_cv_key`, route registration ownership, auth internals, vault internals, or release-transition shortcuts.

next_recursive_step

BE-001KQ-01 backend.storage_security.credential_api_handler_implementation.list_projection baseline_plan
## Boundary

**Real files**:
- `src/backend/storage_security/credential_api_handler_implementation.rs`

**Markers**:
- `BE-001KP-01`
- `select_list_projection`
- `get_credentials_branch`
- `set_mutation_deferred`
- `delete_mutation_deferred`
- `release_transition_guard`

**Next step**:
BE-001KQ-01 backend.storage_security.credential_api_handler_implementation.list_projection baseline_plan

---

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p quantpilot credential --lib`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
