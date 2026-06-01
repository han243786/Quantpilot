# v4.16.0 backend.graph_compile.quantscript_graph.formal_module_conversion.data_source_lowering baseline plan

> Batch: BE-001GQ-01
> Node: `backend.graph_compile.quantscript_graph.formal_module_conversion.data_source_lowering`
> Parent: `backend.graph_compile.quantscript_graph.formal_module_conversion`
> Stage: `baseline_plan`
> Movement: no code movement.
> Speed protocol: `lightweight_two_step`
> Next step: BE-001GQ-02 `data_source_lowering` `extract_closeout`

---

## Summary

This baseline freezes the `data` node lowering branch currently embedded in
`formal_module_conversion.rs`. The next movement may extract only the data
branch and its `fetch(...)` rendering helper.

---

## Matrix Impact

| Matrix | Impact node | Change type |
| --- | --- | --- |
| Flow matrix | BE-001GQ-01 baseline + plan | lightweight two-step stage 1 |
| Norm matrix | parent-child communication / equivalence baseline | data source lowering freeze |
| Guidance matrix | `root.backend.graph_compile.quantscript_graph.formal_module_conversion.data_source_lowering` | planned child white-box node |
| Module tree | `formal_module_conversion -> data_source_lowering` | planned child edge |

---

## Equivalence Baseline

Frozen owner:

```text
src/backend/graph_compile/quantscript_graph/formal_module_conversion.rs
```

Frozen trigger:

```text
For each node where node.type == "data"
```

Frozen config defaults:

```text
config missing -> Value::Null
exchange default -> binance
instrument default -> BTCUSDT
timeframe default -> 1d
window_size accepts f64 >= 1.0, casts to u64, otherwise defaults to 200
node id default -> data
var_name = node_id.replace(['-', '.'], "_")
```

Frozen optional arguments:

```text
ping_enabled is appended only when config.ping_enabled is bool
request_interval_ms is appended only when config.request_interval_ms is u64
```

Frozen output shape:

```text
    let {var_name} = fetch("{instrument}", exchange="{exchange}", interval="{interval}", lookback={lookback}[, ping_enabled=...][, request_interval_ms=...])?
```

Frozen non-goals:

```text
No graph.nodes / graph.edges validation movement.
No risk.profile or execution.profile movement.
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
src/backend/graph_compile/quantscript_graph/formal_module_conversion/data_source_lowering.rs
```

Planned parent additions:

```rust
mod data_source_lowering;

data_source_lowering::append_data_source_lowering_lines(nodes, &mut qs_lines);
```

Planned child surface:

```rust
pub(super) fn append_data_source_lowering_lines(nodes: &[Value], qs_lines: &mut Vec<String>)
```

The parent keeps ownership of:

```text
graph shape validation
overall QS source assembly order
profile lowering
intent lowering call
terminal parse
```

The child owns only:

```text
data node iteration
data config defaulting
fetch argument construction
data variable name normalization
fetch line rendering
```

---

## Speed Protocol Fit

`lightweight_two_step` is valid:

1. The branch is local to one parent and one planned child.
2. No public interface or persistence owner changes.
3. Parent-child communication remains one-way.
4. Existing formal compile tests cover generated QS behavior; extraction will
   additionally run a focused compile endpoint test that includes data lowering.

## Boundary

**Real files**:
- `src/backend/graph_compile/quantscript_graph/formal_module_conversion.rs`
- `src/backend/graph_compile/quantscript_graph/formal_module_conversion/data_source_lowering.rs` (planned)

**Markers**:
- `data_source_lowering baseline_plan`

**Next step**:
BE-001GQ-02 data_source_lowering extract_closeout

---

## Gates

- `git diff --check`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p quantpilot formal_quantscript_compile_endpoint_matches_one_sided_zscore_golden_view`
