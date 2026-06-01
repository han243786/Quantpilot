# v4.16.0 formal_module_conversion parent residual judgment selects data_source_lowering

> Batch: BE-001GP-01
> Node: `backend.graph_compile.quantscript_graph.formal_module_conversion`
> Parent: `backend.graph_compile.quantscript_graph`
> Stage: `parent_residual_judgment`
> Movement: no code movement.

---

## Summary

`intent_lowering` has closed and the recursion has returned to
`formal_module_conversion`. The remaining parent residuals are:

```text
input_shape_validation
data_source_lowering
profile_lowering
unsupported_node_logging
terminal_parse
```

This judgment selects `data_source_lowering` for the next child because it owns
the complete `data` node branch: config decoding, defaulting, request-option
construction, node id normalization, and `fetch(...)` QS rendering.

## leaf_split_decision_gate

leaf_split_base_gate

| Rule | Result | Evidence |
| --- | --- | --- |
| white_box_boundary_named | yes | `data_source_lowering` maps cleanly to the `node_type == "data"` branch. |
| parent_child_communication_kept | yes | Parent can delegate `nodes` and `qs_lines` to one direct child helper. |
| equivalence_baseline_freezable | yes | Data lowering defaults and fetch rendering are local and can be frozen before movement. |

leaf_split_positive_trigger

| Trigger | Result | Evidence |
| --- | --- | --- |
| public_or_handler_boundary | no | No route or schema changes are needed. |
| state_machine_phase | no | This is graph-to-QS lowering, not a runtime state phase. |
| strategy_branch | yes | It is a full branch in the formal conversion lowering pipeline. |
| independent_failure_mode | no | Validation remains parent-owned for now. |
| reuse_pressure | yes | Data fetch rendering is cohesive and likely to grow with future data modules. |

leaf_split_stop_condition

| Stop condition | Result | Evidence |
| --- | --- | --- |
| micro_leaf_without_owner | no | The branch has a real owner and multiple white-box responsibilities. |
| communication_cost_rises | no | One parent-to-child helper call preserves the current pipeline order. |
| local_proof_missing | no | Existing formal compile tests cover data node conversion; baseline will freeze exact defaults. |
| line_count_only | no | Selection is based on branch ownership and future extension pressure. |

leaf_split_decision_result

continue_split

next_recursive_step

BE-001GQ-01 data_source_lowering baseline_plan

## Boundary

**Real files**:
- `src/backend/graph_compile/quantscript_graph/formal_module_conversion.rs`

**Markers**:
- `formal_module_conversion parent_residual_judgment`
- `data_source_lowering_selected`

**Next step**:
BE-001GQ-01 data_source_lowering baseline_plan

---

## Gates

- `git diff --check`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
