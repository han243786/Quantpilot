# v4.16.0 intent_lowering parent residual judgment selects unsupported_intent_failure

> Batch: BE-001GM-01
> Node: `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering`
> Parent: `backend.graph_compile.quantscript_graph.formal_module_conversion`
> Stage: `parent_residual_judgment`
> Movement: no code movement.

---

## Summary

This parent residual judgment confirms that `intent_lowering` has one remaining
named residual: `unsupported_intent_failure`.

The residual is not a loose line-count artifact. It owns the default failure
path for unsupported intent module keys and returns a hard `anyhow::bail!`
diagnostic. It should therefore become the next child leaf before the parent can
claim `stop_split_true`.

## leaf_split_decision_gate

leaf_split_base_gate

| Rule | Result | Evidence |
| --- | --- | --- |
| white_box_boundary_named | yes | Default `match ctx.module_key` branch owns unsupported intent failure. |
| parent_child_communication_kept | yes | Next movement can stay parent-to-child by delegating the failure formatting/helper. |
| equivalence_baseline_freezable | yes | Current observable behavior is the unsupported intent `anyhow::bail!` diagnostic. |

leaf_split_positive_trigger

| Trigger | Result | Evidence |
| --- | --- | --- |
| public_or_handler_boundary | no | No route, schema, or public API changes are needed. |
| state_machine_phase | no | This residual is not a state-machine phase. |
| strategy_branch | yes | It is the final default branch of intent module dispatch. |
| independent_failure_mode | yes | It produces a standalone unsupported-module diagnostic and aborts lowering. |
| reuse_pressure | no | No reuse pressure exists yet; the reason to split is failure ownership. |

leaf_split_stop_condition

| Stop condition | Result | Evidence |
| --- | --- | --- |
| micro_leaf_without_owner | no | The residual has a clear failure-path owner. |
| communication_cost_rises | no | Parent-to-child helper call is enough; no sibling communication is introduced. |
| local_proof_missing | no | Compile gate plus existing zscore golden-view test cover unchanged dispatch behavior. |
| line_count_only | no | Split is justified by failure semantics, not by size. |

leaf_split_decision_result

continue_split

next_recursive_step

BE-001GN-01 unsupported_intent_failure baseline_plan

## Boundary

**Real files**:
- `src/backend/graph_compile/quantscript_graph/formal_module_conversion/intent_lowering.rs`

**Markers**:
- `intent_lowering parent_residual_judgment`
- `unsupported_intent_failure_selected`

**Next step**:
BE-001GN-01 unsupported_intent_failure baseline_plan

---

## Gates

- `git diff --check`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
