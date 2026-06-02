# v4.16.0 backend.strategy_config.diff parent residual judgment selects artifact_diff

> Batch: BE-001IE-01
> Node: `backend.strategy_config.diff`
> Parent: `backend.strategy_config`
> Stage: `parent_residual_judgment`
> Movement: no code movement.

---

## Summary

backend.strategy_config.diff parent residual judgment selects artifact_diff.

`backend.strategy_config.diff` remains open after BE-001ID-01. This round
selects the artifact diff child before evidence diff because it owns the route
request/report boundary, source digest/domain/runtime-boundary comparison, and
the graph-version artifact bridge that prepares left/right artifacts. Evidence
diff stays open as a later child and must not be mixed into the first split.

## leaf_split_decision_gate

leaf_split_base_gate

| Rule | Result | Evidence |
| --- | --- | --- |
| white_box_boundary_named | PASS | `artifact_diff` can name the route request/report and artifact comparison builder without owning evidence diff diagnostics. |
| parent_child_communication_kept | PASS | The child remains under `backend.strategy_config.diff`; old root compatibility exports stay controlled by the parent. |
| equivalence_baseline_freezable | PASS | Existing strategy_config and graph version tests cover artifact diff behavior. |

leaf_split_positive_trigger

| Trigger | Result | Evidence |
| --- | --- | --- |
| public_or_handler_boundary | TRUE | The candidate owns `/api/v1/strategy-config/diff`, `StrategyConfigDiffRequest`, `StrategyConfigDiffReport`, and `build_strategy_config_version_diff`. |
| state_machine_phase | FALSE | Artifact diff is a comparison/report builder, not a runtime state-machine phase. |
| strategy_branch | TRUE | It branches over source digest, config domain, and runtime boundary differences. |
| independent_failure_mode | TRUE | Artifact diff failures are builder/digest/domain comparison issues, distinct from backtest evidence loading and v4 artifact diagnostics. |
| reuse_pressure | TRUE | Graph version compare reuses `build_strategy_config_version_diff` independently from the route handler. |

leaf_split_stop_condition

| Stop condition | Result | Evidence |
| --- | --- | --- |
| micro_leaf_without_owner | FALSE | Route/report/builder ownership is a real child boundary. |
| communication_cost_rises | FALSE | Moving artifact diff first reduces the mixed responsibility in `diff.rs`. |
| local_proof_missing | FALSE | Local compile, strategy_config, and graph version gates are available. |
| line_count_only | FALSE | Selection is driven by public caller and failure-domain separation. |

leaf_split_decision_result

`backend.strategy_config.diff stop_split: false`.

Selected child: `backend.strategy_config.diff.artifact_diff`.

next_recursive_step

BE-001IF-01 backend.strategy_config.diff.artifact_diff baseline_plan
## Boundary

**Real files**:
- `src/backend/strategy_config/diff.rs`
- `src/strategy_config_api.rs`
- `src/backend/graph_compile/graph.rs`
- `src/frontend_api_types.rs`

**Markers**:
- `select artifact_diff`
- `evidence diff remains open`
- `route and graph version bridge candidate`

**Next step**:
BE-001IF-01 backend.strategy_config.diff.artifact_diff baseline_plan

---

## Gates

- `git diff --check`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
