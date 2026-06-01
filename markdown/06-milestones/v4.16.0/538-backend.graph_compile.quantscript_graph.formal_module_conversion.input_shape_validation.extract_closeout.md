# v4.16.0 input_shape_validation actual extraction and closeout complete

> Batch: BE-001GU-02
> Node: `backend.graph_compile.quantscript_graph.formal_module_conversion.input_shape_validation`
> Parent: `backend.graph_compile.quantscript_graph.formal_module_conversion`
> Stage: `extract_closeout`
> Movement: actual extraction + single leaf closeout.
> Speed protocol: `lightweight_two_step`
> Next step: BE-001GV-01 `formal_module_conversion` parent residual judgment

---

## Summary

`input_shape_validation` has been extracted from
`formal_module_conversion.rs` into a direct child module. The parent now calls a
single entry guard before any lowering children run.

---

## Movement

Created:

```text
src/backend/graph_compile/quantscript_graph/formal_module_conversion/input_shape_validation.rs
```

Updated parent:

```rust
mod input_shape_validation;

let (nodes, edges) = input_shape_validation::require_graph_nodes_and_edges(graph_value)?;
```

Final child surface:

```rust
pub(super) fn require_graph_nodes_and_edges(graph_value: &Value) -> anyhow::Result<(&[Value], &[Value])>
```

Added direct local proof:

```text
input_shape_validation::tests::input_shape_validation_returns_nodes_and_edges_arrays
input_shape_validation::tests::input_shape_validation_rejects_missing_nodes_array
input_shape_validation::tests::input_shape_validation_rejects_missing_edges_array
```

---

## Equivalence

Unchanged behavior:

```text
graph.nodes missing or not array still returns "graph.nodes 必须是数组".
graph.edges missing or not array still returns "graph.edges 必须是数组".
nodes validation still runs before edges validation.
No lowering child runs before both arrays are available.
No data lowering, profile lowering, intent lowering, unsupported-node logging, terminal parse, public API, route, schema, persistence, lock owner, or state-machine behavior changed.
No sibling horizontal connection was introduced.
```

---

## leaf_split_decision_gate

leaf_split_base_gate

| Rule | Result | Evidence |
| --- | --- | --- |
| white_box_boundary_named | yes | Child owns graph `nodes`/`edges` required-array checks. |
| parent_child_communication_kept | yes | Parent calls one direct child helper; child does not call siblings. |
| equivalence_baseline_freezable | yes | Local tests freeze success and both error strings. |

leaf_split_positive_trigger

| Trigger | Result | Evidence |
| --- | --- | --- |
| public_or_handler_boundary | no | No route or schema changed. |
| state_machine_phase | no | This is an entry guard, not a runtime phase. |
| strategy_branch | no | Not a strategy branch. |
| independent_failure_mode | yes | The child owns two hard shape failure paths. |
| reuse_pressure | yes | Centralized graph-shape validation protects all downstream formal conversion children. |

leaf_split_stop_condition

| Stop condition | Result | Evidence |
| --- | --- | --- |
| micro_leaf_without_owner | no | The child has a named entry-guard owner and three direct tests. |
| communication_cost_rises | no | One parent-to-child helper returns borrowed slices. |
| local_proof_missing | no | Direct tests, compile gate, and compile endpoint test cover the movement. |
| line_count_only | no | Split is justified by failure-path ownership. |

leaf_split_decision_result

stop_split_true

next_recursive_step

BE-001GV-01 formal_module_conversion parent residual judgment

## Boundary

**Real files**:
- `src/backend/graph_compile/quantscript_graph/formal_module_conversion.rs`
- `src/backend/graph_compile/quantscript_graph/formal_module_conversion/input_shape_validation.rs`

**Markers**:
- `input_shape_validation actual_extraction_done`
- `input_shape_validation closeout_done`
- `input_shape_validation stop_split: true`

**Next step**:
BE-001GV-01 formal_module_conversion parent residual judgment

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
