# v4.16.0 intent_lowering parent closeout sets stop_split true

> Batch: BE-001GO-01
> Node: `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering`
> Parent: `backend.graph_compile.quantscript_graph.formal_module_conversion`
> Stage: `parent_residual_judgment`
> Movement: no code movement.

---

## Summary

`intent_lowering` has completed its recursive child queue and now reaches
`stop_split_true` for this parent. All built-in branch children, shared context,
and unsupported intent failure ownership have local closeout records.

Closed children:

```text
spread_observer_lowering
macd_lowering
double_ma_lowering
rsi_lowering
ma_deviation_lowering
momentum_lowering
zscore_lowering
shared_intent_context
unsupported_intent_failure
```

No remaining residual inside `intent_lowering` is strong enough to continue
splitting without becoming a micro-leaf or raising communication cost.

## leaf_split_decision_gate

leaf_split_base_gate

| Rule | Result | Evidence |
| --- | --- | --- |
| white_box_boundary_named | yes | Parent and nine children have named module-tree white-box nodes. |
| parent_child_communication_kept | yes | Parent dispatches only to direct children; children do not call sibling children. |
| equivalence_baseline_freezable | yes | Branch-level tests plus compile gates cover all moved intent lowering responsibilities. |

leaf_split_positive_trigger

| Trigger | Result | Evidence |
| --- | --- | --- |
| public_or_handler_boundary | no | No public API, route, schema, persistence, or lock owner remains in this parent. |
| state_machine_phase | no | No remaining state-machine phase exists under `intent_lowering`. |
| strategy_branch | no | All supported strategy branches have been extracted and closed. |
| independent_failure_mode | no | Unsupported intent failure has been extracted and closed. |
| reuse_pressure | no | Shared context has been extracted; no further reusable helper is justified. |

leaf_split_stop_condition

| Stop condition | Result | Evidence |
| --- | --- | --- |
| micro_leaf_without_owner | yes | Remaining candidates would be tiny fragments such as individual constants or single expressions. |
| communication_cost_rises | yes | Further splitting would add helper edges without improving ownership. |
| local_proof_missing | no | Existing tests and compile gates prove the current parent. |
| line_count_only | yes | Any further split would be size-driven rather than boundary-driven. |

leaf_split_decision_result

stop_split_true

next_recursive_step

BE-001GP-01 formal_module_conversion parent residual judgment

## Boundary

**Real files**:
- `src/backend/graph_compile/quantscript_graph/formal_module_conversion/intent_lowering.rs`

**Markers**:
- `intent_lowering parent_closeout`
- `intent_lowering stop_split: true`
- `intent_lowering recursive_children_closed`

**Next step**:
BE-001GP-01 formal_module_conversion parent residual judgment

---

## Gates

- `git diff --check`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
