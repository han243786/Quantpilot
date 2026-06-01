# v4.16.0 quantscript_graph parent residual judgment selects artifact_target_projection

> Batch: BE-001HC-01
> Node: `backend.graph_compile.quantscript_graph`
> Parent: `backend.graph_compile`
> Stage: `parent_residual_judgment`
> Movement: no code movement.

---

## Summary

quantscript_graph parent residual judgment selects artifact_target_projection

## leaf_split_decision_gate

leaf_split_base_gate

| Rule | Result | Evidence |
| --- | --- | --- |
| white_box_boundary_named | pass | `artifact_target_projection` is the remaining artifact enrichment cluster: node sources, label targets, runtime targets, source id sanitizers, and diagnostic target values. |
| parent_child_communication_kept | pass | This stage only selects the next child. The planned child must be called by the `quantscript_graph` parent and may not connect directly to route surface, parser, formal conversion, or graph generation siblings. |
| equivalence_baseline_freezable | pass | Existing graph compile/round-trip tests can freeze the metadata artifact shape before movement. |

leaf_split_positive_trigger

| Trigger | Result | Evidence |
| --- | --- | --- |
| public_or_handler_boundary | true | `attach_quantscript_artifacts` and `build_compile_runtime_targets_from_graph` are parent-facing helper boundaries used outside the local parser flow. |
| state_machine_phase | false | The residual projects metadata/artifact targets, not runtime state transitions. |
| strategy_branch | true | It maps graph nodes into QS node source, label target, runtime target, and source id projections. |
| independent_failure_mode | true | Runtime target deserialize fallback already has its own nonfatal warning path. |
| reuse_pressure | true | Runtime target projection is reused by compile/runtime callers through the parent boundary. |

leaf_split_stop_condition

| Stop condition | Result | Evidence |
| --- | --- | --- |
| micro_leaf_without_owner | false | The helper cluster has a stable projection owner and several cohesive private helpers. |
| communication_cost_rises | false | A single child projection module can keep one parent call surface and avoid sibling links. |
| local_proof_missing | false | Existing artifact/round-trip tests can cover the projection after movement. |
| line_count_only | false | Selection is based on artifact projection ownership and public helper pressure, not line count. |

leaf_split_decision_result

`stop_split: false` for `backend.graph_compile.quantscript_graph`; select `artifact_target_projection` as the next child.

next_recursive_step

BE-001HD-01 artifact_target_projection baseline_plan
## Boundary

**Real files**:
- `src/backend/graph_compile/quantscript_graph.rs`

**Markers**:
- `artifact_target_projection_selected`

**Next step**:
BE-001HD-01 artifact_target_projection baseline_plan

---

## Gates

- `git diff --check`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
