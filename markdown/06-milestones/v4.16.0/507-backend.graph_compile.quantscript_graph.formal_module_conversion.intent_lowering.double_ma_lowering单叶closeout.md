# v4.16.0 backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.double_ma_lowering single leaf closeout
> Version type: MINOR architecture / governance
> Execution tier: lightweight
> Batch: BE-001GB-04
> Baseline: `506-backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.double_ma_lowering抽离记录.md`
> Target child: `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.double_ma_lowering`
> Judgment: single leaf closeout, stop further split
> Module tree coordinate: `root.backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.double_ma_lowering`
> Code action: no code movement
> Next step: BE-001GC-01 `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering` parent residual judgment

---

## Matrix Impact

| Matrix | Impact node | Change type |
| --- | --- | --- |
| Flow matrix | BE-001GB-04 `double_ma_lowering` single leaf closeout | child closeout |
| Norm matrix | closeout / stop_split true / equivalence evidence / parent residual return | lightweight tier |
| Guidance matrix | `root.backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.double_ma_lowering` | child white-box node closed |
| Module tree | `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.double_ma_lowering` | stop_split: true |

---

## Completion Evidence

Completed:

```text
double_ma_lowering baseline_frozen
double_ma_lowering plan_frozen
double_ma_lowering actual_extraction_done
double_ma_lowering closeout_done
double_ma_lowering stop_split: true
```

Real files:

```text
src/backend/graph_compile/quantscript_graph/formal_module_conversion/intent_lowering/double_ma_lowering.rs
src/backend/graph_compile/quantscript_graph/formal_module_conversion/intent_lowering.rs
```

Parent keeps only:

```text
mod double_ma_lowering;
double_ma_lowering::append_double_ma_lowering_lines
```

Helper input surface:

```rust
pub(super) fn append_double_ma_lowering_lines(
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

1. `fast_period` and `slow_period` form one tiny double MA config decode.
2. `let fast = sma({}, {})`, `let slow = sma({}, {})`, and `fast > slow` are a single sequential signal rendering unit.
3. BUY emit only depends on `instrument` and fixed quantity; splitting it would add parent-child wiring without a stable owner.
4. The helper owns exactly one built-in branch and is already smaller than the standard leaf threshold.
5. The recursive flow should return to `intent_lowering` parent residual judgment for `rsi`, `ma_deviation`, `momentum`, `zscore`, shared context, and unsupported failure.

Therefore:

```text
double_ma_config_decode_micro_leaf rejected
double_ma_signal_rendering_micro_leaf rejected
double_ma_buy_emit_micro_leaf rejected
```

---

## Equivalence Frozen

Closeout keeps these semantics frozen:

```text
builtin.intent.double_ma
fast_period default 20
slow_period default 50
let fast = sma({}, {})
let slow = sma({}, {})
fast > slow
emit Intent("BUY", instrument="{}", quantity=1.0)
```

Parent `source_var` and `instrument` are still resolved by `intent_lowering`; child consumes only explicit parent inputs.

---

## Parent Child Rule

The only new allowed connection remains:

```text
intent_lowering -> double_ma_lowering
```

Existing allowed connections remain:

```text
formal_module_conversion -> intent_lowering
intent_lowering -> spread_observer_lowering
intent_lowering -> macd_lowering
```

Still forbidden:

```text
formal_module_conversion -> double_ma_lowering
compile_api -> double_ma_lowering
graph_quantscript_api -> double_ma_lowering
graph_api -> double_ma_lowering
runtime sibling -> double_ma_lowering
frontend -> double_ma_lowering
sibling horizontal link
release transition
```

---

## Next Boundary

Next step can only return to the parent:

```text
BE-001GC-01
backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering
root.backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering
intent_lowering stop_split: false
```

BE-001GC-01 can only judge `intent_lowering` residuals and choose the next child. It must not directly move `rsi`, `ma_deviation`, `momentum`, `zscore`, shared context, unsupported intent failure, or release transition.

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

When claiming BE-001GB-04 is complete, state only:

1. Current batch is a `no code movement` single leaf closeout.
2. `double_ma_lowering closeout_done` and `double_ma_lowering stop_split: true` are true.
3. Next step returns to BE-001GC-01 `intent_lowering` parent residual judgment.
4. Do not claim `intent_lowering` is closed.
5. Do not claim `formal_module_conversion`, `backend.graph_compile`, `backend`, or Rust restructuring is closed.

---

## Acceptance Criteria

1. This closeout file is covered by milestone index, module tree, full feature tree, and governance gates.
2. `double_ma_lowering closeout_done` and `double_ma_lowering stop_split: true` are recorded.
3. Next step is fixed to BE-001GC-01 `intent_lowering` parent residual judgment.
4. Governance gates, full feature tree coverage, UTF-8, Rust fmt/check, MACD targeted test, and `git diff --check` all pass.
