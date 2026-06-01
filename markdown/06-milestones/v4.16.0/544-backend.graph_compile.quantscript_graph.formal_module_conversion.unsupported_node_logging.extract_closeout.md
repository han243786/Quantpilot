# v4.16.0 unsupported_node_logging actual extraction and closeout complete

> Batch: BE-001GY-02
> Node: `backend.graph_compile.quantscript_graph.formal_module_conversion.unsupported_node_logging`
> Parent: `backend.graph_compile.quantscript_graph.formal_module_conversion`
> Stage: `extract_closeout`
> Movement: actual extraction + single leaf closeout.
> Speed protocol: `lightweight_two_step`
> Next step: BE-001GZ-01 `formal_module_conversion` parent closeout

---

## Summary

`unsupported_node_logging` has been extracted from
`formal_module_conversion.rs` into a direct child module. The parent now keeps
only the conversion loop and calls the child after profile handling.

---

## Movement

Created:

```text
src/backend/graph_compile/quantscript_graph/formal_module_conversion/unsupported_node_logging.rs
```

Updated parent:

```rust
mod unsupported_node_logging;

unsupported_node_logging::log_if_unsupported_node(node);
```

Final child surface:

```rust
pub(super) fn log_if_unsupported_node(node: &Value)
```

Added direct local proof:

```text
unsupported_node_logging::tests::unsupported_node_logging_keeps_known_node_types_silent
unsupported_node_logging::tests::unsupported_node_logging_formats_unknown_node_message
```

---

## Equivalence

Unchanged behavior:

```text
Known node types data / intent / agent / runtime / runtime_control remain no-op.
Unknown node types still emit the same nonblocking safe_eprintln! diagnostic.
Profile lowering still handles risk/execution before unsupported-node logging.
No generated QS output, input validation, data lowering, profile lowering, intent lowering, terminal parse, public API, route, schema, persistence, lock owner, or state-machine behavior changed.
No sibling horizontal connection was introduced.
```

---

## leaf_split_decision_gate

leaf_split_base_gate

| Rule | Result | Evidence |
| --- | --- | --- |
| white_box_boundary_named | yes | Child owns known-node no-op classification and unknown-node diagnostic formatting. |
| parent_child_communication_kept | yes | Parent calls one direct child helper after profile handling. |
| equivalence_baseline_freezable | yes | Local tests freeze classification and message text. |

leaf_split_positive_trigger

| Trigger | Result | Evidence |
| --- | --- | --- |
| public_or_handler_boundary | no | No public route or schema changed. |
| state_machine_phase | no | This is a nonblocking diagnostic path. |
| strategy_branch | no | Not a strategy branch. |
| independent_failure_mode | yes | Unknown nodes are intentionally logged and skipped without aborting conversion. |
| reuse_pressure | no | Split is driven by final diagnostic ownership. |

leaf_split_stop_condition

| Stop condition | Result | Evidence |
| --- | --- | --- |
| micro_leaf_without_owner | no | The child owns the final named residual and local tests. |
| communication_cost_rises | no | One helper replaces inline match logic. |
| local_proof_missing | no | Direct tests, compile gate, and compile endpoint test cover the movement. |
| line_count_only | no | Split is justified by diagnostic ownership. |

leaf_split_decision_result

stop_split_true

next_recursive_step

BE-001GZ-01 formal_module_conversion parent closeout

## Boundary

**Real files**:
- `src/backend/graph_compile/quantscript_graph/formal_module_conversion.rs`
- `src/backend/graph_compile/quantscript_graph/formal_module_conversion/unsupported_node_logging.rs`

**Markers**:
- `unsupported_node_logging actual_extraction_done`
- `unsupported_node_logging closeout_done`
- `unsupported_node_logging stop_split: true`

**Next step**:
BE-001GZ-01 formal_module_conversion parent closeout

---

## Gates

- `git diff --check`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p quantpilot unsupported_node_logging`
- `cargo test -p quantpilot formal_quantscript_compile_endpoint_matches_one_sided_zscore_golden_view`
