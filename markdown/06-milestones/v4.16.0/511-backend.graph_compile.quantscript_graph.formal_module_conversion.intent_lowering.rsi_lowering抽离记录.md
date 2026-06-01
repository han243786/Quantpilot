# v4.16.0 backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.rsi_lowering actual extraction record
> Version type: MINOR architecture / governance
> Execution tier: standard
> Batch: BE-001GD-03
> Baseline: `510-backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.rsi_lowering抽离方案.md`
> Target child: `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.rsi_lowering`
> Judgment: actual extraction complete
> Module tree coordinate: `root.backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.rsi_lowering`
> Code action: actual extraction
> Next step: BE-001GD-04 `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.rsi_lowering` single leaf closeout

---

## Matrix Impact

| Matrix | Impact node | Change type |
| --- | --- | --- |
| Flow matrix | BE-001GD-03 `rsi_lowering` actual extraction record | code extraction |
| Norm matrix | actual extraction / parent-child communication / equivalence preservation / release transition guard | standard tier |
| Guidance matrix | `root.backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.rsi_lowering` | child file landed |
| Module tree | `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.rsi_lowering` | actual_extraction_done |

---

## Actual Changes

New child file:

```text
src/backend/graph_compile/quantscript_graph/formal_module_conversion/intent_lowering/rsi_lowering.rs
```

Parent file:

```text
src/backend/graph_compile/quantscript_graph/formal_module_conversion/intent_lowering.rs
```

Parent module declaration:

```rust
mod rsi_lowering;
```

Parent `builtin.intent.rsi` branch now only keeps the controlled call:

```rust
"builtin.intent.rsi" => {
    rsi_lowering::append_rsi_lowering_lines(
        node_id,
        cfg,
        &source_var,
        instrument,
        qs_lines,
    );
}
```

Child helper:

```rust
pub(super) fn append_rsi_lowering_lines(
    node_id: &str,
    cfg: &Value,
    source_var: &str,
    instrument: &str,
    qs_lines: &mut Vec<String>,
)
```

Extraction markers:

```text
rsi_lowering actual_extraction_done
rsi_lowering plan_frozen
rsi_lowering baseline_frozen
```

---

## Equivalence Preserved

This batch only moves `builtin.intent.rsi`. It does not change fallback values, QS line text, guard condition, or BUY emit order:

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

QS order remains:

```text
let {node_id}_signal = rsi(source_var, period)
if {node_id}_signal < oversold {
emit Intent("BUY", instrument="...", quantity=1.0)
}
```

The parent still resolves `module_key`, `cfg`, `instrument`, `node_id`, `upstream_edge`, `source_id`, and `source_var`. The child consumes only explicit parent inputs.

---

## Parent Child Communication

The only new allowed connection is:

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

No forbidden connection was added:

```text
formal_module_conversion -> rsi_lowering
compile_api -> rsi_lowering
graph_quantscript_api -> rsi_lowering
graph_api -> rsi_lowering
runtime sibling -> rsi_lowering
frontend -> rsi_lowering
sibling horizontal link
```

release transition guard: no developer release-transition decision exists, so this batch does not bypass parent-child communication for performance.

---

## Not Moved

This batch does not move:

1. `shared_intent_context`
2. `builtin.intent.double_ma` or `double_ma_lowering`
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
BE-001GD-04
backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.rsi_lowering
root.backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.rsi_lowering
```

BE-001GD-04 can only perform single leaf closeout and decide whether `rsi_lowering` should split further. It must not move `ma_deviation`, `momentum`, `zscore`, shared context, unsupported failure, or release transition.

---

## Hallucination Checks

When claiming BE-001GD-03 is complete, state only:

1. `rsi_lowering actual_extraction_done` is true.
2. `src/backend/graph_compile/quantscript_graph/formal_module_conversion/intent_lowering/rsi_lowering.rs` exists.
3. Parent only keeps `mod rsi_lowering;` and `rsi_lowering::append_rsi_lowering_lines(...)`.
4. Other built-in intent branches, shared context, unsupported failure, and release transition were not moved.
5. Do not claim `rsi_lowering` closeout, `intent_lowering` closeout, `formal_module_conversion` closeout, `backend.graph_compile` closeout, `backend` closeout, or Rust restructuring completion.

---

## Acceptance Criteria

1. This file is covered by milestone index, module tree, full feature tree, and governance gates.
2. `rsi_lowering actual_extraction_done` is recorded.
3. The new child file is covered by the full feature tree.
4. Parent-child communication only uses `intent_lowering -> rsi_lowering`.
5. Next step is fixed to BE-001GD-04 single leaf closeout.
6. Governance gates, full feature tree coverage, UTF-8, Rust fmt/check, MACD targeted test, and `git diff --check` all pass.
