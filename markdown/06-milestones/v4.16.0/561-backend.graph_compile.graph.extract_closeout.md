# v4.16.0 backend.graph_compile.graph actual extraction and closeout complete

> Batch: BE-001HK-02
> Node: `backend.graph_compile.graph`
> Parent: `backend.graph_compile`
> Stage: `extract_closeout`
> Movement: code movement.

---

## Summary

backend.graph_compile.graph actual extraction and closeout complete.

`src/backend/graph_compile/graph.rs` now owns graph route registration,
graph CRUD/version/audit/reveal handlers, graph persistence helpers, artifact
replacement rollback, and graph-local tests. `src/graph_api.rs` remains only as
a test compatibility shim for the former root module path.

`backend.graph_compile.compile` and `backend.graph_compile.quantscript_graph`
were not changed.

---

## Movement

Moved into `src/backend/graph_compile/graph.rs`:

```text
register_routes
save_graph
load_latest_graph
list_graphs
list_graph_versions
load_graph_version
compare_graph_versions
restore_graph_version
list_graph_audit_history
delete_graph
reveal_graph_file
load_graph
read/write graph JSON helpers
graph version persistence helpers
artifact replacement and rollback helpers
reveal path resolution helpers
graph local rollback tests
```

Compatibility marker kept:

```text
src/graph_api.rs re-exports resolve_graph_reveal_path_from_value only for cfg(test).
```

Unchanged siblings:

```text
backend.graph_compile.compile
backend.graph_compile.quantscript_graph
```

---

## Equivalence Evidence

Route equivalence:

```text
backend.graph_compile.register_graph_routes -> graph::register_routes
All /api/graphs routes preserve path and method registration.
```

Persistence equivalence:

```text
Graph save lock, graph_id validation, collaboration owner checks, version persistence, latest.json refresh, audit entry, QS artifact bundle commit, and rollback behavior were moved as one owner cluster.
```

Compatibility equivalence:

```text
Internal tests that still reference graph_api::resolve_graph_reveal_path_from_value keep compiling through a cfg(test) shim.
Production route owner is backend.graph_compile.graph.
```

## leaf_split_decision_gate

leaf_split_base_gate

| Rule | Result | Evidence |
| --- | --- | --- |
| white_box_boundary_named | pass | `backend.graph_compile.graph` now owns graph route/persistence behavior. |
| parent_child_communication_kept | pass | Route registration still flows through `backend.graph_compile`; no sibling calls were added. |
| equivalence_baseline_freezable | pass | BE-001HK-01 froze graph routes, persistence/version/reveal helpers, and rollback behavior before movement. |

leaf_split_positive_trigger

| Trigger | Result | Evidence |
| --- | --- | --- |
| public_or_handler_boundary | closed | Graph handlers and route registration are now in the graph child. |
| state_machine_phase | false | No parent state-machine phase remains. |
| strategy_branch | false | Graph persistence stays as one graph API owner at this level. |
| independent_failure_mode | closed | Version, reveal, rollback, and delete/list failure behavior moved with the child owner. |
| reuse_pressure | closed | The only known root-module reuse is handled by a cfg(test) compatibility shim. |

leaf_split_stop_condition

| Stop condition | Result | Evidence |
| --- | --- | --- |
| micro_leaf_without_owner | true | Further split should be decided inside a later graph parent residual judgment, not during this parent extraction. |
| communication_cost_rises | false | Current extraction removes old-owner delegation and preserves parent route mediation. |
| local_proof_missing | false | Graph rollback/list/reveal/version tests passed after movement. |
| line_count_only | true | Any further graph split must be route/persistence-domain driven, not line-count driven. |

leaf_split_decision_result

`stop_split: true` for `backend.graph_compile.graph` at this level.

Further internal graph branch splitting requires a future parent residual judgment.

next_recursive_step

BE-001HL-01 backend.graph_compile parent closeout

## Boundary

**Real files**:
- `src/backend/graph_compile/graph.rs`
- `src/graph_api.rs`
- `src/backend/graph_compile.rs`

**Markers**:
- `graph actual_extraction_done`
- `graph closeout_done`
- `graph stop_split: true`
- `graph_api compatibility shim kept`

**Next step**:
BE-001HL-01 backend.graph_compile parent closeout

---

## Gates

- `git diff --check`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p quantpilot rollback_restores_replaced_graph_artifact --lib`
- `cargo test -p quantpilot graphs_endpoint_lists_saved_graph_files_only`
- `cargo test -p quantpilot reveal_graph_endpoint_returns_not_found_for_missing_graph`
- `cargo test -p quantpilot graph_version_endpoints_list_load_and_restore_versions`
