# v4.16.0 backend.strategy_config.diff.evidence_diff parent residual judgment selects metrics

> Batch: BE-001IT-01
> Node: `backend.strategy_config.diff.evidence_diff`
> Parent: `backend.strategy_config.diff`
> Stage: `parent_residual_judgment`
> Movement: no code movement.

---

## Summary

backend.strategy_config.diff.evidence_diff parent residual judgment selects metrics

## leaf_split_decision_gate

leaf_split_base_gate

| Rule | Result | Evidence |
| --- | --- | --- |
| white_box_boundary_named | PASS | Next child is `backend.strategy_config.diff.evidence_diff.metrics`, owning metrics report schema, field diff helper, and float stabilization helper. |
| parent_child_communication_kept | PASS | The evidence parent will call the metrics child and keep shared helper ownership until the final residual judgment. |
| equivalence_baseline_freezable | PASS | Existing evidence diff tests cover metrics through the parent report status and serialized fields. |

leaf_split_positive_trigger

| Trigger | Result | Evidence |
| --- | --- | --- |
| public_or_handler_boundary | YES | `compare_evidence_metrics` is the parent-owned metrics compare entry and can become the child entry. |
| state_machine_phase | NO | Metrics are summary-field evidence, not state-machine trajectory evidence. |
| strategy_branch | NO | Metrics do not represent a strategy branch; they represent backtest summary output deltas. |
| independent_failure_mode | YES | Metrics can diverge independently from machine trajectory, risk plane, and execution capability evidence. |
| reuse_pressure | LOW | Current callers need metrics through the parent evidence report only. |

leaf_split_stop_condition

| Stop condition | Result | Evidence |
| --- | --- | --- |
| micro_leaf_without_owner | NO | The selected child has a named report contract and compare owner. |
| communication_cost_rises | NO | Moving the whole metrics compare pocket preserves one parent-to-child call and reduces parent density. |
| local_proof_missing | NO | The current evidence diff baseline can prove this child through existing `strategy_config` tests. |
| line_count_only | NO | Selection is based on evidence-family ownership, not file length. |

leaf_split_decision_result

`select_child: backend.strategy_config.diff.evidence_diff.metrics`.

Freeze a focused baseline next. Keep shared helper types/functions and parent
evidence assembly as open residuals under `backend.strategy_config.diff.evidence_diff`.

next_recursive_step

BE-001IU-01 backend.strategy_config.diff.evidence_diff.metrics baseline_plan

## Boundary

**Real files**:
- `src/backend/strategy_config/diff/evidence_diff.rs`
- `src/backend/strategy_config/diff/evidence_diff/machine_trajectory.rs`
- `src/backend/strategy_config/diff/evidence_diff/risk_plane.rs`
- `src/backend/strategy_config/diff/evidence_diff/execution_capability.rs`

**Markers**:
- `closed_child:machine_trajectory`
- `closed_child:risk_plane`
- `closed_child:execution_capability`
- `selected_child:metrics`
- `shared_helpers_retained`

**Next step**:
BE-001IU-01 backend.strategy_config.diff.evidence_diff.metrics baseline_plan

---

## Gates

- `git diff --check`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
