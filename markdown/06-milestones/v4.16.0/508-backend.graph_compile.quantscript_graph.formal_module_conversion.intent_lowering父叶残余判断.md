# v4.16.0 backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering parent residual judgment
> Version type: MINOR architecture / governance
> Execution tier: standard
> Batch: BE-001GC-01
> Baseline: `507-backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.double_ma_lowering单叶closeout.md`
> Target parent: `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering`
> Judgment: parent still has residuals, select `rsi_lowering`
> Module tree coordinate: `root.backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering`
> Code action: no code movement
> Next step: BE-001GD-01 `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.rsi_lowering` single child equivalence baseline

---

## Matrix Impact

| Matrix | Impact node | Change type |
| --- | --- | --- |
| Flow matrix | BE-001GC-01 `intent_lowering` parent residual judgment | parent return / next child selection |
| Norm matrix | recursive residual judgment / child selection / stop_split false / release transition guard | standard tier |
| Guidance matrix | `root.backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering` | child queue continues |
| Module tree | `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering` | parent_residual_judgment |

---

## Completed Child Confirmation

Already closed children:

```text
spread_observer_lowering closeout_done
spread_observer_lowering stop_split: true
macd_lowering closeout_done
macd_lowering stop_split: true
double_ma_lowering closeout_done
double_ma_lowering stop_split: true
```

Parent currently connects to children only through controlled parent-child calls:

```rust
mod spread_observer_lowering;
mod macd_lowering;
mod double_ma_lowering;
spread_observer_lowering::append_spread_observer_lowering_lines(...);
macd_lowering::append_macd_lowering_lines(...);
double_ma_lowering::append_double_ma_lowering_lines(...);
```

---

## Current Parent Residuals

`intent_lowering.rs` still owns these residual clusters:

| Residual cluster | Representative behavior | Current judgment |
| --- | --- | --- |
| `shared_intent_context` | `module_key`, `instrument`, `node_id`, upstream edge and `source_var` derivation | defer, still shared by remaining branches |
| `rsi_lowering` | `period` / `oversold_threshold` or `oversold` -> RSI BUY | selected next |
| `ma_deviation_lowering` | `lookback` / `baseline_period` -> MA ratio SELL | later thin child |
| `momentum_lowering` | `lookback` / `threshold_ratio` or `threshold` -> momentum BUY | later thin child |
| `zscore_lowering` | `window` / `entry_z` -> zscore BUY | later thin child |
| `unsupported_intent_failure` | unsupported intent `anyhow::bail!` | defer, keep hard failure at parent boundary |

Therefore the parent remains open:

```text
intent_lowering parent_residual_judgment
intent_lowering stop_split: false
backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering residual_exists
```

---

## Next Child Selection

This batch selects:

```text
BE-001GD-01
backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.rsi_lowering
root.backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.rsi_lowering
rsi_lowering_selected
```

Selection reasons:

1. `spread_observer_lowering`, `macd_lowering`, and `double_ma_lowering` are closed.
2. `rsi` is now the first remaining built-in intent branch in match order.
3. It is a thin branch with one config decode block, one RSI signal line, one threshold guard, and one BUY emit.
4. It still needs parent-provided `node_id`, `source_var`, `instrument`, and `qs_lines`; extracting it preserves the parent-child rule.
5. It reduces parent match-body complexity without touching shared intent context or unsupported failure behavior.

---

## Frozen Invariants For Next Baseline

BE-001GD-01 must freeze:

```text
builtin.intent.rsi
period default 14
oversold_threshold
oversold
rsi({}, {})
{}_signal
if {}_signal < {}
emit Intent("BUY", instrument="{}", quantity=1.0)
```

The parent must retain:

```text
shared_intent_context
builtin.intent.ma_deviation
builtin.intent.momentum
builtin.intent.zscore
builtin.intent.spread_observer
builtin.intent.macd
builtin.intent.double_ma
spread_observer_lowering::append_spread_observer_lowering_lines
macd_lowering::append_macd_lowering_lines
double_ma_lowering::append_double_ma_lowering_lines
unsupported intent
anyhow::bail!
```

Allowed parent-child links:

```text
formal_module_conversion -> intent_lowering
intent_lowering -> spread_observer_lowering
intent_lowering -> macd_lowering
intent_lowering -> double_ma_lowering
intent_lowering -> rsi_lowering
```

Forbidden links:

```text
formal_module_conversion -> rsi_lowering
compile_api -> rsi_lowering
graph_quantscript_api -> rsi_lowering
graph_api -> rsi_lowering
runtime sibling -> rsi_lowering
frontend -> rsi_lowering
sibling horizontal link
```

release transition guard: current work is not in release transition. Do not bypass the parent-child rule for performance.

---

## Out Of Scope

This batch does not:

1. Modify Rust code.
2. Create `rsi_lowering.rs`.
3. Move the `builtin.intent.rsi` branch.
4. Extract shared intent context.
5. Extract `ma_deviation`, `momentum`, `zscore`, or unsupported intent failure.
6. Change parent helper signatures.
7. Add sibling horizontal links.
8. Start release transition.
9. Claim `intent_lowering`, `formal_module_conversion`, `backend.graph_compile`, `backend`, or Rust restructuring is closed.

---

## Next Boundary

Next step can only enter:

```text
BE-001GD-01
backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.rsi_lowering
root.backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.rsi_lowering
```

BE-001GD-01 can only create the single child equivalence baseline for the RSI branch. It must not create a child file, move Rust, rewrite shared context, or start release transition.

---

## Verification Gates

This batch is a `no code movement` parent residual judgment. Run before commit:

```powershell
git diff --check
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
cargo fmt --check
cargo check -p quantpilot
```

---

## Hallucination Checks

When claiming BE-001GC-01 is complete, state only:

1. Current batch is a `no code movement` parent residual judgment.
2. `intent_lowering stop_split: false`.
3. `rsi_lowering_selected` only means the next baseline was selected.
4. Next step can only enter BE-001GD-01 `rsi_lowering` single child equivalence baseline.
5. Do not claim `rsi_lowering` has been extracted.
6. Do not claim `intent_lowering`, `formal_module_conversion`, `backend.graph_compile`, `backend`, or Rust restructuring is closed.

---

## Acceptance Criteria

1. This file is covered by milestone index, module tree, full feature tree, and governance gates.
2. `intent_lowering parent_residual_judgment` is recorded.
3. `rsi_lowering_selected` is recorded.
4. Next step is fixed to BE-001GD-01 `rsi_lowering` single child equivalence baseline.
5. Governance gates, full feature tree coverage, UTF-8, Rust fmt/check, and `git diff --check` all pass.
