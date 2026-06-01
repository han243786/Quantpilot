# v4.16.0 backend.graph_compile.quantscript_graph.formal_module_conversion.input_shape_validation baseline plan

> Batch: BE-001GU-01
> Node: `backend.graph_compile.quantscript_graph.formal_module_conversion.input_shape_validation`
> Parent: `backend.graph_compile.quantscript_graph.formal_module_conversion`
> Stage: `baseline_plan`
> Movement: no code movement.
> Speed protocol: `lightweight_two_step`
> Next step: BE-001GU-02 `input_shape_validation` `extract_closeout`

---

## Summary

This baseline freezes the entry shape validation for formal graph conversion.
The next movement may extract only the required-array checks for `graph.nodes`
and `graph.edges`.

---

## Matrix Impact

| Matrix | Impact node | Change type |
| --- | --- | --- |
| Flow matrix | BE-001GU-01 baseline + plan | lightweight two-step stage 1 |
| Norm matrix | parent-child communication / entry failure ownership | input shape validation freeze |
| Guidance matrix | `root.backend.graph_compile.quantscript_graph.formal_module_conversion.input_shape_validation` | planned child white-box node |
| Module tree | `formal_module_conversion -> input_shape_validation` | planned child edge |

---

## Equivalence Baseline

Frozen owner:

```text
src/backend/graph_compile/quantscript_graph/formal_module_conversion.rs
```

Frozen valid output:

```rust
(&[Value], &[Value])
```

Frozen invalid `nodes` behavior:

```text
graph.nodes missing or not array -> anyhow error: graph.nodes 必须是数组
```

Frozen invalid `edges` behavior:

```text
graph.edges missing or not array -> anyhow error: graph.edges 必须是数组
```

Frozen ordering:

```text
nodes validation runs before edges validation
no lowering children run until both arrays are available
```

Frozen non-goals:

```text
No semantic validation of node contents.
No data_source_lowering movement.
No profile_lowering movement.
No intent_lowering movement.
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
src/backend/graph_compile/quantscript_graph/formal_module_conversion/input_shape_validation.rs
```

Planned parent additions:

```rust
mod input_shape_validation;

let (nodes, edges) = input_shape_validation::require_graph_nodes_and_edges(graph_value)?;
```

Planned child surface:

```rust
pub(super) fn require_graph_nodes_and_edges(graph_value: &Value) -> anyhow::Result<(&[Value], &[Value])>
```

The parent keeps ownership of:

```text
overall QS source assembly order
data_source_lowering call
profile_lowering call
unsupported_node_logging until selected separately
intent_lowering call
terminal parse
```

The child owns only:

```text
required graph.nodes array check
required graph.edges array check
borrowed nodes/edges slice return
shape error strings
```

---

## Speed Protocol Fit

`lightweight_two_step` is valid:

1. The child is a narrow entry guard with a clear failure owner.
2. Parent-child communication is one direct call returning borrowed slices.
3. Local unit tests can freeze success and both failure paths.
4. No route, schema, persistence, lock owner, public API, or state-machine change occurs.

## Boundary

**Real files**:
- `src/backend/graph_compile/quantscript_graph/formal_module_conversion.rs`
- `src/backend/graph_compile/quantscript_graph/formal_module_conversion/input_shape_validation.rs` (planned)

**Markers**:
- `input_shape_validation baseline_plan`

**Next step**:
BE-001GU-02 input_shape_validation extract_closeout

---

## Gates

- `git diff --check`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p quantpilot input_shape_validation`
- `cargo test -p quantpilot formal_quantscript_compile_endpoint_matches_one_sided_zscore_golden_view`
