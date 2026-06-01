# v4.16.0 formal_module_conversion parent residual judgment selects profile_lowering

> Batch: BE-001GR-01
> Node: `backend.graph_compile.quantscript_graph.formal_module_conversion`
> Parent: `backend.graph_compile.quantscript_graph`
> Stage: `parent_residual_judgment`
> Movement: no code movement.

---

## Summary

`formal_module_conversion` has closed `intent_lowering` and
`data_source_lowering`. The remaining residuals are now:

```text
input_shape_validation
profile_lowering
unsupported_node_logging
terminal_parse
```

This judgment selects `profile_lowering` because it owns the cohesive
risk/execution profile branch pair: risk profile defaults, execution profile
defaults, and formal QuantScript profile line rendering.

## leaf_split_decision_gate

leaf_split_base_gate

| Rule | Result | Evidence |
| --- | --- | --- |
| white_box_boundary_named | yes | `profile_lowering` maps to the `risk` and `execution` match arms. |
| parent_child_communication_kept | yes | Parent can delegate `nodes` and `qs_lines` to one direct child helper. |
| equivalence_baseline_freezable | yes | Risk/execution defaults and rendered profile line shapes are local and deterministic. |

leaf_split_positive_trigger

| Trigger | Result | Evidence |
| --- | --- | --- |
| public_or_handler_boundary | no | No route, schema, or public API changes are needed. |
| state_machine_phase | no | This is graph-to-QS lowering, not a runtime phase. |
| strategy_branch | yes | It is a full formal conversion branch pair for risk/execution nodes. |
| independent_failure_mode | no | Unsupported-node logging remains parent-owned for now. |
| reuse_pressure | yes | Profile rendering is cohesive and likely to grow with future formal profile options. |

leaf_split_stop_condition

| Stop condition | Result | Evidence |
| --- | --- | --- |
| micro_leaf_without_owner | no | The selected child has multiple real white-box responsibilities. |
| communication_cost_rises | no | One parent-to-child helper call preserves conversion order. |
| local_proof_missing | no | Baseline can freeze exact defaults and output lines before movement. |
| line_count_only | no | Selection is based on branch ownership, not size alone. |

leaf_split_decision_result

continue_split

next_recursive_step

BE-001GS-01 profile_lowering baseline_plan

## Boundary

**Real files**:
- `src/backend/graph_compile/quantscript_graph/formal_module_conversion.rs`

**Markers**:
- `formal_module_conversion parent_residual_judgment`
- `profile_lowering_selected`

**Next step**:
BE-001GS-01 profile_lowering baseline_plan

---

## Gates

- `git diff --check`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
