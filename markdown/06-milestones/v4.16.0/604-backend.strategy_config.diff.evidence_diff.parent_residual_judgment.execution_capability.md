# v4.16.0 backend.strategy_config.diff.evidence_diff parent residual judgment selects execution_capability

> Batch: BE-001IQ-01
> Node: `backend.strategy_config.diff.evidence_diff`
> Parent: `backend.strategy_config.diff`
> Stage: `parent_residual_judgment`
> Movement: no code movement.

---

## Summary

backend.strategy_config.diff.evidence_diff parent residual judgment selects execution_capability

## leaf_split_decision_gate

leaf_split_base_gate

| Rule | Result | Evidence |
| --- | --- | --- |
| white_box_boundary_named | PASS | Next child is `backend.strategy_config.diff.evidence_diff.execution_capability`, owning the execution capability report schema, compare entry, and signature helper. |
| parent_child_communication_kept | PASS | The evidence parent will call the child and keep shared helper ownership until a later residual judgment selects helpers. |
| equivalence_baseline_freezable | PASS | Existing evidence diff tests cover execution capability through the parent report and direct compare helper re-export. |

leaf_split_positive_trigger

| Trigger | Result | Evidence |
| --- | --- | --- |
| public_or_handler_boundary | YES | `compare_execution_capability_evidence` is already a crate-visible compare entry. |
| state_machine_phase | NO | This child is execution capability evidence, not the closed machine trajectory state leaf. |
| strategy_branch | YES | It owns accepted/rejected execution feasibility evidence by runtime mode, capability, source, and status. |
| independent_failure_mode | YES | Capability acceptance/source/status changes can fail independently from risk-plane and metrics evidence. |
| reuse_pressure | MEDIUM | Tests and the parent report need this compare result; future diagnostics can reuse the child without touching metrics. |

leaf_split_stop_condition

| Stop condition | Result | Evidence |
| --- | --- | --- |
| micro_leaf_without_owner | NO | The selected child has a named report contract and compare owner. |
| communication_cost_rises | NO | Moving the whole execution capability compare pocket preserves one parent-to-child call and reduces parent density. |
| local_proof_missing | NO | The current evidence diff baseline can prove this child through existing `strategy_config` tests. |
| line_count_only | NO | Selection is based on evidence-family ownership, not file length. |

leaf_split_decision_result

`select_child: backend.strategy_config.diff.evidence_diff.execution_capability`.

Freeze a focused baseline next. Keep `metrics` and shared helpers as open
residuals under `backend.strategy_config.diff.evidence_diff`.

next_recursive_step

BE-001IR-01 backend.strategy_config.diff.evidence_diff.execution_capability baseline_plan

## Boundary

**Real files**:
- `src/backend/strategy_config/diff/evidence_diff.rs`
- `src/backend/strategy_config/diff/evidence_diff/machine_trajectory.rs`
- `src/backend/strategy_config/diff/evidence_diff/risk_plane.rs`

**Markers**:
- `closed_child:machine_trajectory`
- `closed_child:risk_plane`
- `selected_child:execution_capability`
- `open_residual:metrics`
- `shared_helpers_retained`

**Next step**:
BE-001IR-01 backend.strategy_config.diff.evidence_diff.execution_capability baseline_plan

---

## Gates

- `git diff --check`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
