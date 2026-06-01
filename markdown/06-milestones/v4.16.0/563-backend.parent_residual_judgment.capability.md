# v4.16.0 backend parent residual judgment selects capability

> Batch: BE-001HM-01
> Node: `backend`
> Parent: `root`
> Stage: `parent_residual_judgment`
> Movement: no code movement.

---

## Summary

backend parent residual judgment selects capability.

Closed top-level backend children now include `interface_boundary`, `runtime`,
and `graph_compile`. Remaining backend residuals are:

```text
capability
strategy_config
storage_security
ops_governance
app_state_wiring
test_support
```

The next recursive child is `backend.capability` because the real capability
snapshot owner still lives in `src/capability_api.rs`, while the backend child
only delegates through `snapshot.rs`.

## leaf_split_decision_gate

leaf_split_base_gate

| Rule | Result | Evidence |
| --- | --- | --- |
| white_box_boundary_named | pass | `backend.capability` is a named top-level backend child with a real old-owner residual. |
| parent_child_communication_kept | pass | Current route/caller flow goes through `backend.capability -> snapshot`; no sibling shortcut is introduced. |
| equivalence_baseline_freezable | pass | `src/capability_api.rs` still owns the current capability response and hash behavior for baseline freeze. |

leaf_split_positive_trigger

| Trigger | Result | Evidence |
| --- | --- | --- |
| public_or_handler_boundary | true | `get_capabilities` is exposed through the backend capability facade and route stack. |
| state_machine_phase | false | Capability is a snapshot/contract domain, not a state-machine phase. |
| strategy_branch | false | Strategy capability data is declarative; no strategy branch movement is selected here. |
| independent_failure_mode | true | Capability hash/context/unsupported-module reasons can fail or drift independently from runtime and graph compile. |
| reuse_pressure | true | Capability contract/hash/context helpers are reused by governance/runtime surfaces and should sit behind the capability owner. |

leaf_split_stop_condition

| Stop condition | Result | Evidence |
| --- | --- | --- |
| micro_leaf_without_owner | false | Capability is a stable backend top-level owner. |
| communication_cost_rises | false | Moving implementation into `backend.capability` removes old-owner delegation. |
| local_proof_missing | false | Capability tests and frontend governance tests can verify the next movement. |
| line_count_only | false | The split is driven by public snapshot ownership and helper reuse, not line count. |

leaf_split_decision_result

`backend stop_split: false`.

Selected next child: `backend.capability`.

next_recursive_step

BE-001HN-01 backend.capability baseline_plan

## Boundary

**Real files**:
- `src/backend/capability.rs`
- `src/backend/capability/snapshot.rs`
- `src/capability_api.rs`

**Markers**:
- `backend capability_selected`
- `graph_compile closed`
- `capability residual open`

**Next step**:
BE-001HN-01 backend.capability baseline_plan

---

## Gates

- `git diff --check`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
