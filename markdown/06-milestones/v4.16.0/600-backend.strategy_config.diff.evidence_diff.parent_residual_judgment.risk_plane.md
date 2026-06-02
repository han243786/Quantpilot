# v4.16.0 backend.strategy_config.diff.evidence_diff parent residual judgment selects risk_plane

> Batch: BE-001IN-01
> Node: `backend.strategy_config.diff.evidence_diff`
> Parent: `backend.strategy_config.diff`
> Stage: `parent_residual_judgment`
> Movement: no code movement.

---

## Summary

backend.strategy_config.diff.evidence_diff parent residual judgment selects risk_plane

## leaf_split_decision_gate

leaf_split_base_gate

| Rule | Result | Evidence |
| --- | --- | --- |
| white_box_boundary_named | PASS | Next child is `backend.strategy_config.diff.evidence_diff.risk_plane`, owning `StrategyConfigRiskPlaneEvidenceDiff`, `compare_risk_plane_evidence`, and `risk_decision_signature`. |
| parent_child_communication_kept | PASS | The evidence parent will call the risk-plane child; shared helpers remain parent-owned until their own residual judgment. |
| equivalence_baseline_freezable | PASS | Existing evidence diff tests exercise risk-plane comparison through the parent report and direct compare helper re-export. |

leaf_split_positive_trigger

| Trigger | Result | Evidence |
| --- | --- | --- |
| public_or_handler_boundary | YES | `compare_risk_plane_evidence` is already a crate-visible compare entry. |
| state_machine_phase | NO | This is not a state-machine trajectory leaf; that child is already closed. |
| strategy_branch | YES | Risk-plane allow/reject decisions are a separate strategy governance branch inside v4 evidence. |
| independent_failure_mode | YES | Risk decision counts, reasons, and first divergence can change independently from execution capability and metrics. |
| reuse_pressure | MEDIUM | Tests and parent report need the compare result; future evidence diagnostics can reuse the same child without touching sibling evidence families. |

leaf_split_stop_condition

| Stop condition | Result | Evidence |
| --- | --- | --- |
| micro_leaf_without_owner | NO | The selected child has a named report contract and compare owner. |
| communication_cost_rises | NO | Moving the whole risk-plane compare pocket lowers parent density while preserving one parent-to-child call. |
| local_proof_missing | NO | The current evidence diff baseline can prove this child through existing `strategy_config` tests. |
| line_count_only | NO | Selection is based on evidence-family ownership, not line count. |

leaf_split_decision_result

`select_child: backend.strategy_config.diff.evidence_diff.risk_plane`.

Freeze a focused baseline next. Keep `execution_capability`, `metrics`, and shared
helpers as open residuals under `backend.strategy_config.diff.evidence_diff`.

next_recursive_step

BE-001IO-01 backend.strategy_config.diff.evidence_diff.risk_plane baseline_plan

## Boundary

**Real files**:
- `src/backend/strategy_config/diff/evidence_diff.rs`
- `src/backend/strategy_config/diff/evidence_diff/machine_trajectory.rs`

**Markers**:
- `closed_child:machine_trajectory`
- `selected_child:risk_plane`
- `open_residual:execution_capability`
- `open_residual:metrics`
- `shared_helpers_retained`

**Next step**:
BE-001IO-01 backend.strategy_config.diff.evidence_diff.risk_plane baseline_plan

---

## Gates

- `git diff --check`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
