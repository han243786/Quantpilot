# v4.16.0 backend.strategy_config.diff parent residual judgment selects evidence_diff

> Batch: BE-001IH-01
> Node: `backend.strategy_config.diff`
> Parent: `backend.strategy_config`
> Stage: `parent_residual_judgment`
> Movement: no code movement.

---

## Summary

backend.strategy_config.diff parent residual judgment selects evidence_diff.

`artifact_diff stop_split: true` is closed at BE-001IG-01. The remaining diff
parent residual is the backtest evidence diff pocket: bound backtest loading,
v4 evidence artifact diagnostics, machine trajectory/risk plane/execution
capability/metrics report schemas, and the supporting signature/count helpers.

## leaf_split_decision_gate

leaf_split_base_gate

| Rule | Result | Evidence |
| --- | --- | --- |
| white_box_boundary_named | PASS | `evidence_diff` can be named around backtest evidence diagnostics and v4 evidence comparison without owning artifact diff. |
| parent_child_communication_kept | PASS | The child stays under `backend.strategy_config.diff`; graph/backtest storage remains external. |
| equivalence_baseline_freezable | PASS | Existing `strategy_config --lib` evidence diff test and graph version regression cover this path. |

leaf_split_positive_trigger

| Trigger | Result | Evidence |
| --- | --- | --- |
| public_or_handler_boundary | TRUE | `build_strategy_config_evidence_diff_for_backtests` and `StrategyConfigEvidenceDiffReport` are public compatibility surfaces. |
| state_machine_phase | FALSE | This is evidence comparison/reporting, not a runtime phase. |
| strategy_branch | TRUE | It branches over machine trajectory, risk plane, execution capability, and metrics evidence. |
| independent_failure_mode | TRUE | Missing backtest binding, graph mismatch, and missing v4 artifact diagnostics are evidence-specific. |
| reuse_pressure | TRUE | Graph version compare and frontend response types reuse evidence diff reports independently from artifact diff. |

leaf_split_stop_condition

| Stop condition | Result | Evidence |
| --- | --- | --- |
| micro_leaf_without_owner | FALSE | Evidence diff owns concrete report schemas and loading diagnostics. |
| communication_cost_rises | FALSE | Moving it will leave the parent as a thin facade/re-export boundary. |
| local_proof_missing | FALSE | Local compile, strategy_config, and graph version gates are available. |
| line_count_only | FALSE | Selection is driven by failure domain and public report surface. |

leaf_split_decision_result

`backend.strategy_config.diff stop_split: false`.

Selected child: `backend.strategy_config.diff.evidence_diff`.

next_recursive_step

BE-001II-01 backend.strategy_config.diff.evidence_diff baseline_plan
## Boundary

**Real files**:
- `src/backend/strategy_config/diff.rs`
- `src/backend/strategy_config/diff/artifact_diff.rs`
- `src/strategy_config_api.rs`
- `src/backend/graph_compile/graph.rs`
- `src/frontend_api_types.rs`

**Markers**:
- `select evidence_diff`
- `artifact diff closed`
- `backtest evidence diagnostics candidate`

**Next step**:
BE-001II-01 backend.strategy_config.diff.evidence_diff baseline_plan

---

## Gates

- `git diff --check`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
