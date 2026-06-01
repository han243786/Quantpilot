# v4.16.0 backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.ma_deviation_lowering baseline plan

> Version type: MINOR architecture / governance
> Execution tier: lightweight
> Batch: BE-001GF-01
> Node: `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.ma_deviation_lowering`
> Parent: `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering`
> Stage: `baseline_plan`
> Movement: no code movement
> Speed protocol: `lightweight_two_step`
> Next step: BE-001GF-02 `ma_deviation_lowering` `extract_closeout`

---

## Matrix Impact

| Matrix | Impact node | Change type |
| --- | --- | --- |
| Flow matrix | BE-001GF-01 baseline + plan | lightweight two-step stage 1 |
| Norm matrix | parent-child communication / equivalence baseline | branch freeze |
| Guidance matrix | `root.backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.ma_deviation_lowering` | planned child white-box node |
| Module tree | `intent_lowering -> ma_deviation_lowering` | planned child edge |

---

## Equivalence Baseline

Freeze the current `builtin.intent.ma_deviation` branch exactly as implemented in:

```text
src/backend/graph_compile/quantscript_graph/formal_module_conversion/intent_lowering.rs
```

Frozen input surface:

```text
cfg
source_var
instrument
qs_lines
```

Frozen config defaults:

```text
lookback default 15
baseline_period default 150
```

Frozen output lines:

```text
let ma_dev = sma(source_var, lookback) / sma(source_var, baseline_period)
if ma_dev > 1 {
emit Intent("SELL", instrument="{instrument}", quantity=1.0)
}
```

No route, schema, persistence, lock owner, public API, or state-machine behavior changes are allowed in this batch.

---

## Extraction Plan

Planned child file:

```text
src/backend/graph_compile/quantscript_graph/formal_module_conversion/intent_lowering/ma_deviation_lowering.rs
```

Planned parent additions:

```rust
mod ma_deviation_lowering;
ma_deviation_lowering::append_ma_deviation_lowering_lines(cfg, &source_var, instrument, qs_lines);
```

Planned helper:

```rust
pub(super) fn append_ma_deviation_lowering_lines(
    cfg: &Value,
    source_var: &str,
    instrument: &str,
    qs_lines: &mut Vec<String>,
)
```

The parent keeps ownership of:

```text
node iteration
module_key dispatch
upstream edge lookup
source_var resolution
instrument fallback
unsupported intent failure
```

The child owns only:

```text
builtin.intent.ma_deviation branch rendering
lookback/baseline_period local defaults
SELL intent output lines
```

---

## Speed Protocol Fit

`lightweight_two_step` is valid here:

1. `baseline_plan` freezes the branch and the planned helper in one document.
2. `extract_closeout` can move the branch and close the child in one follow-up document.
3. Inner fragments such as config decode, `ma_dev` render, and SELL emit are micro-leaves without independent owners.
4. The only allowed edge is `intent_lowering -> ma_deviation_lowering`.

---

## Equivalence Gates

The following gates are sufficient for this stage:

```powershell
git diff --check
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
cargo fmt --check
cargo check -p quantpilot
```

The next `extract_closeout` should additionally run the most relevant formal compile targeted test available for built-in intent lowering.
