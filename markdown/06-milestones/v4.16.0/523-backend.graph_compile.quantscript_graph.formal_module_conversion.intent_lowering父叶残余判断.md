# v4.16.0 backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering parent residual judgment

> Version type: MINOR architecture / governance
> Execution tier: lightweight
> Batch: BE-001GK-01
> Node: `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering`
> Parent: `backend.graph_compile.quantscript_graph.formal_module_conversion`
> Stage: `parent_residual_judgment`
> Movement: no code movement
> Speed protocol: `recursive_speed_protocol`
> Next step: BE-001GL-01 `shared_intent_context` `baseline_plan`

---

## Matrix Impact

| Matrix | Impact node | Change type |
| --- | --- | --- |
| Flow matrix | BE-001GK-01 parent residual judgment | recursive selection |
| Norm matrix | leaf split gate / parent-child communication / release transition guard | hard rule execution |
| Guidance matrix | `root.backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering` | child selection |
| Module tree | `intent_lowering` residual queue | selects `shared_intent_context` |

---

## Parent Residual Snapshot

Completed children:

```text
spread_observer_lowering stop_split: true
macd_lowering stop_split: true
double_ma_lowering stop_split: true
rsi_lowering stop_split: true
ma_deviation_lowering stop_split: true
momentum_lowering stop_split: true
zscore_lowering stop_split: true
```

Open residuals:

```text
shared_intent_context
unsupported_intent_failure
```

Current parent still owns context resolution before dispatch:

```text
module_key
cfg
instrument
node_id
upstream_edge
source_id
source_var
```

The unsupported module failure remains parent-owned for now.

---

## leaf_split_decision_gate

leaf_split_base_gate

| Rule | Result | Evidence |
| --- | --- | --- |
| white_box_boundary_named | pass | Candidate child is `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.shared_intent_context`. |
| parent_child_communication_kept | pass | Parent can call context helper once per intent node and pass the resulting fields to branch children. |
| equivalence_baseline_freezable | pass | Context defaults and normalization are local: `instrument=BTCUSDT`, missing source defaults to `data`, and `-` / `.` normalize to `_`. |

leaf_split_positive_trigger

| Trigger | Result | Evidence |
| --- | --- | --- |
| public_or_handler_boundary | no | No public API, route, schema, persistence, or lock owner changes. |
| state_machine_phase | no | No state transition or persistence phase. |
| strategy_branch | no | This is shared dispatch context, not a strategy branch. |
| independent_failure_mode | no | Missing upstream source currently uses fallback, not an error path. |
| reuse_pressure | yes | Every built-in branch consumes the same `node_id`, `cfg`, `instrument`, and `source_var` context. |

leaf_split_stop_condition

| Stop condition | Result | Evidence |
| --- | --- | --- |
| micro_leaf_without_owner | no | The context tuple is a stable shared owner across all branch calls. |
| communication_cost_rises | no | Parent still mediates all calls; no child-to-child or sibling horizontal link is introduced. |
| local_proof_missing | no | Compile/check and formal built-in intent tests cover the same rendered output. |
| line_count_only | no | Reason is shared context ownership, not line count. |

leaf_split_decision_result

```text
continue_split
shared_intent_context_selected
intent_lowering stop_split: false
```

next_recursive_step

```text
BE-001GL-01
backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.shared_intent_context
baseline_plan
```

---

## Speed Protocol Result

`shared_intent_context` qualifies for `lightweight_two_step`:

1. It has no external API or state owner changes.
2. It preserves parent-mediated communication.
3. It has a local, freezeable equivalence surface.
4. It does not require release transition or sibling horizontal link.

BE-001GL can therefore use:

```text
BE-001GL-01 baseline_plan
BE-001GL-02 extract_closeout
```

---

## Forbidden Actions

Still forbidden:

```text
shared_intent_context -> child lowering modules
child lowering modules -> shared_intent_context
runtime sibling -> shared_intent_context
frontend -> shared_intent_context
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
