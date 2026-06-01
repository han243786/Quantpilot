# v4.16.0 backend.graph_compile parent residual judgment selects graph

> Batch: BE-001HJ-01
> Node: `backend.graph_compile`
> Parent: `backend`
> Stage: `parent_residual_judgment`
> Movement: no code movement.

---

## Summary

backend.graph_compile parent residual judgment selects graph.

`backend.graph_compile.quantscript_graph` and `backend.graph_compile.compile`
are now closed at this level. The remaining old-owner residual is
`backend.graph_compile.graph -> crate::graph_api`.

The next recursive child is `backend.graph_compile.graph`, which owns graph
CRUD, latest graph loading, version/audit operations, artifact bundle commit,
reveal path resolution, and graph persistence cleanup.

## leaf_split_decision_gate

leaf_split_base_gate

| Rule | Result | Evidence |
| --- | --- | --- |
| white_box_boundary_named | pass | `backend.graph_compile.graph` is the only remaining named residual under `backend.graph_compile`. |
| parent_child_communication_kept | pass | The current facade delegates through `backend.graph_compile -> graph`; no sibling route or release-transition shortcut is introduced. |
| equivalence_baseline_freezable | pass | `src/graph_api.rs` still owns the complete legacy behavior and can be frozen before extraction. |

leaf_split_positive_trigger

| Trigger | Result | Evidence |
| --- | --- | --- |
| public_or_handler_boundary | true | `graph_api` owns graph save/list/load/version/audit/delete/reveal handlers. |
| state_machine_phase | false | No parent state-machine phase remains; graph persistence lifecycle belongs to the selected child. |
| strategy_branch | false | Graph route behavior is persistence/versioning, not strategy lowering. |
| independent_failure_mode | true | Graph persistence, version rollback, artifact replacement rollback, and reveal-path validation have independent failure modes. |
| reuse_pressure | true | `resolve_graph_reveal_path_from_value` is used by tests and should remain behind a controlled parent/root export if needed. |

leaf_split_stop_condition

| Stop condition | Result | Evidence |
| --- | --- | --- |
| micro_leaf_without_owner | false | The graph child is a durable route/persistence owner, not a cosmetic wrapper. |
| communication_cost_rises | false | Moving graph behavior behind the existing child facade removes old-owner delegation. |
| local_proof_missing | false | Existing graph/version/reveal tests can verify the next movement. |
| line_count_only | false | The split is driven by route/API ownership and persistence failure boundaries. |

leaf_split_decision_result

`backend.graph_compile stop_split: false`.

Selected next child: `backend.graph_compile.graph`.

next_recursive_step

BE-001HK-01 backend.graph_compile.graph baseline_plan

## Boundary

**Real files**:
- `src/backend/graph_compile.rs`
- `src/backend/graph_compile/graph.rs`
- `src/graph_api.rs`

**Markers**:
- `backend.graph_compile graph_selected`
- `quantscript_graph closed`
- `compile closed`
- `graph residual open`

**Next step**:
BE-001HK-01 backend.graph_compile.graph baseline_plan

---

## Gates

- `git diff --check`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
