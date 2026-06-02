# v4.16.0 backend.ops_governance.hotswap single leaf closeout stops further split

> Batch: BE-001LM-03
> Node: `backend.ops_governance.hotswap`
> Parent: `backend.ops_governance`
> Stage: `single_leaf_closeout`
> Movement: No code movement.

---

## Summary

`backend.ops_governance.hotswap` is closed as a single leaf after BE-001LM-02 moved the hotswap handlers into the child module.

Current owned files:

- `src/backend/ops_governance/hotswap.rs`
- `src/backend/ops_governance/hotswap/handlers.rs`

The leaf owns:

- hotswap route facade registration;
- submit handler;
- status handler;
- list handler;
- local use of `auth::UserId`;
- local use of `AppState.hotswap_records`;
- local response and problem JSON mapping.

## leaf_split_decision_gate

leaf_split_base_gate

| Rule | Result | Evidence |
| --- | --- | --- |
| white_box_boundary_named | PASS | `backend.ops_governance.hotswap` has a named route facade and local handler owner. |
| parent_child_communication_kept | PASS | Parent ops governance calls the hotswap child facade; handlers stay under that child. |
| equivalence_baseline_freezable | PASS | BE-001LM-01 froze route, handler, DTO, AppState, and response boundaries. |

leaf_split_positive_trigger

| Trigger | Result | Evidence |
| --- | --- | --- |
| public_or_handler_boundary | PARTIAL | The leaf has three route handlers, but they are tightly coupled to one hotswap record store. |
| state_machine_phase | NO | Current lifecycle is only `proposed` plus `idle`; no multi-phase state machine owner exists here. |
| strategy_branch | NO | submit/status/list are projections of one hotswap domain, not separate strategy branches. |
| independent_failure_mode | PARTIAL | Validation and lookup errors differ, but all failures stay within the same record-store surface. |
| reuse_pressure | NO | No shared helper reuse pressure was found. |

leaf_split_stop_condition

| Stop condition | Result | Evidence |
| --- | --- | --- |
| micro_leaf_without_owner | YES | Splitting submit validation, status projection, or list projection would create tiny owners with no durable external contract. |
| communication_cost_rises | YES | More child files would add parent/child bridge overhead around one in-memory hotswap store. |
| local_proof_missing | YES | No dedicated hotswap tests were found; further split would widen proof demands without behavior gain. |
| line_count_only | YES | Further split would be driven mostly by function count and line count. |

leaf_split_decision_result

`stop_split_true`

`backend.ops_governance.hotswap stop_split: true`.

The next recursive step returns to `backend.ops_governance` parent residual judgment.

## Next Step

BE-001LN-01 backend.ops_governance parent_residual_judgment

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
