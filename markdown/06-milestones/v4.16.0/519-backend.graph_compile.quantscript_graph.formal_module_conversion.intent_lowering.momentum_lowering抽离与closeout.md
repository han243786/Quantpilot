# v4.16.0 backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.momentum_lowering extract closeout

> Version type: MINOR architecture / governance
> Execution tier: lightweight
> Batch: BE-001GH-02
> Baseline plan: `518-backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.momentum_lowering等价基线与抽离方案.md`
> Target child: `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.momentum_lowering`
> Stage: `extract_closeout`
> Movement: Rust branch extraction + single leaf closeout
> Speed protocol: `lightweight_two_step`
> Next step: BE-001GI-01 `intent_lowering` parent residual judgment

---

## Matrix Impact

| Matrix | Impact node | Change type |
| --- | --- | --- |
| Flow matrix | BE-001GH-02 extract + closeout | lightweight two-step stage 2 |
| Norm matrix | parent-child communication / leaf split gate / equivalence proof | hard rule execution |
| Guidance matrix | `root.backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.momentum_lowering` | child white-box node closed |
| Module tree | `intent_lowering -> momentum_lowering` | new one-way child edge |

---

## Actual Extraction

Created child file:

```text
src/backend/graph_compile/quantscript_graph/formal_module_conversion/intent_lowering/momentum_lowering.rs
```

Parent now keeps only:

```rust
mod momentum_lowering;
momentum_lowering::append_momentum_lowering_lines(node_id, cfg, &source_var, instrument, qs_lines);
```

Child owns:

```rust
pub(super) fn append_momentum_lowering_lines(
    node_id: &str,
    cfg: &Value,
    source_var: &str,
    instrument: &str,
    qs_lines: &mut Vec<String>,
)
```

Frozen semantics preserved:

```text
lookback default 10
threshold_ratio preferred
threshold fallback
threshold default 0.02
let {node_id}_signal = momentum(source_var, lookback)
if {node_id}_signal > threshold
emit Intent("BUY", instrument="{instrument}", quantity=1.0)
```

---

## leaf_split_decision_gate

leaf_split_base_gate

| Rule | Result | Evidence |
| --- | --- | --- |
| white_box_boundary_named | pass | `momentum_lowering` now has a named child file and helper. |
| parent_child_communication_kept | pass | Parent resolves shared context and calls child once; child does not call siblings. |
| equivalence_baseline_freezable | pass | BE-001GH-01 froze defaults, fallback order, output lines, and helper surface. |

leaf_split_positive_trigger

| Trigger | Result | Evidence |
| --- | --- | --- |
| public_or_handler_boundary | no | No route, API, schema, persistence, or lock owner changed. |
| state_machine_phase | no | No state transition exists in this branch. |
| strategy_branch | yes | `builtin.intent.momentum` is an independent built-in intent branch. |
| independent_failure_mode | no | Missing-source diagnostics and unsupported module failure remain outside this child. |
| reuse_pressure | no | Single parent caller; extraction is for white-box branch ownership. |

leaf_split_stop_condition

| Stop condition | Result | Evidence |
| --- | --- | --- |
| micro_leaf_without_owner | yes | Further split would produce config decode, signal render, and BUY emit micro-leaves. |
| communication_cost_rises | yes | Further split would pass tiny local values through extra parent-child calls. |
| local_proof_missing | no | Current leaf can be proven by compile/check and formal momentum tests. |
| line_count_only | no | Extraction was based on strategy branch ownership. |

leaf_split_decision_result

```text
stop_split_true
momentum_lowering actual_extraction_done
momentum_lowering closeout_done
momentum_lowering stop_split: true
```

next_recursive_step

```text
BE-001GI-01
backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering
parent_residual_judgment
```

---

## Parent Child Rule

Allowed:

```text
intent_lowering -> momentum_lowering
```

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

## Gates

This batch must pass:

```powershell
git diff --check
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
cargo fmt --check
cargo check -p quantpilot
cargo test -p quantpilot formal_quantscript_compile_endpoint_matches_one_sided_momentum_golden_view
```
