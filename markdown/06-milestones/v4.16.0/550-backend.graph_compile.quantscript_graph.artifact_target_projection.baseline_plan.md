# v4.16.0 artifact_target_projection equivalence baseline and extraction plan

> Batch: BE-001HD-01
> Node: `backend.graph_compile.quantscript_graph.artifact_target_projection`
> Parent: `backend.graph_compile.quantscript_graph`
> Stage: `baseline_plan`
> Movement: no code movement.
> Speed protocol: `lightweight_two_step`
> Next step: BE-001HD-02 `artifact_target_projection` `extract_closeout`

---

## Summary

This baseline freezes the QuantScript graph artifact projection cluster. The
next movement may extract only artifact enrichment and runtime target
projection into `artifact_target_projection.rs`; the parent keeps route surface
and mediates the sibling graph-to-QS generator through a callback.

---

## Matrix Impact

| Matrix | Impact node | Change type |
| --- | --- | --- |
| Flow matrix | BE-001HD-01 baseline + plan | lightweight two-step stage 1 |
| Norm matrix | parent-child communication / artifact projection boundary | artifact projection freeze |
| Guidance matrix | `root.backend.graph_compile.quantscript_graph.artifact_target_projection` | planned child white-box node |
| Module tree | `quantscript_graph -> artifact_target_projection` | planned child edge |

---

## Equivalence Baseline

Frozen owner:

```text
src/backend/graph_compile/quantscript_graph.rs
```

Frozen parent-facing helpers:

```text
attach_quantscript_artifacts
build_compile_runtime_targets_from_graph
```

Frozen private projection helpers:

```text
build_quantscript_node_sources
build_quantscript_label_targets
build_quantscript_runtime_targets
script_data_source_id_from_graph_node
sanitize_quantscript_source_segment
sanitize_quantscript_runtime_id
insert_label_target
diagnostic_target_value
```

Frozen behavior:

```text
Artifacts keep graph_source, formal_source, node_sources, label_targets, runtime_targets, generated_at, and saved_path.
Existing metadata.source_mode is preserved or defaults to graph.
Node source generation still uses graph_to_qs_generation::generate_node_quantscript through the parent boundary.
Label targets keep node id/name/name-field/config-field aliases and duplicate-first semantics.
Runtime targets keep data/intent/agent/risk/runtime/execution mappings and unknown-node no-op behavior.
CompileRuntimeTargets deserialize failure still safe-logs and falls back to default.
```

Frozen non-goals:

```text
No route surface movement.
No parser movement.
No graph-to-QS generation movement.
No formal module conversion movement.
No public API, schema, persistence, lock owner, or state-machine change.
No child-to-child sibling call.
No release-transition optimization.
```

---

## Extraction Plan

Planned child file:

```text
src/backend/graph_compile/quantscript_graph/artifact_target_projection.rs
```

Planned parent additions:

```rust
mod artifact_target_projection;

pub(crate) fn attach_quantscript_artifacts(
    graph: &mut Value,
    quantscript: &str,
    generated_at: u64,
    quantscript_path: &std::path::Path,
) {
    artifact_target_projection::attach_quantscript_artifacts(
        graph,
        quantscript,
        generated_at,
        quantscript_path,
        graph_to_qs_generation::generate_node_quantscript,
    )
}

pub(crate) fn build_compile_runtime_targets_from_graph(graph: &Value) -> CompileRuntimeTargets {
    artifact_target_projection::build_compile_runtime_targets_from_graph(graph)
}
```

Planned child surfaces:

```rust
pub(super) fn attach_quantscript_artifacts(
    graph: &mut Value,
    quantscript: &str,
    generated_at: u64,
    quantscript_path: &std::path::Path,
    generate_node_quantscript: fn(&Value, &[Value], &[Value]) -> anyhow::Result<String>,
)

pub(super) fn build_compile_runtime_targets_from_graph(graph: &Value) -> CompileRuntimeTargets
```

The parent keeps ownership of:

```text
route registration and handlers
public helper wrapper signatures
mediating graph_to_qs_generation child reuse
parser wrapper and artifact attachment call order
```

The child owns only:

```text
artifact map mutation
node source projection via parent-supplied generator
label target projection
runtime target projection
source id sanitization
diagnostic target value construction
CompileRuntimeTargets fallback warning
```

---

## Speed Protocol Fit

`lightweight_two_step` is valid:

1. The child is a cohesive artifact/runtime projection cluster.
2. Parent wrappers preserve external signatures and avoid sibling links.
3. Existing tests already freeze node_sources, label_targets, formal_source, and runtime_targets.
4. No route, schema, persistence, lock, or state-machine behavior moves.

## Boundary

**Real files**:
- `src/backend/graph_compile/quantscript_graph.rs`
- `src/backend/graph_compile/quantscript_graph/artifact_target_projection.rs` (planned)

**Markers**:
- `artifact_target_projection baseline_frozen`
- `artifact_target_projection plan_frozen`

**Next step**:
BE-001HD-02 artifact_target_projection extract_closeout

---

## Gates

- `git diff --check`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p quantpilot attach_quantscript_artifacts_preserves_node_source_targets`
- `cargo test -p quantpilot attach_quantscript_artifacts_preserves_formal_source`
- `cargo test -p quantpilot formal_quantscript_compile_endpoint_returns_success`
