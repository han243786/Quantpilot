# v4.16.0 strategy_graph_parser equivalence baseline and extraction plan

> Batch: BE-001HB-01
> Node: `backend.graph_compile.quantscript_graph.strategy_graph_parser`
> Parent: `backend.graph_compile.quantscript_graph`
> Stage: `baseline_plan`
> Movement: no code movement.
> Speed protocol: `lightweight_two_step`
> Next step: BE-001HB-02 `strategy_graph_parser` `extract_closeout`

---

## Summary

This baseline freezes the route-facing `strategy_graph` QuantScript parser.
The next movement may extract only the source parser body and its private parse
helpers into `strategy_graph_parser.rs`; the parent keeps the public wrapper and
artifact attachment.

---

## Matrix Impact

| Matrix | Impact node | Change type |
| --- | --- | --- |
| Flow matrix | BE-001HB-01 baseline + plan | lightweight two-step stage 1 |
| Norm matrix | parent-child communication / parser failure boundary | strategy graph parser freeze |
| Guidance matrix | `root.backend.graph_compile.quantscript_graph.strategy_graph_parser` | planned child white-box node |
| Module tree | `quantscript_graph -> strategy_graph_parser` | planned child edge |

---

## Equivalence Baseline

Frozen owner:

```text
src/backend/graph_compile/quantscript_graph.rs
```

Frozen public parent entry:

```rust
pub(crate) fn parse_graph_quantscript_source(source: &str) -> anyhow::Result<Value>
```

Frozen parser helpers:

```text
parse_qs_scalar
parse_qs_node_header
parse_qs_connect
```

Frozen behavior:

```text
Line normalization keeps tab-to-four-spaces replacement.
Empty lines and # comments remain ignored.
Header must start with `strategy_graph ` and contain ` {`.
Header metadata still defaults name/version/mode to Imported Strategy/1.0.0/paper.
Node headers still accept only runtime/execution/plugin forms.
Runtime node config still receives inherited `mode`.
Connections still parse only `connect a.port -> b.port`.
Imported graph metadata, validation_state, compile_summary, node runtime_state, and artifact attachment remain equivalent.
```

Frozen non-goals:

```text
No route surface movement.
No graph-to-QS generation movement.
No formal module conversion movement.
No artifact target projection movement.
No runtime target helper movement.
No public API, schema, persistence, lock owner, or state-machine change.
No sibling module call.
No release-transition optimization.
```

---

## Extraction Plan

Planned child file:

```text
src/backend/graph_compile/quantscript_graph/strategy_graph_parser.rs
```

Planned parent additions:

```rust
mod strategy_graph_parser;

pub(crate) fn parse_graph_quantscript_source(source: &str) -> anyhow::Result<Value> {
    let now = current_time_ms();
    let mut graph = strategy_graph_parser::parse_strategy_graph_source(source, now)?;
    attach_quantscript_artifacts(&mut graph, source, now, std::path::Path::new(""));
    Ok(graph)
}
```

Planned child surface:

```rust
pub(super) fn parse_strategy_graph_source(source: &str, now: u64) -> anyhow::Result<Value>
```

The parent keeps ownership of:

```text
route error mapping
current timestamp acquisition
artifact attachment
public `parse_graph_quantscript_source` wrapper
```

The child owns only:

```text
source line normalization
strategy_graph header parse
metadata scalar parse
node section parse
connection section parse
imported graph value assembly before artifact attachment
```

---

## Speed Protocol Fit

`lightweight_two_step` is valid:

1. The planned child has one public-to-parent parse surface.
2. The movement does not alter output shape; parent attaches artifacts after child parse.
3. No sibling child call is required.
4. Existing round-trip tests can freeze parser equivalence.

## Boundary

**Real files**:
- `src/backend/graph_compile/quantscript_graph.rs`
- `src/backend/graph_compile/quantscript_graph/strategy_graph_parser.rs` (planned)

**Markers**:
- `strategy_graph_parser baseline_frozen`
- `strategy_graph_parser plan_frozen`

**Next step**:
BE-001HB-02 strategy_graph_parser extract_closeout

---

## Gates

- `git diff --check`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p quantpilot formal_quantscript_text_to_core_ir_to_graph_round_trip_sample`
- `cargo test -p quantpilot formal_quantscript_source_cannot_be_parsed_as_strategy_graph_source`
