# v4.16.0 backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.double_ma_lowering extraction record
> Version type: MINOR architecture / governance
> Execution tier: standard
> Batch: BE-001GB-03
> Baseline: `505-backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.double_ma_lowering抽离方案.md`
> Target child: `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.double_ma_lowering`
> Judgment: actual extraction complete
> Module tree coordinate: `root.backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.double_ma_lowering`
> Code action: actual extraction
> Next step: BE-001GB-04 `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.double_ma_lowering` single leaf closeout

---

## Matrix Impact
| Matrix | Impact node | Change type |
| --- | --- | --- |
| Flow matrix | BE-001GB-03 `double_ma_lowering` actual extraction record | code extraction |
| Norm matrix | actual extraction / parent-child communication / equivalence preservation / release transition guard | standard tier |
| Guidance matrix | `root.backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.double_ma_lowering` | child file landed |
| Module tree | `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.double_ma_lowering` | actual_extraction_done |

---

## Actual Changes

New child file:

```text
src/backend/graph_compile/quantscript_graph/formal_module_conversion/intent_lowering/double_ma_lowering.rs
```

Parent file:

```text
src/backend/graph_compile/quantscript_graph/formal_module_conversion/intent_lowering.rs
```

Parent module declaration:

```rust
mod double_ma_lowering;
```

Parent `builtin.intent.double_ma` branch now only keeps the controlled call:

```rust
"builtin.intent.double_ma" => {
    double_ma_lowering::append_double_ma_lowering_lines(
        cfg,
        &source_var,
        instrument,
        qs_lines,
    );
}
```

Child helper:

```rust
pub(super) fn append_double_ma_lowering_lines(
    cfg: &Value,
    source_var: &str,
    instrument: &str,
    qs_lines: &mut Vec<String>,
)
```

Extraction markers:

```text
double_ma_lowering actual_extraction_done
double_ma_lowering plan_frozen
double_ma_lowering baseline_frozen
```

---

## Equivalence Preserved

This batch only moves `builtin.intent.double_ma`; it does not change fallback values, QS line text, guard condition, or BUY emit order:

```text
builtin.intent.double_ma
fast_period default 20
slow_period default 50
let fast = sma({}, {})
let slow = sma({}, {})
fast > slow
emit Intent("BUY", instrument="{}", quantity=1.0)
```

QS order remains:

```text
let fast = sma(...)
let slow = sma(...)
if fast > slow {
emit Intent("BUY", instrument="{}", quantity=1.0)
}
```

The parent still resolves `module_key`, `cfg`, `instrument`, `node_id`, `upstream_edge`, `source_id`, and `source_var`. The child consumes only explicit parent inputs.

---

## Parent Child Communication

The only new allowed connection is:

```text
intent_lowering -> double_ma_lowering
```

Existing allowed connections remain:

```text
formal_module_conversion -> intent_lowering
intent_lowering -> spread_observer_lowering
intent_lowering -> macd_lowering
```

No new forbidden connection was added:

```text
formal_module_conversion -> double_ma_lowering
compile_api -> double_ma_lowering
graph_quantscript_api -> double_ma_lowering
graph_api -> double_ma_lowering
runtime sibling -> double_ma_lowering
frontend -> double_ma_lowering
sibling horizontal link
```

release transition guard: no developer release-transition decision exists, so this batch does not bypass parent-child communication for performance.

---

## Not Moved

This batch does not move:

1. `shared_intent_context`
2. `builtin.intent.rsi`
3. `builtin.intent.ma_deviation`
4. `builtin.intent.macd` or `macd_lowering`
5. `builtin.intent.momentum`
6. `builtin.intent.zscore`
7. `builtin.intent.spread_observer` or `spread_observer_lowering`
8. unsupported intent `anyhow::bail!`
9. `formal_module_conversion.rs`, route surface, parser, artifact target projection, frontend caller, or runtime caller
10. release transition

---

## Verification Gates

Run before commit:

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

## Next Boundary

Next step can only be:

```text
BE-001GB-04
backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.double_ma_lowering
root.backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.double_ma_lowering
```

BE-001GB-04 can only perform single leaf closeout and decide whether `double_ma_lowering` should split further. It must not move `rsi`, `ma_deviation`, `momentum`, `zscore`, shared context, unsupported failure, or release transition.

---

## Hallucination Checks

When claiming BE-001GB-03 is complete, state only:

1. `double_ma_lowering actual_extraction_done` is true.
2. `src/backend/graph_compile/quantscript_graph/formal_module_conversion/intent_lowering/double_ma_lowering.rs` exists.
3. Parent only keeps `mod double_ma_lowering;` and `double_ma_lowering::append_double_ma_lowering_lines(...)`.
4. Other built-in intent branches, shared context, unsupported failure, and release transition were not moved.
5. Do not claim `double_ma_lowering` closeout, `intent_lowering` closeout, `formal_module_conversion` closeout, `backend.graph_compile` closeout, `backend` closeout, or Rust restructuring completion.

---

## Acceptance Criteria

1. This file is covered by milestone index, module tree, full feature tree, and governance gates.
2. `double_ma_lowering actual_extraction_done` is recorded.
3. The new child file is covered by the full feature tree.
4. Parent-child communication only uses `intent_lowering -> double_ma_lowering`.
5. Next step is fixed to BE-001GB-04 single leaf closeout.
6. Governance gates, full feature tree coverage, UTF-8, Rust fmt/check, MACD targeted test, and `git diff --check` all pass.
