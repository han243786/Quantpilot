# v4.16.0 backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.shared_intent_context baseline plan

> Version type: MINOR architecture / governance
> Execution tier: lightweight
> Batch: BE-001GL-01
> Node: `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.shared_intent_context`
> Parent: `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering`
> Stage: `baseline_plan`
> Movement: no code movement
> Speed protocol: `lightweight_two_step`
> Next step: BE-001GL-02 `shared_intent_context` `extract_closeout`

---

## Matrix Impact

| Matrix | Impact node | Change type |
| --- | --- | --- |
| Flow matrix | BE-001GL-01 baseline + plan | lightweight two-step stage 1 |
| Norm matrix | parent-child communication / equivalence baseline | shared context freeze |
| Guidance matrix | `root.backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.shared_intent_context` | planned child white-box node |
| Module tree | `intent_lowering -> shared_intent_context` | planned child edge |

---

## Equivalence Baseline

Freeze the shared context resolution currently owned by:

```text
src/backend/graph_compile/quantscript_graph/formal_module_conversion/intent_lowering.rs
```

Frozen input surface:

```text
node
edges
```

Frozen output fields:

```text
module_key
cfg
instrument
node_id
source_var
```

Frozen defaults and transforms:

```text
instrument default BTCUSDT
node_id default ""
source_id default data
source_var = source_id.replace(['-', '.'], "_")
```

No route, schema, persistence, lock owner, public API, or state-machine behavior changes are allowed in this batch.

---

## Extraction Plan

Planned child file:

```text
src/backend/graph_compile/quantscript_graph/formal_module_conversion/intent_lowering/shared_intent_context.rs
```

Planned parent additions:

```rust
mod shared_intent_context;
let ctx = shared_intent_context::resolve_intent_lowering_context(node, edges);
match ctx.module_key { ... }
```

Planned child surface:

```rust
pub(super) struct IntentLoweringContext<'a> {
    pub(super) module_key: &'a str,
    pub(super) cfg: &'a Value,
    pub(super) instrument: &'a str,
    pub(super) node_id: &'a str,
    pub(super) source_var: String,
}

pub(super) fn resolve_intent_lowering_context<'a>(
    node: &'a Value,
    edges: &'a [Value],
) -> IntentLoweringContext<'a>
```

The parent keeps ownership of:

```text
intent node iteration
module_key dispatch
child branch calls
unsupported intent failure
```

The child owns only:

```text
context extraction for one intent node
instrument fallback
upstream edge lookup
source var normalization
```

---

## Speed Protocol Fit

`lightweight_two_step` is valid here:

1. `baseline_plan` freezes the context struct and resolver in one document.
2. `extract_closeout` can create the child and close it in one follow-up document.
3. The helper has a stable owner and does not require sibling direct calls.
4. The only allowed edge is `intent_lowering -> shared_intent_context`.

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

The next `extract_closeout` should additionally run a formal built-in intent lowering targeted test.
