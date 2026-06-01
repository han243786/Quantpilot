# v4.16.0 formal_module_conversion parent residual judgment selects terminal_parse

> Batch: BE-001GV-01
> Node: `backend.graph_compile.quantscript_graph.formal_module_conversion`
> Parent: `backend.graph_compile.quantscript_graph`
> Stage: `parent_residual_judgment`
> Movement: no code movement.

---

## Summary

`formal_module_conversion` has closed `intent_lowering`,
`data_source_lowering`, `profile_lowering`, and `input_shape_validation`. The
remaining residuals are:

```text
unsupported_node_logging
terminal_parse
```

This judgment selects `terminal_parse` because it owns the conversion terminal:
closing the generated strategy source, joining QS lines, and invoking
`parse_quant_script_module`. It is a stronger boundary than unsupported-node
logging because it is the final returned `ScriptModule` construction point.

## leaf_split_decision_gate

leaf_split_base_gate

| Rule | Result | Evidence |
| --- | --- | --- |
| white_box_boundary_named | yes | `terminal_parse` maps to `qs_lines.push("}")`, `join("\n")`, and `parse_quant_script_module`. |
| parent_child_communication_kept | yes | Parent can delegate the final QS line vector to one direct child helper. |
| equivalence_baseline_freezable | yes | Generated source joining and parse call behavior are local and deterministic. |

leaf_split_positive_trigger

| Trigger | Result | Evidence |
| --- | --- | --- |
| public_or_handler_boundary | no | No route or schema changes are needed. |
| state_machine_phase | yes | It is the terminal phase of formal graph-to-QS conversion. |
| strategy_branch | no | Not a strategy branch. |
| independent_failure_mode | yes | QS parse failure is the final hard conversion failure path. |
| reuse_pressure | yes | Terminal parse ownership can later support parse diagnostics without touching parent orchestration. |

leaf_split_stop_condition

| Stop condition | Result | Evidence |
| --- | --- | --- |
| micro_leaf_without_owner | no | The child owns a real terminal phase. |
| communication_cost_rises | no | One parent-to-child call consumes `qs_lines` at the existing terminal point. |
| local_proof_missing | no | Baseline can freeze closing brace, join delimiter, and parse call before movement. |
| line_count_only | no | Selection is based on phase ownership. |

leaf_split_decision_result

continue_split

next_recursive_step

BE-001GW-01 terminal_parse baseline_plan

## Boundary

**Real files**:
- `src/backend/graph_compile/quantscript_graph/formal_module_conversion.rs`

**Markers**:
- `formal_module_conversion parent_residual_judgment`
- `terminal_parse_selected`

**Next step**:
BE-001GW-01 terminal_parse baseline_plan

---

## Gates

- `git diff --check`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
