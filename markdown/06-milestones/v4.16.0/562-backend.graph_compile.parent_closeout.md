# v4.16.0 backend.graph_compile parent closeout sets stop_split true

> Batch: BE-001HL-01
> Node: `backend.graph_compile`
> Parent: `backend`
> Stage: `parent_residual_judgment`
> Movement: no code movement.

---

## Summary

backend.graph_compile parent closeout sets stop_split true.

All selected children under `backend.graph_compile` have been extracted and
closed at this level:

```text
backend.graph_compile.quantscript_graph
backend.graph_compile.compile
backend.graph_compile.graph
```

The remaining parent file is only a route group mediator. Further splitting
would create wrapper fragments rather than a stable white-box owner.

## leaf_split_decision_gate

leaf_split_base_gate

| Rule | Result | Evidence |
| --- | --- | --- |
| white_box_boundary_named | pass | The parent owns the named graph compile route group and three closed child owners. |
| parent_child_communication_kept | pass | `src/backend/graph_compile.rs` delegates to `compile`, `graph`, and `quantscript_graph` without child-to-child route shortcuts. |
| equivalence_baseline_freezable | pass | Each child has its own baseline/extraction closeout and targeted compile/graph/QS tests. |

leaf_split_positive_trigger

| Trigger | Result | Evidence |
| --- | --- | --- |
| public_or_handler_boundary | false | Route handler bodies now live in children; parent retains only route group wrappers. |
| state_machine_phase | false | No state-machine phase remains in the parent. |
| strategy_branch | false | QS graph/formal conversion, compile, and graph persistence branches are closed by child owners. |
| independent_failure_mode | false | Compile errors, graph persistence failures, and QS parse/conversion failures are isolated. |
| reuse_pressure | false | Compatibility exports/shims are root-level or child-owned; parent does not hold reusable logic. |

leaf_split_stop_condition

| Stop condition | Result | Evidence |
| --- | --- | --- |
| micro_leaf_without_owner | true | Further split would only separate three wrapper functions. |
| communication_cost_rises | true | More wrappers would increase route mediation without improving isolation. |
| local_proof_missing | false | Local proof exists through child closeouts and cargo checks/tests. |
| line_count_only | true | Any remaining split signal would be cosmetic line-count pressure. |

leaf_split_decision_result

`stop_split: true` for `backend.graph_compile`.

next_recursive_step

BE-001HM-01 backend parent residual judgment

## Boundary

**Real files**:
- `src/backend/graph_compile.rs`
- `src/backend/graph_compile/compile.rs`
- `src/backend/graph_compile/graph.rs`
- `src/backend/graph_compile/quantscript_graph.rs`

**Markers**:
- `backend.graph_compile parent_closeout`
- `backend.graph_compile recursive_children_closed`
- `backend.graph_compile stop_split: true`

**Next step**:
BE-001HM-01 backend parent residual judgment

---

## Gates

- `git diff --check`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
