# v4.16.0 backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.ma_deviation_lowering extract closeout

> Version type: MINOR architecture / governance
> Execution tier: lightweight
> Batch: BE-001GF-02
> Baseline plan: `515-backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.ma_deviation_lowering等价基线与抽离方案.md`
> Target child: `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.ma_deviation_lowering`
> Stage: `extract_closeout`
> Movement: Rust branch extraction + single leaf closeout
> Speed protocol: `lightweight_two_step`
> Next step: BE-001GG-01 `intent_lowering` parent residual judgment

---

## Matrix Impact

| Matrix | Impact node | Change type |
| --- | --- | --- |
| Flow matrix | BE-001GF-02 extract + closeout | lightweight two-step stage 2 |
| Norm matrix | parent-child communication / leaf split gate / equivalence proof | hard rule execution |
| Guidance matrix | `root.backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.ma_deviation_lowering` | child white-box node closed |
| Module tree | `intent_lowering -> ma_deviation_lowering` | new one-way child edge |

---

## Actual Extraction

Created child file:

```text
src/backend/graph_compile/quantscript_graph/formal_module_conversion/intent_lowering/ma_deviation_lowering.rs
```

Parent now keeps only:

```rust
mod ma_deviation_lowering;
ma_deviation_lowering::append_ma_deviation_lowering_lines(cfg, &source_var, instrument, qs_lines);
```

Child owns:

```rust
pub(super) fn append_ma_deviation_lowering_lines(
    cfg: &Value,
    source_var: &str,
    instrument: &str,
    qs_lines: &mut Vec<String>,
)
```

Frozen semantics preserved:

```text
lookback default 15
baseline_period default 150
let ma_dev = sma(source_var, lookback) / sma(source_var, baseline_period)
if ma_dev > 1
emit Intent("SELL", instrument="{instrument}", quantity=1.0)
```

---

## leaf_split_decision_gate

leaf_split_base_gate

| Rule | Result | Evidence |
| --- | --- | --- |
| white_box_boundary_named | pass | `ma_deviation_lowering` now has a named child file and helper. |
| parent_child_communication_kept | pass | Parent resolves shared context and calls child once; child does not call siblings. |
| equivalence_baseline_freezable | pass | BE-001GF-01 froze defaults, output lines, and allowed helper surface. |

leaf_split_positive_trigger

| Trigger | Result | Evidence |
| --- | --- | --- |
| public_or_handler_boundary | no | No route, API, schema, persistence, or lock owner changed. |
| state_machine_phase | no | No state transition exists in this branch. |
| strategy_branch | yes | `builtin.intent.ma_deviation` is an independent built-in intent branch. |
| independent_failure_mode | no | Unsupported intent failure remains parent-owned. |
| reuse_pressure | no | Single parent caller; white-box ownership is the reason for extraction. |

leaf_split_stop_condition

| Stop condition | Result | Evidence |
| --- | --- | --- |
| micro_leaf_without_owner | yes | Further split would produce config decode, line render, and SELL emit micro-leaves. |
| communication_cost_rises | yes | Further split would pass tiny local values through extra parent-child calls. |
| local_proof_missing | no | Current leaf can be proven by compile/check and formal intent lowering tests. |
| line_count_only | no | Extraction was based on strategy branch ownership. |

leaf_split_decision_result

```text
stop_split_true
ma_deviation_lowering actual_extraction_done
ma_deviation_lowering closeout_done
ma_deviation_lowering stop_split: true
```

next_recursive_step

```text
BE-001GG-01
backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering
parent_residual_judgment
```

---

## Parent Child Rule

Allowed:

```text
intent_lowering -> ma_deviation_lowering
```

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

## Gates

This batch must pass:

```powershell
git diff --check
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
cargo fmt --check
cargo check -p quantpilot
cargo test -p quantpilot formal_quantscript_compile_endpoint_returns_structured_lowering_diagnostic_for_missing_macd_source
```
