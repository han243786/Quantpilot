# v4.16.0 backend.graph_compile.graph equivalence baseline and extraction plan

> Batch: BE-001HK-01
> Node: `backend.graph_compile.graph`
> Parent: `backend.graph_compile`
> Stage: `baseline_plan`
> Movement: no code movement.
> Speed protocol: `lightweight_two_step`
> Next step: BE-001HK-02 `backend.graph_compile.graph` `extract_closeout`

---

## Summary

This baseline freezes the graph route/persistence residual before moving it out
of the old root-level `src/graph_api.rs` implementation owner. The next
movement may extract graph route registration, graph handlers, graph version
persistence, graph artifact replacement/rollback, and reveal-path helpers into
`src/backend/graph_compile/graph.rs`.

`backend.graph_compile.quantscript_graph` and `backend.graph_compile.compile`
remain closed and must not be changed by the graph extraction.

---

## Matrix Impact

| Matrix | Impact node | Change type |
| --- | --- | --- |
| Flow matrix | BE-001HK-01 baseline + plan | lightweight two-step stage 1 |
| Norm matrix | parent-child communication / graph route-persistence boundary | graph residual freeze |
| Guidance matrix | `root.backend.graph_compile.graph` | planned real owner |
| Module tree | `backend.graph_compile -> graph` | final graph_compile child edge becomes real implementation owner |

---

## Equivalence Baseline

Frozen owner before extraction:

```text
src/graph_api.rs
```

Frozen child facade:

```text
src/backend/graph_compile/graph.rs
```

Frozen route surface:

```text
POST /api/graphs/save
GET /api/graphs
GET /api/graphs/latest
GET /api/graphs/:graph_id/audit
GET /api/graphs/:graph_id/versions
GET /api/graphs/:graph_id/versions/compare/:left_version_id/:right_version_id
GET /api/graphs/:graph_id/versions/:version_id
POST /api/graphs/:graph_id/versions/:version_id/restore
POST /api/graphs/:graph_id/reveal
GET /api/graphs/:graph_id
DELETE /api/graphs/:graph_id
```

Frozen handler/helper cluster:

```text
register_graph_routes
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
persist_graph_version
commit_graph_artifact_bundle
replace_graph_artifact
rollback_graph_replacements
refresh_latest_graph_after_delete
resolve_graph_reveal_path_from_value
```

Frozen behavior:

```text
Graph save keeps save lock, graph_id validation, collaboration owner checks, version persistence, latest.json refresh, audit entry, QS artifact bundle commit, and rollback on artifact replacement failure.
Graph list/latest/load/delete keeps storage paths, pagination, index handling, latest refresh, and not-found mapping.
Version endpoints keep version id validation, optional QS source compare input, restore behavior, and graph_version_compare response shape.
Reveal keeps preference for existing QuantScript path and canonical absolute path validation.
```

Frozen non-goals:

```text
No compile movement.
No quantscript_graph movement.
No compile diagnostics or artifact builder movement.
No graph version compare semantic change.
No storage root, lock ordering, route path, response schema, persistence format, or shell reveal behavior change.
No sibling horizontal connection.
No release-transition optimization.
```

---

## Extraction Plan

Planned real owner:

```text
src/backend/graph_compile/graph.rs
```

Planned compatibility result:

```text
src/graph_api.rs may remain as a thin compatibility shim if tests or public module imports require the old module name.
```

Planned child surface:

```rust
pub(crate) fn register_routes(router: Router<AppState>) -> Router<AppState>

pub(crate) async fn resolve_graph_reveal_path_from_value(
    graph: &Value,
    graph_json_path: &FsPath,
) -> anyhow::Result<PathBuf>
```

Planned parent/root wiring:

```text
backend.graph_compile.register_graph_routes continues delegating to graph::register_routes.
Root tests may use a compatibility graph_api shim for resolve_graph_reveal_path_from_value.
```

The child owns only:

```text
graph route registration
graph CRUD handlers
graph version/audit handlers
graph artifact bundle replacement and rollback
graph index/version read helpers
graph reveal path resolution
graph local tests
```

The parent/root keeps ownership of:

```text
backend.graph_compile route group mediation
graph_version_compare module
quantscript_graph child
compile child
root compatibility module if required
```

---

## Speed Protocol Fit

`lightweight_two_step` is valid:

1. The graph child is the only remaining residual under `backend.graph_compile`.
2. The old owner has one coherent route/persistence domain.
3. Compatibility can be kept as a marker/shim while moving implementation.
4. Existing graph list, reveal, version, and rollback tests can prove equivalence.

## Boundary

**Real files**:
- `src/backend/graph_compile/graph.rs`
- `src/graph_api.rs`
- `src/lib.rs`
- `src/tests_backend.rs`

**Markers**:
- `graph baseline_frozen`
- `graph plan_frozen`
- `compile unchanged`
- `quantscript_graph unchanged`

**Next step**:
BE-001HK-02 backend.graph_compile.graph extract_closeout

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
