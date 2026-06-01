# v4.16.0 backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering parent residual judgment

> Version type: MINOR architecture / governance
> Execution tier: lightweight
> Batch: BE-001GE-01
> Node: `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering`
> Parent: `backend.graph_compile.quantscript_graph.formal_module_conversion`
> Stage: `parent_residual_judgment`
> Movement: no code movement
> Speed protocol: `recursive_speed_protocol`
> Next step: BE-001GF-01 `ma_deviation_lowering` `baseline_plan`

---

## Matrix Impact

| Matrix | Impact node | Change type |
| --- | --- | --- |
| Flow matrix | BE-001GE-01 parent residual judgment | recursive selection |
| Norm matrix | leaf split gate / parent-child communication / release transition guard | hard rule execution |
| Guidance matrix | `root.backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering` | child selection |
| Module tree | `intent_lowering` residual queue | selects `ma_deviation_lowering` |

---

## Parent Residual Snapshot

Completed children:

```text
spread_observer_lowering stop_split: true
macd_lowering stop_split: true
double_ma_lowering stop_split: true
rsi_lowering stop_split: true
```

Open residuals:

```text
ma_deviation_lowering
momentum_lowering
zscore_lowering
shared_intent_context
unsupported_intent_failure
```

Current parent still owns shared context:

```text
node_id
upstream_edge
source_id
source_var
instrument
module_key dispatch
unsupported intent failure
```

Therefore `intent_lowering stop_split: false` remains true.

---

## leaf_split_decision_gate

leaf_split_base_gate

| Rule | Result | Evidence |
| --- | --- | --- |
| white_box_boundary_named | pass | Candidate child is `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.ma_deviation_lowering`. |
| parent_child_communication_kept | pass | Parent can keep resolving `cfg`, `source_var`, `instrument`, then call one child helper. |
| equivalence_baseline_freezable | pass | Defaults `lookback=15`, `baseline_period=150`, `ma_dev > 1`, and SELL emit are local and freezable. |

leaf_split_positive_trigger

| Trigger | Result | Evidence |
| --- | --- | --- |
| public_or_handler_boundary | no | No external public API or route changes. |
| state_machine_phase | no | No state transition or persistence phase. |
| strategy_branch | yes | `builtin.intent.ma_deviation` is an independent built-in intent branch. |
| independent_failure_mode | no | No independent diagnostic path beyond the shared unsupported fallback. |
| reuse_pressure | no | No second caller; extraction is for white-box branch ownership. |

leaf_split_stop_condition

| Stop condition | Result | Evidence |
| --- | --- | --- |
| micro_leaf_without_owner | no | The whole MA deviation branch has a stable owner; inner config/emit fragments would not. |
| communication_cost_rises | no | Child helper needs only `cfg`, `source_var`, `instrument`, and `qs_lines`. |
| local_proof_missing | no | Existing compile tests and targeted compile gate can verify equivalence. |
| line_count_only | no | Reason is strategy branch ownership, not line count. |

leaf_split_decision_result

```text
continue_split
ma_deviation_lowering_selected
intent_lowering stop_split: false
```

next_recursive_step

```text
BE-001GF-01
backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.ma_deviation_lowering
baseline_plan
```

---

## Speed Protocol Result

`ma_deviation_lowering` qualifies for `lightweight_two_step` because:

1. No public API / route / schema / persistence / lock owner changes are required.
2. Parent-child communication stays one-way.
3. Equivalence surface is a small strategy branch with local defaults and output lines.
4. No sibling horizontal link or release transition is involved.

BE-001GF can therefore use:

```text
BE-001GF-01 baseline_plan
BE-001GF-02 extract_closeout
```

---

## Forbidden Actions

Still forbidden:

```text
formal_module_conversion -> ma_deviation_lowering
macd_lowering -> ma_deviation_lowering
rsi_lowering -> ma_deviation_lowering
double_ma_lowering -> ma_deviation_lowering
spread_observer_lowering -> ma_deviation_lowering
sibling horizontal link
release transition
```

---

## Verification Gates

This batch must pass:

```powershell
git diff --check
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
cargo fmt --check
cargo check -p quantpilot
```
