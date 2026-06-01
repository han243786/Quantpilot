# v4.16.0 backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.momentum_lowering baseline plan

> Version type: MINOR architecture / governance
> Execution tier: lightweight
> Batch: BE-001GH-01
> Node: `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.momentum_lowering`
> Parent: `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering`
> Stage: `baseline_plan`
> Movement: no code movement
> Speed protocol: `lightweight_two_step`
> Next step: BE-001GH-02 `momentum_lowering` `extract_closeout`

---

## Matrix Impact

| Matrix | Impact node | Change type |
| --- | --- | --- |
| Flow matrix | BE-001GH-01 baseline + plan | lightweight two-step stage 1 |
| Norm matrix | parent-child communication / equivalence baseline | branch freeze |
| Guidance matrix | `root.backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.momentum_lowering` | planned child white-box node |
| Module tree | `intent_lowering -> momentum_lowering` | planned child edge |

---

## Equivalence Baseline

Freeze the current `builtin.intent.momentum` branch exactly as implemented in:

```text
src/backend/graph_compile/quantscript_graph/formal_module_conversion/intent_lowering.rs
```

Frozen input surface:

```text
node_id
cfg
source_var
instrument
qs_lines
```

Frozen config defaults and fallback:

```text
lookback default 10
threshold_ratio preferred
threshold fallback
threshold default 0.02
```

Frozen output lines:

```text
let {node_id}_signal = momentum(source_var, lookback)
if {node_id}_signal > threshold {
emit Intent("BUY", instrument="{instrument}", quantity=1.0)
}
```

No route, schema, persistence, lock owner, public API, or state-machine behavior changes are allowed in this batch.

---

## Extraction Plan

Planned child file:

```text
src/backend/graph_compile/quantscript_graph/formal_module_conversion/intent_lowering/momentum_lowering.rs
```

Planned parent additions:

```rust
mod momentum_lowering;
momentum_lowering::append_momentum_lowering_lines(node_id, cfg, &source_var, instrument, qs_lines);
```

Planned helper:

```rust
pub(super) fn append_momentum_lowering_lines(
    node_id: &str,
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
builtin.intent.momentum branch rendering
lookback and threshold fallback
momentum signal line
BUY intent output lines
```

---

## Speed Protocol Fit

`lightweight_two_step` is valid here:

1. `baseline_plan` freezes the branch and helper in one document.
2. `extract_closeout` can move the branch and close the child in one follow-up document.
3. Inner fragments such as config decode, signal render, and BUY emit are micro-leaves without independent owners.
4. The only allowed edge is `intent_lowering -> momentum_lowering`.

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

The next `extract_closeout` should additionally run a formal momentum targeted test.
