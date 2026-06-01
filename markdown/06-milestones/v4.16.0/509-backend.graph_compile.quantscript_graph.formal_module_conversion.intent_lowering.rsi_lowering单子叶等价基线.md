# v4.16.0 backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.rsi_lowering single child equivalence baseline
> Version type: MINOR architecture / governance
> Execution tier: standard
> Batch: BE-001GD-01
> Baseline: `508-backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering父叶残余判断.md`
> Target child: `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.rsi_lowering`
> Judgment: single child equivalence baseline frozen
> Module tree coordinate: `root.backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.rsi_lowering`
> Code action: no code movement
> Next step: BE-001GD-02 `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.rsi_lowering` extraction plan

---

## Matrix Impact

| Matrix | Impact node | Change type |
| --- | --- | --- |
| Flow matrix | BE-001GD-01 `rsi_lowering` single child equivalence baseline | child baseline |
| Norm matrix | equivalence baseline / branch-level extraction guard / parent-child communication / release transition guard | standard tier |
| Guidance matrix | `root.backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.rsi_lowering` | new white-box child candidate |
| Module tree | `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.rsi_lowering` | baseline_frozen |

---

## Current Real Boundary

`rsi_lowering` is still an inline match branch in:

```text
src/backend/graph_compile/quantscript_graph/formal_module_conversion/intent_lowering.rs
builtin.intent.rsi
```

Planned child path for the later extraction plan:

```text
src/backend/graph_compile/quantscript_graph/formal_module_conversion/intent_lowering/rsi_lowering.rs
```

Baseline markers:

```text
rsi_lowering baseline_frozen
rsi_lowering_selected
intent_lowering stop_split: false
spread_observer_lowering stop_split: true
macd_lowering stop_split: true
double_ma_lowering stop_split: true
```

This batch does not create the child file and does not move Rust code.

---

## White Box Input Surface

The current branch depends on parent-prepared inputs:

| Input | Source | Meaning |
| --- | --- | --- |
| `node_id` | current intent node id | signal variable prefix |
| `cfg` | `node.config` | RSI period and oversold threshold |
| `source_var` | shared upstream resolution | RSI source variable |
| `instrument` | intent config / default `BTCUSDT` | emitted Intent instrument |
| `qs_lines` | parent QS line buffer | generated QuantScript line sink |

Candidate helper signature for BE-001GD-02:

```rust
pub(super) fn append_rsi_lowering_lines(
    node_id: &str,
    cfg: &Value,
    source_var: &str,
    instrument: &str,
    qs_lines: &mut Vec<String>,
)
```

BE-001GD-02 may refine only local return/error shape if needed. It must not add a sibling caller or bypass the `intent_lowering` parent.

---

## Equivalence Semantics Frozen

The later extraction must keep parameter lookup and fallback:

```text
period default 14
oversold_threshold
oversold
oversold default 30.0
```

It must keep the generated RSI line:

```text
let {}_signal = rsi({}, {})
```

It must keep guard and BUY emit:

```text
if {}_signal < {}
emit Intent("BUY", instrument="{}", quantity=1.0)
```

It must keep QS line order:

```text
let {node_id}_signal = rsi(source_var, period)
if {node_id}_signal < oversold {
emit Intent("BUY", instrument="...", quantity=1.0)
}
```

Required markers:

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

---

## Parent Child Rule

Later extraction can add only this connection:

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

Forbidden connections:

```text
formal_module_conversion -> rsi_lowering
compile_api -> rsi_lowering
graph_quantscript_api -> rsi_lowering
graph_api -> rsi_lowering
runtime sibling -> rsi_lowering
frontend -> rsi_lowering
sibling horizontal link
```

release transition guard: no developer release-transition decision exists, so performance cannot justify bypassing the parent-child rule.

---

## Non Goals

This baseline does not:

1. Create `rsi_lowering.rs`.
2. Modify `intent_lowering.rs`.
3. Extract shared intent context.
4. Extract `ma_deviation`, `momentum`, `zscore`, or unsupported intent failure.
5. Change `spread_observer_lowering`, `macd_lowering`, or `double_ma_lowering`.
6. Change `formal_module_conversion.rs`.
7. Change parser, route surface, artifact target projection, frontend caller, or runtime caller.
8. Start release transition.

---

## Regression Gates

Later actual extraction should run at least:

```text
cargo fmt
cargo check -p quantpilot
cargo test -p quantpilot formal_quantscript_compile_endpoint_returns_structured_lowering_diagnostic_for_missing_macd_source
```

This baseline must pass before commit:

```powershell
git diff --check
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
cargo fmt --check
cargo check -p quantpilot
```

---

## Next Boundary

Next step can only enter:

```text
BE-001GD-02
backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.rsi_lowering
root.backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.rsi_lowering
```

BE-001GD-02 can only write the extraction plan: planned child, parent `mod`, helper signature, allowed branch migration block, gates, and rollback point. It must not create the child file or move Rust.

---

## Hallucination Checks

When claiming BE-001GD-01 is complete, state only:

1. Current batch is a `no code movement` single child equivalence baseline.
2. `rsi_lowering baseline_frozen` is true, but the child file has not been created.
3. `builtin.intent.rsi` still lives in `intent_lowering.rs`.
4. Next step can only enter BE-001GD-02 extraction plan.
5. Do not claim `rsi_lowering` has been extracted.
6. Do not claim `intent_lowering`, `formal_module_conversion`, `backend.graph_compile`, `backend`, or Rust restructuring is closed.

---

## Acceptance Criteria

1. This baseline file is covered by milestone index, module tree, full feature tree, and governance gates.
2. `rsi_lowering baseline_frozen` is recorded.
3. Next step is fixed to BE-001GD-02 `rsi_lowering` extraction plan.
4. Governance gates, full feature tree coverage, UTF-8, Rust fmt/check, and `git diff --check` all pass.
