# v4.16.0 route_surface actual extraction and closeout complete

> Batch: BE-001HF-02
> Node: `backend.graph_compile.quantscript_graph.route_surface`
> Parent: `backend.graph_compile.quantscript_graph`
> Stage: `extract_closeout`
> Movement: actual extraction + single leaf closeout.
> Speed protocol: `lightweight_two_step`
> Next step: BE-001HG-01 `quantscript_graph` parent closeout

---

## Summary

`route_surface` has been extracted from `quantscript_graph.rs` into a direct
child module. The parent now keeps only the public `register_routes` wrapper,
while the child owns route registration and the two HTTP handlers.

---

## Movement

Created:

```text
src/backend/graph_compile/quantscript_graph/route_surface.rs
```

Updated parent:

```rust
mod route_surface;

pub(crate) fn register_routes(router: Router<AppState>) -> Router<AppState> {
    route_surface::register_routes(router)
}
```

Final child surface:

```rust
pub(super) fn register_routes(router: Router<AppState>) -> Router<AppState>
```

Moved into child:

```text
load_graph_quantscript
parse_graph_quantscript
```

---

## Equivalence

Unchanged behavior:

```text
Route paths and methods remain unchanged.
Load route still validates graph id, reads `{graph_id}.qs`, and maps missing IO through not_found_io_error.
Parse route still calls parent `parse_graph_quantscript_source` and returns json_bad_request("bad_request", ...).
Parser, artifact projection, graph generation, formal conversion, public API path, schema, persistence, lock owner, and state-machine behavior were not moved.
No child-to-child sibling horizontal connection was introduced.
```

---

## leaf_split_decision_gate

leaf_split_base_gate

| Rule | Result | Evidence |
| --- | --- | --- |
| white_box_boundary_named | yes | Child owns route registration, load handler, parse handler, and route-local error mapping. |
| parent_child_communication_kept | yes | Parent calls child route registration; child calls parent parser wrapper and does not import parser/artifact/generation/formal children. |
| equivalence_baseline_freezable | yes | Route parse error coverage and graph round-trip coverage passed after movement. |

leaf_split_positive_trigger

| Trigger | Result | Evidence |
| --- | --- | --- |
| public_or_handler_boundary | yes | This is the route facade boundary. |
| state_machine_phase | no | HTTP route handling is not runtime state transition logic. |
| strategy_branch | no | Strategy parser/generation/conversion remain separate children. |
| independent_failure_mode | yes | Load not-found and parse bad-request mapping stay route-local. |
| reuse_pressure | yes | App router assembly continues through the parent `register_routes` wrapper. |

leaf_split_stop_condition

| Stop condition | Result | Evidence |
| --- | --- | --- |
| micro_leaf_without_owner | no | Route facade is the final stable owner. |
| communication_cost_rises | no | One parent wrapper delegates to one child route registration. |
| local_proof_missing | no | Focused route/round-trip tests and compile gate passed. |
| line_count_only | no | Split is justified by final public route boundary. |

leaf_split_decision_result

stop_split_true

next_recursive_step

BE-001HG-01 quantscript_graph parent closeout

## Boundary

**Real files**:
- `src/backend/graph_compile/quantscript_graph.rs`
- `src/backend/graph_compile/quantscript_graph/route_surface.rs`

**Markers**:
- `route_surface actual_extraction_done`
- `route_surface closeout_done`
- `route_surface stop_split: true`

**Next step**:
BE-001HG-01 quantscript_graph parent closeout

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
