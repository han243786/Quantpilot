# v4.16.0 backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.rsi_lowering single leaf closeout
> Version type: MINOR architecture / governance
> Execution tier: lightweight
> Batch: BE-001GD-04
> Baseline: `511-backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.rsi_lowering抽离记录.md`
> Target child: `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.rsi_lowering`
> Judgment: single leaf closeout, stop further split
> Module tree coordinate: `root.backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.rsi_lowering`
> Code action: no code movement
> Next step: BE-001GE-01 `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering` parent residual judgment

---

## Matrix Impact

| Matrix | Impact node | Change type |
| --- | --- | --- |
| Flow matrix | BE-001GD-04 `rsi_lowering` single leaf closeout | child closeout |
| Norm matrix | closeout / stop_split true / equivalence evidence / parent residual return | lightweight tier |
| Guidance matrix | `root.backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.rsi_lowering` | child white-box node closed |
| Module tree | `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.rsi_lowering` | stop_split: true |

---

## Completion Evidence

Completed:

```text
rsi_lowering baseline_frozen
rsi_lowering plan_frozen
rsi_lowering actual_extraction_done
rsi_lowering closeout_done
rsi_lowering stop_split: true
```

Real files:

```text
src/backend/graph_compile/quantscript_graph/formal_module_conversion/intent_lowering/rsi_lowering.rs
src/backend/graph_compile/quantscript_graph/formal_module_conversion/intent_lowering.rs
```

Parent keeps only:

```text
mod rsi_lowering;
rsi_lowering::append_rsi_lowering_lines
```

Helper input surface:

```rust
pub(super) fn append_rsi_lowering_lines(
    node_id: &str,
    cfg: &Value,
    source_var: &str,
    instrument: &str,
    qs_lines: &mut Vec<String>,
)
```

---

## Further Split Judgment

This leaf does not split further.

Reasons:

1. `period`, `oversold_threshold`, and `oversold` form one tiny RSI config decode.
2. `let {}_signal = rsi({}, {})` and `if {}_signal < {}` are one sequential signal rendering unit.
3. BUY emit depends only on `instrument` and fixed quantity; splitting it would add parent-child wiring without a stable owner.
4. The helper owns exactly one built-in branch and is already below the standard leaf threshold.
5. The recursive flow should return to `intent_lowering` parent residual judgment for `ma_deviation`, `momentum`, `zscore`, shared context, and unsupported failure.

Therefore:

```text
rsi_config_decode_micro_leaf rejected
rsi_signal_rendering_micro_leaf rejected
rsi_buy_emit_micro_leaf rejected
```

---

## Equivalence Frozen

Closeout keeps these semantics frozen:

```text
builtin.intent.rsi
period default 14
oversold_threshold
oversold
oversold default 30.0
let {}_signal = rsi({}, {})
if {}_signal < {}
emit Intent("BUY", instrument="{}", quantity=1.0)
```

Parent `node_id`, `source_var`, and `instrument` are still resolved by `intent_lowering`; child consumes only explicit parent inputs.

---

## Parent Child Rule

The only new allowed connection remains:

```text
intent_lowering -> rsi_lowering
```

Existing allowed connections remain:

```text
formal_module_conversion -> intent_lowering
intent_lowering -> spread_observer_lowering
intent_lowering -> macd_lowering
intent_lowering -> double_ma_lowering
```

Still forbidden:

```text
formal_module_conversion -> rsi_lowering
compile_api -> rsi_lowering
graph_quantscript_api -> rsi_lowering
graph_api -> rsi_lowering
runtime sibling -> rsi_lowering
frontend -> rsi_lowering
sibling horizontal link
release transition
```

---

## Next Boundary

Next step can only return to the parent:

```text
BE-001GE-01
backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering
root.backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering
intent_lowering stop_split: false
```

BE-001GE-01 can only judge `intent_lowering` residuals and choose the next child. It must not directly move `ma_deviation`, `momentum`, `zscore`, shared context, unsupported intent failure, or release transition.

---

## Verification Gates

This batch is a `no code movement` closeout. Run before commit:

```powershell
git diff --check
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
cargo fmt --check
cargo check -p quantpilot
cargo test -p quantpilot formal_quantscript_compile_endpoint_returns_structured_lowering_diagnostic_for_missing_macd_source
```

---

## Hallucination Checks

When claiming BE-001GD-04 is complete, state only:

1. Current batch is a `no code movement` single leaf closeout.
2. `rsi_lowering closeout_done` and `rsi_lowering stop_split: true` are true.
3. Next step returns to BE-001GE-01 `intent_lowering` parent residual judgment.
4. Do not claim `intent_lowering` is closed.
5. Do not claim `formal_module_conversion`, `backend.graph_compile`, `backend`, or Rust restructuring is closed.

---

## Acceptance Criteria

1. This closeout file is covered by milestone index, module tree, full feature tree, and governance gates.
2. `rsi_lowering closeout_done` and `rsi_lowering stop_split: true` are recorded.
3. Next step is fixed to BE-001GE-01 `intent_lowering` parent residual judgment.
4. Governance gates, full feature tree coverage, UTF-8, Rust fmt/check, MACD targeted test, and `git diff --check` all pass.
