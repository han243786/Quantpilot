# v4.16.0 backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering parent residual judgment

> Version type: MINOR architecture / governance
> Execution tier: lightweight
> Batch: BE-001GG-01
> Node: `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering`
> Parent: `backend.graph_compile.quantscript_graph.formal_module_conversion`
> Stage: `parent_residual_judgment`
> Movement: no code movement
> Speed protocol: `recursive_speed_protocol`
> Next step: BE-001GH-01 `momentum_lowering` `baseline_plan`

---

## Matrix Impact

| Matrix | Impact node | Change type |
| --- | --- | --- |
| Flow matrix | BE-001GG-01 parent residual judgment | recursive selection |
| Norm matrix | leaf split gate / parent-child communication / release transition guard | hard rule execution |
| Guidance matrix | `root.backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering` | child selection |
| Module tree | `intent_lowering` residual queue | selects `momentum_lowering` |

---

## Parent Residual Snapshot

Completed children:

```text
spread_observer_lowering stop_split: true
macd_lowering stop_split: true
double_ma_lowering stop_split: true
rsi_lowering stop_split: true
ma_deviation_lowering stop_split: true
```

Open residuals:

```text
momentum_lowering
zscore_lowering
shared_intent_context
unsupported_intent_failure
```

Current parent still owns:

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
| white_box_boundary_named | pass | Candidate child is `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.momentum_lowering`. |
| parent_child_communication_kept | pass | Parent can keep resolving `node_id`, `cfg`, `source_var`, `instrument`, then call one child helper. |
| equivalence_baseline_freezable | pass | Defaults `lookback=10`, `threshold_ratio`/`threshold` fallback, `momentum` line, threshold guard, and BUY emit are local and freezable. |

leaf_split_positive_trigger

| Trigger | Result | Evidence |
| --- | --- | --- |
| public_or_handler_boundary | no | No external API, route, schema, persistence, or lock owner changes. |
| state_machine_phase | no | No state transition or persistence phase. |
| strategy_branch | yes | `builtin.intent.momentum` is an independent built-in intent branch. |
| independent_failure_mode | no | Missing source diagnostics and unsupported module failure remain shared/parent-owned. |
| reuse_pressure | no | Single caller; extraction is for branch ownership and white-box closure. |

leaf_split_stop_condition

| Stop condition | Result | Evidence |
| --- | --- | --- |
| micro_leaf_without_owner | no | The whole momentum branch has a stable owner; inner config/render/emit pieces would not. |
| communication_cost_rises | no | Child helper can use `node_id`, `cfg`, `source_var`, `instrument`, and `qs_lines` only. |
| local_proof_missing | no | Existing formal momentum compile tests and compile/check gates can cover equivalence. |
| line_count_only | no | Reason is strategy branch ownership, not line count. |

leaf_split_decision_result

```text
continue_split
momentum_lowering_selected
intent_lowering stop_split: false
```

next_recursive_step

```text
BE-001GH-01
backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.momentum_lowering
baseline_plan
```

---

## Speed Protocol Result

`momentum_lowering` qualifies for `lightweight_two_step`:

1. No public API / route / schema / persistence / lock owner changes are required.
2. Parent-child communication remains one-way.
3. The branch has local defaults and a compact output surface.
4. No sibling horizontal link or release transition is involved.

BE-001GH can therefore use:

```text
BE-001GH-01 baseline_plan
BE-001GH-02 extract_closeout
```

---

## Forbidden Actions

Still forbidden:

```text
formal_module_conversion -> momentum_lowering
ma_deviation_lowering -> momentum_lowering
zscore_lowering -> momentum_lowering
runtime sibling -> momentum_lowering
frontend -> momentum_lowering
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
