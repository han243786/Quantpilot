# v4.16.0 backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.rsi_lowering extraction plan
> Version type: MINOR architecture / governance
> Execution tier: standard
> Batch: BE-001GD-02
> Baseline: `509-backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.rsi_lowering单子叶等价基线.md`
> Target child: `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.rsi_lowering`
> Judgment: extraction plan frozen
> Module tree coordinate: `root.backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.rsi_lowering`
> Code action: no code movement
> Next step: BE-001GD-03 `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.rsi_lowering` actual extraction record

---

## Matrix Impact

| Matrix | Impact node | Change type |
| --- | --- | --- |
| Flow matrix | BE-001GD-02 `rsi_lowering` extraction plan | plan freeze |
| Norm matrix | plan freeze / branch move boundary / parent-child communication / rollback point | standard tier |
| Guidance matrix | `root.backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.rsi_lowering` | planned child file |
| Module tree | `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.rsi_lowering` | plan_frozen |

---

## Planned Change

BE-001GD-03 may only create this child:

```text
src/backend/graph_compile/quantscript_graph/formal_module_conversion/intent_lowering/rsi_lowering.rs
```

Parent file:

```text
src/backend/graph_compile/quantscript_graph/formal_module_conversion/intent_lowering.rs
```

Parent may only add this module declaration:

```rust
mod rsi_lowering;
```

Parent `builtin.intent.rsi` branch may only be replaced by a controlled call:

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

Planned helper:

```rust
pub(super) fn append_rsi_lowering_lines(
    node_id: &str,
    cfg: &Value,
    source_var: &str,
    instrument: &str,
    qs_lines: &mut Vec<String>,
)
```

Plan markers:

```text
rsi_lowering plan_frozen
rsi_lowering baseline_frozen
```

---

## Allowed Migration Block

BE-001GD-03 may only migrate this branch:

```rust
"builtin.intent.rsi" => {
    let period = cfg.get("period").and_then(|v| v.as_u64()).unwrap_or(14);
    let oversold = cfg
        .get("oversold_threshold")
        .or_else(|| cfg.get("oversold"))
        .and_then(Value::as_f64)
        .unwrap_or(30.0);
    qs_lines.push(format!(
        "    let {}_signal = rsi({}, {})",
        node_id, source_var, period
    ));
    qs_lines.push(format!("    if {}_signal < {} {{", node_id, oversold));
    qs_lines.push(format!(
        "        emit Intent(\"BUY\", instrument=\"{}\", quantity=1.0)",
        instrument
    ));
    qs_lines.push("    }".to_string());
}
```

Do not move shared context:

```text
module_key
cfg
instrument
node_id
upstream_edge
source_id
source_var
```

Do not move other branches:

```text
builtin.intent.double_ma
builtin.intent.ma_deviation
builtin.intent.macd
builtin.intent.momentum
builtin.intent.zscore
builtin.intent.spread_observer
unsupported intent
anyhow::bail!
```

---

## Equivalence Invariants

BE-001GD-03 must keep:

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

QS line order must remain:

```text
let {node_id}_signal = rsi(source_var, period)
if {node_id}_signal < oversold {
emit Intent("BUY", instrument="...", quantity=1.0)
}
```

---

## Parent Child Rule

BE-001GD-03 may add only:

```text
intent_lowering -> rsi_lowering
```

Existing allowed links remain:

```text
formal_module_conversion -> intent_lowering
intent_lowering -> spread_observer_lowering
intent_lowering -> macd_lowering
intent_lowering -> double_ma_lowering
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

release transition guard: current work is not in release transition. Do not use performance to bypass parent-child communication.

---

## Rollback Point

If BE-001GD-03 fails compile or tests, rollback only this child:

1. Delete `mod rsi_lowering;`.
2. Delete `rsi_lowering::append_rsi_lowering_lines(...)` call.
3. Restore `builtin.intent.rsi` branch into the parent match.
4. Delete planned child file.

Do not rollback `spread_observer_lowering`, `macd_lowering`, `double_ma_lowering`, or the `intent_lowering` parent extraction.

---

## Verification Gates

BE-001GD-03 must run at least:

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

Next step can only enter:

```text
BE-001GD-03
backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.rsi_lowering
root.backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.rsi_lowering
```

BE-001GD-03 can only create planned child, add parent `mod rsi_lowering;`, and move the `builtin.intent.rsi` branch. It must not touch `ma_deviation`, `momentum`, `zscore`, shared context, unsupported failure, or release transition.

---

## Hallucination Checks

When claiming BE-001GD-02 is complete, state only:

1. Current batch is a `no code movement` extraction plan.
2. `rsi_lowering plan_frozen` is true.
3. Planned child has not been created and `builtin.intent.rsi` has not moved.
4. Next step can only enter BE-001GD-03 actual extraction record.
5. Do not claim `rsi_lowering` has been extracted.
6. Do not claim `intent_lowering`, `formal_module_conversion`, `backend.graph_compile`, `backend`, or Rust restructuring is closed.

---

## Acceptance Criteria

1. This plan file is covered by milestone index, module tree, full feature tree, and governance gates.
2. `rsi_lowering plan_frozen` is recorded.
3. Next step is fixed to BE-001GD-03 `rsi_lowering` actual extraction record.
4. Governance gates, full feature tree coverage, UTF-8, Rust fmt/check, and `git diff --check` all pass.
