# v4.16.0 profile_lowering actual extraction and closeout complete

> Batch: BE-001GS-02
> Node: `backend.graph_compile.quantscript_graph.formal_module_conversion.profile_lowering`
> Parent: `backend.graph_compile.quantscript_graph.formal_module_conversion`
> Stage: `extract_closeout`
> Movement: actual extraction + single leaf closeout.
> Speed protocol: `lightweight_two_step`
> Next step: BE-001GT-01 `formal_module_conversion` parent residual judgment

---

## Summary

`profile_lowering` has been extracted from the parent formal conversion loop
into a direct child module. The final child surface is a single-node helper that
returns whether it handled the node; this preserves the parent loop's ownership
of unsupported-node logging and avoids a second all-node traversal.

---

## Movement

Created:

```text
src/backend/graph_compile/quantscript_graph/formal_module_conversion/profile_lowering.rs
```

Updated parent:

```rust
mod profile_lowering;

if profile_lowering::append_profile_lowering_line(node, &mut qs_lines) {
    continue;
}
```

Final child surface:

```rust
pub(super) fn append_profile_lowering_line(node: &Value, qs_lines: &mut Vec<String>) -> bool
```

Why this differs from the baseline's all-node helper sketch:

```text
The single-node handled helper keeps unsupported_node_logging in the same parent loop and prevents the profile child from owning non-profile node iteration.
```

Added direct local proof:

```text
profile_lowering::tests::profile_lowering_renders_risk_and_execution_lines_with_existing_defaults
```

---

## Equivalence

Unchanged behavior:

```text
Risk profile defaults and output shape are unchanged.
Execution profile defaults and output shape are unchanged.
Profile lines still appear after data fetch lines and before intent lowering.
Unsupported node logging remains parent-owned.
No graph validation, data lowering, intent lowering, terminal parse, public API, route, schema, persistence, lock owner, or state-machine behavior changed.
No sibling horizontal connection was introduced.
```

---

## leaf_split_decision_gate

leaf_split_base_gate

| Rule | Result | Evidence |
| --- | --- | --- |
| white_box_boundary_named | yes | Child owns risk/execution profile line rendering. |
| parent_child_communication_kept | yes | Parent calls one direct child helper per node and no sibling is called by the child. |
| equivalence_baseline_freezable | yes | Local test freezes representative risk/execution outputs; compile endpoint protects broader conversion. |

leaf_split_positive_trigger

| Trigger | Result | Evidence |
| --- | --- | --- |
| public_or_handler_boundary | no | No public route or schema changed. |
| state_machine_phase | no | This is formal graph-to-QS lowering, not a runtime phase. |
| strategy_branch | yes | The child owns the risk/execution branch pair. |
| independent_failure_mode | no | Unsupported-node logging remains outside this child. |
| reuse_pressure | yes | Profile rendering is cohesive and likely to grow with future formal profile options. |

leaf_split_stop_condition

| Stop condition | Result | Evidence |
| --- | --- | --- |
| micro_leaf_without_owner | no | The child has real branch-pair ownership. |
| communication_cost_rises | no | One parent-to-child helper preserves local order and ownership. |
| local_proof_missing | no | Direct unit test, compile gate, and compile endpoint test cover the movement. |
| line_count_only | no | Split is justified by branch ownership. |

leaf_split_decision_result

stop_split_true

next_recursive_step

BE-001GT-01 formal_module_conversion parent residual judgment

## Boundary

**Real files**:
- `src/backend/graph_compile/quantscript_graph/formal_module_conversion.rs`
- `src/backend/graph_compile/quantscript_graph/formal_module_conversion/profile_lowering.rs`

**Markers**:
- `profile_lowering actual_extraction_done`
- `profile_lowering closeout_done`
- `profile_lowering stop_split: true`

**Next step**:
BE-001GT-01 formal_module_conversion parent residual judgment

---

## Gates

- `git diff --check`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p quantpilot profile_lowering`
- `cargo test -p quantpilot formal_quantscript_compile_endpoint_matches_one_sided_zscore_golden_view`
