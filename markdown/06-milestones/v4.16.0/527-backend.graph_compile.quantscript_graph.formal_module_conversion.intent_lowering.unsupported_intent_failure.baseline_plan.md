# v4.16.0 backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.unsupported_intent_failure baseline plan

> Batch: BE-001GN-01
> Node: `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.unsupported_intent_failure`
> Parent: `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering`
> Stage: `baseline_plan`
> Movement: no code movement
> Speed protocol: `lightweight_two_step`
> Next step: BE-001GN-02 `unsupported_intent_failure` `extract_closeout`

---

## Summary

This baseline freezes the final unsupported intent failure branch under
`intent_lowering`. The next movement may extract only the failure helper and its
message owner; supported intent dispatch must remain unchanged.

---

## Matrix Impact

| Matrix | Impact node | Change type |
| --- | --- | --- |
| Flow matrix | BE-001GN-01 baseline + plan | lightweight two-step stage 1 |
| Norm matrix | parent-child communication / hard failure ownership | unsupported intent failure freeze |
| Guidance matrix | `root.backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.unsupported_intent_failure` | planned child white-box node |
| Module tree | `intent_lowering -> unsupported_intent_failure` | planned child edge |

---

## Equivalence Baseline

Frozen owner:

```text
src/backend/graph_compile/quantscript_graph/formal_module_conversion/intent_lowering.rs
```

Frozen trigger:

```text
`match ctx.module_key` default branch for an intent node
```

Frozen output behavior:

```text
anyhow::bail!(
    "不支持的意图模块 '{}': 当前版本仅支持 {}。请升级到支持该模块的版本。",
    ctx.module_key,
    "double_ma/ma_deviation/rsi/macd/momentum/zscore/spread_observer"
)
```

Frozen non-goals:

```text
No supported intent branch change.
No source context resolution change.
No route, schema, persistence, lock owner, public API, or state-machine change.
No sibling module call.
No release-transition optimization.
```

---

## Extraction Plan

Planned child file:

```text
src/backend/graph_compile/quantscript_graph/formal_module_conversion/intent_lowering/unsupported_intent_failure.rs
```

Planned parent additions:

```rust
mod unsupported_intent_failure;

_ => {
    unsupported_intent_failure::bail_unsupported_intent(ctx.module_key)?;
}
```

Planned child surface:

```rust
pub(super) fn bail_unsupported_intent(module_key: &str) -> anyhow::Result<()>
```

The parent keeps ownership of:

```text
intent node iteration
context resolution
module_key dispatch
supported branch calls
```

The child owns only:

```text
supported intent display string
unsupported intent diagnostic construction
hard failure return
```

---

## Speed Protocol Fit

`lightweight_two_step` remains valid:

1. The baseline and plan are local to one parent branch.
2. The extraction is a child helper with one parent-to-child edge.
3. The failure path has an independent owner, but no broader API or schema
   blast radius.
4. A direct unit test for the helper can freeze the exact diagnostic while the
   existing zscore golden-view test verifies supported branch behavior remains
   unchanged.

## Boundary

**Real files**:
- `src/backend/graph_compile/quantscript_graph/formal_module_conversion/intent_lowering.rs`
- `src/backend/graph_compile/quantscript_graph/formal_module_conversion/intent_lowering/unsupported_intent_failure.rs` (planned)

**Markers**:
- `unsupported_intent_failure baseline_plan`

**Next step**:
BE-001GN-02 unsupported_intent_failure extract_closeout

---

## Gates

- `git diff --check`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p quantpilot unsupported_intent_failure`
- `cargo test -p quantpilot formal_quantscript_compile_endpoint_matches_one_sided_zscore_golden_view`
