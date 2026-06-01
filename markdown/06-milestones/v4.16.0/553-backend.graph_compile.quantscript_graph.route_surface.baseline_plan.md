# v4.16.0 route_surface equivalence baseline and extraction plan

> Batch: BE-001HF-01
> Node: `backend.graph_compile.quantscript_graph.route_surface`
> Parent: `backend.graph_compile.quantscript_graph`
> Stage: `baseline_plan`
> Movement: no code movement.
> Speed protocol: `lightweight_two_step`
> Next step: BE-001HF-02 `route_surface` `extract_closeout`

---

## Summary

This baseline freezes the final local route facade inside
`backend.graph_compile.quantscript_graph`. The next movement may extract only
route registration and HTTP handlers into `route_surface.rs`; parser, artifact,
generation, and formal conversion helpers stay behind parent wrappers.

---

## Matrix Impact

| Matrix | Impact node | Change type |
| --- | --- | --- |
| Flow matrix | BE-001HF-01 baseline + plan | lightweight two-step stage 1 |
| Norm matrix | parent-child communication / route facade boundary | route surface freeze |
| Guidance matrix | `root.backend.graph_compile.quantscript_graph.route_surface` | planned child white-box node |
| Module tree | `quantscript_graph -> route_surface` | planned child edge |

---

## Equivalence Baseline

Frozen owner:

```text
src/backend/graph_compile/quantscript_graph.rs
```

Frozen route surface:

```text
GET /api/graphs/:graph_id/quantscript
POST /api/quantscript/graph/parse
```

Frozen handlers:

```text
register_routes
load_graph_quantscript
parse_graph_quantscript
```

Frozen behavior:

```text
Route registration keeps the same paths and methods.
Load route still validates graph_id, reads `{graph_id}.qs` from AppState.graph_store_dir, and maps IO failure through not_found_io_error.
Parse route still calls the parent `parse_graph_quantscript_source` wrapper and maps parser failures to json_bad_request("bad_request", "strategy_graph QuantScript ...").
No helper child is called directly from outside the parent boundary.
```

Frozen non-goals:

```text
No parser movement.
No artifact projection movement.
No graph-to-QS generation movement.
No formal module conversion movement.
No public API path, schema, persistence, lock owner, or state-machine change.
No sibling module call.
No release-transition optimization.
```

---

## Extraction Plan

Planned child file:

```text
src/backend/graph_compile/quantscript_graph/route_surface.rs
```

Planned parent additions:

```rust
mod route_surface;

pub(crate) fn register_routes(router: Router<AppState>) -> Router<AppState> {
    route_surface::register_routes(router)
}
```

Planned child surface:

```rust
pub(super) fn register_routes(router: Router<AppState>) -> Router<AppState>
```

The child owns:

```text
route registration
load_graph_quantscript handler
parse_graph_quantscript handler
route-local error mapping
```

The parent keeps ownership of:

```text
public register_routes wrapper
parse_graph_quantscript_source
attach_quantscript_artifacts
build_compile_runtime_targets_from_graph
child module mediation
```

---

## Speed Protocol Fit

`lightweight_two_step` is valid:

1. This is the final route facade residual of the parent.
2. Route paths, methods, and response mapping remain unchanged.
3. Child only calls parent helper wrappers; no sibling import is needed.
4. Existing route parse coverage and compile graph coverage can freeze behavior.

## Boundary

**Real files**:
- `src/backend/graph_compile/quantscript_graph.rs`
- `src/backend/graph_compile/quantscript_graph/route_surface.rs` (planned)

**Markers**:
- `route_surface baseline_frozen`
- `route_surface plan_frozen`

**Next step**:
BE-001HF-02 route_surface extract_closeout

---

## Gates

- `git diff --check`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p quantpilot formal_quantscript_source_cannot_be_parsed_as_strategy_graph_source`
- `cargo test -p quantpilot formal_quantscript_text_to_core_ir_to_graph_round_trip_sample`
