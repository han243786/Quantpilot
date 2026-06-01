# v4.16.0 artifact_target_projection actual extraction and closeout complete

> Batch: BE-001HD-02
> Node: `backend.graph_compile.quantscript_graph.artifact_target_projection`
> Parent: `backend.graph_compile.quantscript_graph`
> Stage: `extract_closeout`
> Movement: actual extraction + single leaf closeout.
> Speed protocol: `lightweight_two_step`
> Next step: BE-001HE-01 `quantscript_graph` parent residual judgment

---

## Summary

`artifact_target_projection` has been extracted from `quantscript_graph.rs`
into a direct child module. Parent wrappers keep the public helper signatures
and mediate graph-to-QS node source generation through a callback, so the child
does not import or call the sibling generator module.

---

## Movement

Created:

```text
src/backend/graph_compile/quantscript_graph/artifact_target_projection.rs
```

Updated parent:

```rust
mod artifact_target_projection;

artifact_target_projection::attach_quantscript_artifacts(
    graph,
    quantscript,
    generated_at,
    quantscript_path,
    graph_to_qs_generation::generate_node_quantscript,
)

artifact_target_projection::build_compile_runtime_targets_from_graph(graph)
```

Final child surfaces:

```rust
pub(super) fn attach_quantscript_artifacts(..., generate_node_quantscript: NodeSourceGenerator)
pub(super) fn build_compile_runtime_targets_from_graph(graph: &Value) -> CompileRuntimeTargets
```

Added direct local proof:

```text
artifact_target_projection::tests::source_segment_sanitizer_keeps_unknown_fallback
artifact_target_projection::tests::runtime_id_sanitizer_trims_outer_underscores
```

---

## Equivalence

Unchanged behavior:

```text
Artifacts keep graph_source, formal_source, node_sources, label_targets, runtime_targets, generated_at, and saved_path.
Existing metadata.source_mode is preserved or defaults to graph.
Node source generation still uses the same graph-to-QS node generator, but only through parent mediation.
Label targets keep alias and duplicate-first semantics.
Runtime target mappings and CompileRuntimeTargets fallback warning remain equivalent.
No route surface, parser, graph generation, formal conversion, public API, schema, persistence, lock owner, or state-machine behavior changed.
No child-to-child sibling horizontal connection was introduced.
```

---

## leaf_split_decision_gate

leaf_split_base_gate

| Rule | Result | Evidence |
| --- | --- | --- |
| white_box_boundary_named | yes | Child owns artifact mutation, node source projection, label targets, runtime targets, sanitizer helpers, and compile target decode. |
| parent_child_communication_kept | yes | Parent wrappers call the child and pass the graph-to-QS generator callback; child has no sibling import. |
| equivalence_baseline_freezable | yes | Existing artifact and formal compile endpoint tests passed after movement. |

leaf_split_positive_trigger

| Trigger | Result | Evidence |
| --- | --- | --- |
| public_or_handler_boundary | yes | Parent-facing wrappers preserve `attach_quantscript_artifacts` and `build_compile_runtime_targets_from_graph`. |
| state_machine_phase | no | This is artifact/runtime target projection, not a runtime state transition. |
| strategy_branch | yes | Child maps strategy graph nodes into artifact/runtime target views. |
| independent_failure_mode | yes | Runtime target deserialize fallback remains isolated and nonfatal. |
| reuse_pressure | yes | Compile/runtime/backtest callers reuse runtime target projection through parent wrapper. |

leaf_split_stop_condition

| Stop condition | Result | Evidence |
| --- | --- | --- |
| micro_leaf_without_owner | no | The projection helper cluster has a cohesive artifact owner. |
| communication_cost_rises | no | One callback-mediated child call avoids extra sibling links. |
| local_proof_missing | no | Direct sanitizer tests and artifact/compile endpoint tests passed. |
| line_count_only | no | Split is justified by artifact projection ownership and reuse pressure. |

leaf_split_decision_result

stop_split_true

next_recursive_step

BE-001HE-01 quantscript_graph parent residual judgment

## Boundary

**Real files**:
- `src/backend/graph_compile/quantscript_graph.rs`
- `src/backend/graph_compile/quantscript_graph/artifact_target_projection.rs`

**Markers**:
- `artifact_target_projection actual_extraction_done`
- `artifact_target_projection closeout_done`
- `artifact_target_projection stop_split: true`

**Next step**:
BE-001HE-01 quantscript_graph parent residual judgment

---

## Gates

- `git diff --check`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p quantpilot artifact_target_projection --lib`
- `cargo test -p quantpilot attach_quantscript_artifacts_preserves_node_source_targets`
- `cargo test -p quantpilot attach_quantscript_artifacts_preserves_formal_source`
- `cargo test -p quantpilot formal_quantscript_compile_endpoint_returns_success`
