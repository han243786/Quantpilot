# v4.16.0 backend.graph_compile.quantscript_graph.formal_module_conversion.profile_lowering baseline plan

> Batch: BE-001GS-01
> Node: `backend.graph_compile.quantscript_graph.formal_module_conversion.profile_lowering`
> Parent: `backend.graph_compile.quantscript_graph.formal_module_conversion`
> Stage: `baseline_plan`
> Movement: no code movement.
> Speed protocol: `lightweight_two_step`
> Next step: BE-001GS-02 `profile_lowering` `extract_closeout`

---

## Summary

This baseline freezes the risk/execution profile lowering currently embedded in
`formal_module_conversion.rs`. The next movement may extract only the
profile-rendering branch pair and preserve the current conversion order.

---

## Matrix Impact

| Matrix | Impact node | Change type |
| --- | --- | --- |
| Flow matrix | BE-001GS-01 baseline + plan | lightweight two-step stage 1 |
| Norm matrix | parent-child communication / equivalence baseline | profile lowering freeze |
| Guidance matrix | `root.backend.graph_compile.quantscript_graph.formal_module_conversion.profile_lowering` | planned child white-box node |
| Module tree | `formal_module_conversion -> profile_lowering` | planned child edge |

---

## Equivalence Baseline

Frozen owner:

```text
src/backend/graph_compile/quantscript_graph/formal_module_conversion.rs
```

Frozen risk trigger:

```text
node.type == "risk"
```

Frozen risk config defaults:

```text
profile = config.profile_id or config.profile_name or "global"
max_position = config.max_position f64 or 0.2
max_total_leverage = config.max_total_leverage f64 or 3.0
max_exchange_leverage = config.max_exchange_leverage f64 or 3.0
min_action_interval_ms = config.min_action_interval_ms u64 or 100
```

Frozen risk output:

```text
    risk.profile("{profile}", max_position={max_pos}, max_total_leverage={max_lev}, max_exchange_leverage={max_exchange_lev}, min_action_interval_ms={min_interval})
```

Frozen execution trigger:

```text
node.type == "execution"
```

Frozen execution config defaults:

```text
profile = config.profile_id or config.profile_name or config.mode or "paper"
fee_bps = config.fee_bps f64 or 10.0
slippage_bps = config.slippage_bps f64 or 5.0
```

Frozen execution output:

```text
    execution.profile("{profile}", fee_bps={fee}, slippage_bps={slip})
```

Frozen non-goals:

```text
No graph.nodes / graph.edges validation movement.
No data source lowering movement.
No intent lowering movement.
No unsupported node logging movement.
No terminal parse movement.
No public API, route, schema, persistence, lock owner, or state-machine change.
No sibling module call.
No release-transition optimization.
```

---

## Extraction Plan

Planned child file:

```text
src/backend/graph_compile/quantscript_graph/formal_module_conversion/profile_lowering.rs
```

Planned parent additions:

```rust
mod profile_lowering;

profile_lowering::append_profile_lowering_lines(nodes, &mut qs_lines);
```

Planned child surface:

```rust
pub(super) fn append_profile_lowering_lines(nodes: &[Value], qs_lines: &mut Vec<String>)
```

The parent keeps ownership of:

```text
graph shape validation
overall QS source assembly order
data_source_lowering call
intent_lowering call
unsupported node logging until it is selected separately
terminal parse
```

The child owns only:

```text
risk node profile lowering
execution node profile lowering
profile defaults
formal profile QS line rendering
```

---

## Speed Protocol Fit

`lightweight_two_step` is valid:

1. The child stays local to one parent and one planned file.
2. No API, schema, persistence, or lock owner changes.
3. Parent-child communication remains one-way.
4. A direct local unit test can freeze risk/execution line output, and compile
   endpoint tests can protect the formal conversion path.

## Boundary

**Real files**:
- `src/backend/graph_compile/quantscript_graph/formal_module_conversion.rs`
- `src/backend/graph_compile/quantscript_graph/formal_module_conversion/profile_lowering.rs` (planned)

**Markers**:
- `profile_lowering baseline_plan`

**Next step**:
BE-001GS-02 profile_lowering extract_closeout

---

## Gates

- `git diff --check`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p quantpilot profile_lowering`
- `cargo test -p quantpilot formal_quantscript_compile_endpoint_matches_one_sided_zscore_golden_view`
