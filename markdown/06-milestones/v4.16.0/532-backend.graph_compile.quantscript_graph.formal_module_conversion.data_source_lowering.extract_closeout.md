# v4.16.0 data_source_lowering actual extraction and closeout complete

> Batch: BE-001GQ-02
> Node: `backend.graph_compile.quantscript_graph.formal_module_conversion.data_source_lowering`
> Parent: `backend.graph_compile.quantscript_graph.formal_module_conversion`
> Stage: `extract_closeout`
> Movement: actual extraction + single leaf closeout.
> Speed protocol: `lightweight_two_step`
> Next step: BE-001GR-01 `formal_module_conversion` parent residual judgment

---

## Summary

`data_source_lowering` has been extracted from
`formal_module_conversion.rs` into a direct child module. The parent preserves
overall graph-to-QS assembly order and now delegates data node fetch rendering
through one parent-to-child helper call.

---

## Movement

Created:

```text
src/backend/graph_compile/quantscript_graph/formal_module_conversion/data_source_lowering.rs
```

Updated parent:

```rust
mod data_source_lowering;

data_source_lowering::append_data_source_lowering_lines(nodes, &mut qs_lines);
```

Frozen child surface:

```rust
pub(super) fn append_data_source_lowering_lines(nodes: &[Value], qs_lines: &mut Vec<String>)
```

Added direct local proof:

```text
data_source_lowering::tests::data_source_lowering_renders_fetch_line_with_existing_defaults_and_options
```

---

## Equivalence

Unchanged behavior:

```text
Only nodes with type == "data" append fetch lines.
exchange, instrument, timeframe, window_size, ping_enabled, request_interval_ms, node id fallback, and var-name normalization stay unchanged.
The data fetch lines are appended before profile lowering and intent lowering, as before.
No graph shape validation, profile lowering, unsupported node logging, terminal parse, public API, route, schema, persistence, lock owner, or state-machine behavior changed.
No sibling horizontal connection was introduced.
```

---

## leaf_split_decision_gate

leaf_split_base_gate

| Rule | Result | Evidence |
| --- | --- | --- |
| white_box_boundary_named | yes | Child file owns data node fetch rendering. |
| parent_child_communication_kept | yes | Parent calls one child helper; child does not call siblings. |
| equivalence_baseline_freezable | yes | Local unit test freezes representative rendered fetch output; compile endpoint test covers formal conversion behavior. |

leaf_split_positive_trigger

| Trigger | Result | Evidence |
| --- | --- | --- |
| public_or_handler_boundary | no | No public route or schema changed. |
| state_machine_phase | no | This is a lowering branch, not a runtime phase. |
| strategy_branch | yes | It is a full `data` branch in formal graph-to-QS conversion. |
| independent_failure_mode | no | Validation failures remain parent-owned. |
| reuse_pressure | yes | Data fetch rendering is cohesive and likely to grow with future data modules. |

leaf_split_stop_condition

| Stop condition | Result | Evidence |
| --- | --- | --- |
| micro_leaf_without_owner | no | The child has real branch ownership and a direct test. |
| communication_cost_rises | no | One parent-to-child call replaces inline code. |
| local_proof_missing | no | Direct unit test, compile gate, and zscore golden-view test cover this movement. |
| line_count_only | no | Split is justified by branch ownership and extension pressure. |

leaf_split_decision_result

stop_split_true

next_recursive_step

BE-001GR-01 formal_module_conversion parent residual judgment

## Boundary

**Real files**:
- `src/backend/graph_compile/quantscript_graph/formal_module_conversion.rs`
- `src/backend/graph_compile/quantscript_graph/formal_module_conversion/data_source_lowering.rs`

**Markers**:
- `data_source_lowering actual_extraction_done`
- `data_source_lowering closeout_done`
- `data_source_lowering stop_split: true`

**Next step**:
BE-001GR-01 formal_module_conversion parent residual judgment

---

## Gates

- `git diff --check`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p quantpilot data_source_lowering`
- `cargo test -p quantpilot formal_quantscript_compile_endpoint_matches_one_sided_zscore_golden_view`
