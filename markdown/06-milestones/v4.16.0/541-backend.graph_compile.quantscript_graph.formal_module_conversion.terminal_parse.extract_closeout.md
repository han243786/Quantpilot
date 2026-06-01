# v4.16.0 terminal_parse actual extraction and closeout complete

> Batch: BE-001GW-02
> Node: `backend.graph_compile.quantscript_graph.formal_module_conversion.terminal_parse`
> Parent: `backend.graph_compile.quantscript_graph.formal_module_conversion`
> Stage: `extract_closeout`
> Movement: actual extraction + single leaf closeout.
> Speed protocol: `lightweight_two_step`
> Next step: BE-001GX-01 `formal_module_conversion` parent residual judgment

---

## Summary

`terminal_parse` has been extracted from `formal_module_conversion.rs` into a
direct child module. The parent now hands the generated QS line vector to the
terminal child, which appends the closing brace, joins with newline, and invokes
`parse_quant_script_module`.

---

## Movement

Created:

```text
src/backend/graph_compile/quantscript_graph/formal_module_conversion/terminal_parse.rs
```

Updated parent:

```rust
mod terminal_parse;

terminal_parse::parse_generated_qs_lines(qs_lines)
```

Final child surface:

```rust
pub(super) fn parse_generated_qs_lines(mut qs_lines: Vec<String>) -> anyhow::Result<ScriptModule>
```

---

## Equivalence

Unchanged behavior:

```text
The generated QS source is still closed with "}".
QS lines are still joined with "\n".
The terminal parser remains `parse_quant_script_module(&qs_source)`.
No generated line content, input validation, data lowering, profile lowering, intent lowering, unsupported-node logging, public API, route, schema, persistence, lock owner, or state-machine behavior changed.
No sibling horizontal connection was introduced.
```

---

## leaf_split_decision_gate

leaf_split_base_gate

| Rule | Result | Evidence |
| --- | --- | --- |
| white_box_boundary_named | yes | Child owns terminal close/join/parse. |
| parent_child_communication_kept | yes | Parent calls one direct child helper with `qs_lines`. |
| equivalence_baseline_freezable | yes | Compile endpoint golden-view test exercises the terminal parse path. |

leaf_split_positive_trigger

| Trigger | Result | Evidence |
| --- | --- | --- |
| public_or_handler_boundary | no | No public route or schema changed. |
| state_machine_phase | yes | Terminal parse is the final phase of formal conversion. |
| strategy_branch | no | Not a strategy branch. |
| independent_failure_mode | yes | Parse failure remains the final hard conversion failure path. |
| reuse_pressure | yes | Terminal parse ownership can grow diagnostics without changing parent orchestration. |

leaf_split_stop_condition

| Stop condition | Result | Evidence |
| --- | --- | --- |
| micro_leaf_without_owner | no | The child owns a named terminal phase. |
| communication_cost_rises | no | One terminal helper replaces inline code. |
| local_proof_missing | no | Compile gate and formal compile endpoint test cover the movement. |
| line_count_only | no | Split is justified by terminal phase ownership. |

leaf_split_decision_result

stop_split_true

next_recursive_step

BE-001GX-01 formal_module_conversion parent residual judgment

## Boundary

**Real files**:
- `src/backend/graph_compile/quantscript_graph/formal_module_conversion.rs`
- `src/backend/graph_compile/quantscript_graph/formal_module_conversion/terminal_parse.rs`

**Markers**:
- `terminal_parse actual_extraction_done`
- `terminal_parse closeout_done`
- `terminal_parse stop_split: true`

**Next step**:
BE-001GX-01 formal_module_conversion parent residual judgment

---

## Gates

- `git diff --check`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p quantpilot formal_quantscript_compile_endpoint_matches_one_sided_zscore_golden_view`
