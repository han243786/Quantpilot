# v4.16.0 unsupported_intent_failure actual extraction and closeout complete

> Batch: BE-001GN-02
> Node: `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.unsupported_intent_failure`
> Parent: `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering`
> Stage: `extract_closeout`
> Movement: actual extraction + single leaf closeout.
> Speed protocol: `lightweight_two_step`
> Next step: BE-001GO-01 `intent_lowering` parent residual closeout

---

## Summary

`unsupported_intent_failure` has been extracted from the parent
`intent_lowering` match default branch into a child helper. The parent still
owns intent iteration, context resolution, and module dispatch; the child owns
only the supported intent display list and hard unsupported intent diagnostic.

---

## Movement

Created:

```text
src/backend/graph_compile/quantscript_graph/formal_module_conversion/intent_lowering/unsupported_intent_failure.rs
```

Updated parent:

```rust
mod unsupported_intent_failure;

_ => {
    unsupported_intent_failure::bail_unsupported_intent(ctx.module_key)?;
}
```

Frozen child surface:

```rust
pub(super) fn bail_unsupported_intent(module_key: &str) -> anyhow::Result<()>
```

Added direct local proof:

```text
unsupported_intent_failure::tests::unsupported_intent_failure_message_stays_stable
```

---

## Equivalence

Unchanged behavior:

```text
Supported built-in intent branches still dispatch through the parent match.
Unsupported intent branches still abort with the same Chinese diagnostic.
The supported module display string is unchanged.
No route, schema, persistence, lock owner, public API, or state-machine behavior changed.
No sibling horizontal connection was introduced.
```

---

## leaf_split_decision_gate

leaf_split_base_gate

| Rule | Result | Evidence |
| --- | --- | --- |
| white_box_boundary_named | yes | Child file owns unsupported intent diagnostic construction. |
| parent_child_communication_kept | yes | Parent calls one child helper and no child calls siblings. |
| equivalence_baseline_freezable | yes | Direct unit test freezes exact failure message; zscore golden-view keeps supported dispatch covered. |

leaf_split_positive_trigger

| Trigger | Result | Evidence |
| --- | --- | --- |
| public_or_handler_boundary | no | No public route or schema changed. |
| state_machine_phase | no | This is a hard failure helper, not a phase. |
| strategy_branch | yes | Parent default branch is now delegated to the child. |
| independent_failure_mode | yes | The child owns the standalone unsupported intent error path. |
| reuse_pressure | no | Reuse is not the split driver. |

leaf_split_stop_condition

| Stop condition | Result | Evidence |
| --- | --- | --- |
| micro_leaf_without_owner | no | The child has a named failure owner and local test. |
| communication_cost_rises | no | One parent-to-child helper call replaces inline failure code. |
| local_proof_missing | no | Direct unit test and compile gate cover the movement. |
| line_count_only | no | Split is based on failure ownership. |

leaf_split_decision_result

stop_split_true

next_recursive_step

BE-001GO-01 intent_lowering parent residual closeout

## Boundary

**Real files**:
- `src/backend/graph_compile/quantscript_graph/formal_module_conversion/intent_lowering.rs`
- `src/backend/graph_compile/quantscript_graph/formal_module_conversion/intent_lowering/unsupported_intent_failure.rs`

**Markers**:
- `unsupported_intent_failure actual_extraction_done`
- `unsupported_intent_failure closeout_done`
- `unsupported_intent_failure stop_split: true`

**Next step**:
BE-001GO-01 intent_lowering parent residual closeout

---

## Gates

- `git diff --check`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p quantpilot unsupported_intent_failure`
- `cargo test -p quantpilot formal_quantscript_compile_endpoint_matches_one_sided_zscore_golden_view`
