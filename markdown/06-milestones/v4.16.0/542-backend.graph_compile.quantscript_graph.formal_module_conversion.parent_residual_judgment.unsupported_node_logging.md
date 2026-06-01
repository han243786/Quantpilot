# v4.16.0 formal_module_conversion parent residual judgment selects unsupported_node_logging

> Batch: BE-001GX-01
> Node: `backend.graph_compile.quantscript_graph.formal_module_conversion`
> Parent: `backend.graph_compile.quantscript_graph`
> Stage: `parent_residual_judgment`
> Movement: no code movement.

---

## Summary

`formal_module_conversion` has closed all major lowering and terminal parse
children. The only remaining child-worthy residual is now
`unsupported_node_logging`.

This judgment selects `unsupported_node_logging` because it owns the nonblocking
diagnostic path for unknown node types and the skip list for known-but-not-QS
rendered node types.

## leaf_split_decision_gate

leaf_split_base_gate

| Rule | Result | Evidence |
| --- | --- | --- |
| white_box_boundary_named | yes | `unsupported_node_logging` maps to the remaining known-node no-op list and unknown-node `safe_eprintln!` message. |
| parent_child_communication_kept | yes | Parent can call one direct child helper after profile handling. |
| equivalence_baseline_freezable | yes | Known-node suppression and unknown-node message formatting are local. |

leaf_split_positive_trigger

| Trigger | Result | Evidence |
| --- | --- | --- |
| public_or_handler_boundary | no | No route or schema changes are needed. |
| state_machine_phase | no | This is a diagnostic side path. |
| strategy_branch | no | Not a strategy branch. |
| independent_failure_mode | yes | Unknown node types are intentionally nonblocking diagnostics. |
| reuse_pressure | no | Split is driven by final residual ownership, not reuse pressure. |

leaf_split_stop_condition

| Stop condition | Result | Evidence |
| --- | --- | --- |
| micro_leaf_without_owner | no | The child has a real diagnostic owner and no-op classification list. |
| communication_cost_rises | no | One parent-to-child helper replaces inline match logic. |
| local_proof_missing | no | Baseline can add a local message/classification proof before movement. |
| line_count_only | no | This is the last named diagnostic residual, not a size-only split. |

leaf_split_decision_result

continue_split

next_recursive_step

BE-001GY-01 unsupported_node_logging baseline_plan

## Boundary

**Real files**:
- `src/backend/graph_compile/quantscript_graph/formal_module_conversion.rs`

**Markers**:
- `formal_module_conversion parent_residual_judgment`
- `unsupported_node_logging_selected`

**Next step**:
BE-001GY-01 unsupported_node_logging baseline_plan

---

## Gates

- `git diff --check`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
