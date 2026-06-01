# v4.16.0 backend.graph_compile parent residual judgment selects compile

> Batch: BE-001HH-01
> Node: `backend.graph_compile`
> Parent: `backend`
> Stage: `parent_residual_judgment`
> Movement: no code movement.

---

## Summary

backend.graph_compile parent residual judgment selects compile.

`backend.graph_compile.quantscript_graph` is now closed and set to `stop_split: true`.
The parent still owns two old-owner residuals through thin route facade children:
`backend.graph_compile.compile -> crate::compile_api` and
`backend.graph_compile.graph -> crate::graph_api`.

The next recursive child is `backend.graph_compile.compile` because `src/compile_api.rs`
contains the compile route surface, runtime protocol compile helper, strategy IR compile
request, and formal QuantScript compile request. It is smaller than `graph_api`, has a
clear public route boundary, and now has a stable parent-mediated QS graph dependency.

## leaf_split_decision_gate

leaf_split_base_gate

| Rule | Result | Evidence |
| --- | --- | --- |
| white_box_boundary_named | pass | `backend.graph_compile` has named route facade children: `compile`, `graph`, and the now-closed `quantscript_graph`. |
| parent_child_communication_kept | pass | `src/backend/graph_compile.rs` delegates through child modules; no release-transition sibling shortcut is introduced. |
| equivalence_baseline_freezable | pass | `compile_api` and `graph_api` still provide the legacy behavior, so the next child can freeze an exact no-code-movement baseline before extraction. |

leaf_split_positive_trigger

| Trigger | Result | Evidence |
| --- | --- | --- |
| public_or_handler_boundary | true | `src/compile_api.rs` owns `/api/runtime/compile`, `/api/strategy-ir/compile`, and `/api/formal-quantscript/compile` route handlers. |
| state_machine_phase | false | The parent itself has no state-machine phase; compile request flow is the selected child residual. |
| strategy_branch | true | Compile has runtime config, strategy IR, and formal QS branches that can be evaluated under a compile child baseline. |
| independent_failure_mode | true | Compile diagnostics, QS protocol conversion, strategy IR failures, and formal QS failures are separable from graph persistence failures. |
| reuse_pressure | true | `compile_runtime_protocol_via_qs` is called outside the route surface and needs a controlled parent-mediated export after extraction. |

leaf_split_stop_condition

| Stop condition | Result | Evidence |
| --- | --- | --- |
| micro_leaf_without_owner | false | `compile` is a durable route/API owner, not a cosmetic wrapper split. |
| communication_cost_rises | false | Moving compile behind the existing `backend.graph_compile.compile` child reduces old-owner delegation without adding sibling links. |
| local_proof_missing | false | Existing compile and formal QS tests can verify the selected child after baseline/extraction. |
| line_count_only | false | The split is driven by route/API ownership and external helper reuse, not line count alone. |

leaf_split_decision_result

`backend.graph_compile stop_split: false`.

Selected next child: `backend.graph_compile.compile`.

Remaining residuals after selection:
- `backend.graph_compile.compile` selected for BE-001HI-01 baseline_plan.
- `backend.graph_compile.graph` remains open and still delegates to `crate::graph_api`.

next_recursive_step

BE-001HI-01 backend.graph_compile.compile baseline_plan

## Boundary

**Real files**:
- `src/backend/graph_compile.rs`
- `src/backend/graph_compile/compile.rs`
- `src/backend/graph_compile/graph.rs`
- `src/compile_api.rs`
- `src/graph_api.rs`

**Markers**:
- `backend.graph_compile parent_residual_judgment`
- `compile_selected`
- `graph_residual_kept_open`

**Next step**:
BE-001HI-01 backend.graph_compile.compile baseline_plan

---

## Gates

- `git diff --check`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
