# v4.16.0 formal_module_conversion parent residual judgment selects input_shape_validation

> Batch: BE-001GT-01
> Node: `backend.graph_compile.quantscript_graph.formal_module_conversion`
> Parent: `backend.graph_compile.quantscript_graph`
> Stage: `parent_residual_judgment`
> Movement: no code movement.

---

## Summary

`formal_module_conversion` has closed `intent_lowering`,
`data_source_lowering`, and `profile_lowering`. The remaining residuals are now:

```text
input_shape_validation
unsupported_node_logging
terminal_parse
```

This judgment selects `input_shape_validation` because it owns the conversion
entry failure path for malformed graph JSON. It is a stronger split candidate
than logging or terminal parse because the behavior is observable through
returned `anyhow` errors and protects every downstream child call.

## leaf_split_decision_gate

leaf_split_base_gate

| Rule | Result | Evidence |
| --- | --- | --- |
| white_box_boundary_named | yes | `input_shape_validation` maps to the entry `graph.nodes` / `graph.edges` required-array checks. |
| parent_child_communication_kept | yes | Parent can call one direct child helper before all lowering children. |
| equivalence_baseline_freezable | yes | The two error strings and returned borrowed slices are local and deterministic. |

leaf_split_positive_trigger

| Trigger | Result | Evidence |
| --- | --- | --- |
| public_or_handler_boundary | no | No route or schema changes are needed. |
| state_machine_phase | no | This is input validation, not a runtime phase. |
| strategy_branch | no | Not a strategy branch. |
| independent_failure_mode | yes | Missing/non-array `nodes` and `edges` return hard conversion errors. |
| reuse_pressure | yes | Centralized graph-shape validation is a reusable entry guard for all formal conversion children. |

leaf_split_stop_condition

| Stop condition | Result | Evidence |
| --- | --- | --- |
| micro_leaf_without_owner | no | The child owns a real entry guard and failure path. |
| communication_cost_rises | no | One helper returns `nodes` and `edges`; downstream communication does not change. |
| local_proof_missing | no | Baseline can add local tests for both invalid shapes before movement. |
| line_count_only | no | Selection is based on failure ownership and guard semantics. |

leaf_split_decision_result

continue_split

next_recursive_step

BE-001GU-01 input_shape_validation baseline_plan

## Boundary

**Real files**:
- `src/backend/graph_compile/quantscript_graph/formal_module_conversion.rs`

**Markers**:
- `formal_module_conversion parent_residual_judgment`
- `input_shape_validation_selected`

**Next step**:
BE-001GU-01 input_shape_validation baseline_plan

---

## Gates

- `git diff --check`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
