# v4.16.0 strategy_graph_parser actual extraction and closeout complete

> Batch: BE-001HB-02
> Node: `backend.graph_compile.quantscript_graph.strategy_graph_parser`
> Parent: `backend.graph_compile.quantscript_graph`
> Stage: `extract_closeout`
> Movement: actual extraction + single leaf closeout.
> Speed protocol: `lightweight_two_step`
> Next step: BE-001HC-01 `quantscript_graph` parent residual judgment

---

## Summary

`strategy_graph_parser` has been extracted from `quantscript_graph.rs` into a
direct child module. The parent now keeps the public
`parse_graph_quantscript_source` wrapper, timestamp acquisition, and artifact
attachment, while the child owns source parsing and imported graph assembly.

---

## Movement

Created:

```text
src/backend/graph_compile/quantscript_graph/strategy_graph_parser.rs
```

Updated parent:

```rust
mod strategy_graph_parser;

let now = current_time_ms();
let mut graph = strategy_graph_parser::parse_strategy_graph_source(source, now)?;
attach_quantscript_artifacts(&mut graph, source, now, std::path::Path::new(""));
```

Final child surface:

```rust
pub(super) fn parse_strategy_graph_source(source: &str, now: u64) -> anyhow::Result<Value>
```

Added direct local proof:

```text
strategy_graph_parser::tests::parse_qs_scalar_keeps_basic_literals
strategy_graph_parser::tests::parse_qs_node_header_keeps_supported_kinds_only
strategy_graph_parser::tests::parse_qs_connect_keeps_port_mapping
```

---

## Equivalence

Unchanged behavior:

```text
Line normalization, header parse, metadata defaults, node section parsing, runtime mode injection, connection parsing, imported graph shape, public wrapper signature, timestamp ownership, and artifact attachment remain equivalent.
No route surface, graph-to-QS generation, formal module conversion, artifact target projection, runtime target helper, public API, schema, persistence, lock owner, or state-machine behavior changed.
No sibling horizontal connection was introduced.
```

---

## leaf_split_decision_gate

leaf_split_base_gate

| Rule | Result | Evidence |
| --- | --- | --- |
| white_box_boundary_named | yes | Child owns source line normalization, header metadata parse, node parse, connection parse, and imported graph assembly before artifacts. |
| parent_child_communication_kept | yes | Parent calls one `pub(super)` child parser and keeps artifact attachment outside the child. |
| equivalence_baseline_freezable | yes | Round-trip parser coverage and direct helper tests freeze the moved behavior. |

leaf_split_positive_trigger

| Trigger | Result | Evidence |
| --- | --- | --- |
| public_or_handler_boundary | yes | Parent preserves the route-facing `parse_graph_quantscript_source` wrapper. |
| state_machine_phase | no | This is parser/projection logic, not runtime state. |
| strategy_branch | yes | Child owns the `strategy_graph` source sections and graph import semantics. |
| independent_failure_mode | yes | Parser errors remain isolated from generation/formal conversion. |
| reuse_pressure | yes | Parser body is reused through the stable parent wrapper. |

leaf_split_stop_condition

| Stop condition | Result | Evidence |
| --- | --- | --- |
| micro_leaf_without_owner | no | Parser helper cluster has a stable owner and local tests. |
| communication_cost_rises | no | One child parser call replaces inline parser body without cross-child calls. |
| local_proof_missing | no | Direct unit tests plus route-facing round-trip/error tests passed. |
| line_count_only | no | Split is justified by parser ownership and failure boundary. |

leaf_split_decision_result

stop_split_true

next_recursive_step

BE-001HC-01 quantscript_graph parent residual judgment

## Boundary

**Real files**:
- `src/backend/graph_compile/quantscript_graph.rs`
- `src/backend/graph_compile/quantscript_graph/strategy_graph_parser.rs`

**Markers**:
- `strategy_graph_parser actual_extraction_done`
- `strategy_graph_parser closeout_done`
- `strategy_graph_parser stop_split: true`

**Next step**:
BE-001HC-01 quantscript_graph parent residual judgment

---

## Gates

- `git diff --check`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p quantpilot strategy_graph_parser --lib`
- `cargo test -p quantpilot formal_quantscript_text_to_core_ir_to_graph_round_trip_sample`
- `cargo test -p quantpilot formal_quantscript_source_cannot_be_parsed_as_strategy_graph_source`
