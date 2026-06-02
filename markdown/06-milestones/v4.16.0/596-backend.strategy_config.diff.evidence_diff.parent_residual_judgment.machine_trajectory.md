# v4.16.0 backend.strategy_config.diff.evidence_diff parent residual judgment selects machine_trajectory

> Batch: BE-001IK-01
> Node: `backend.strategy_config.diff.evidence_diff`
> Parent: `backend.strategy_config.diff`
> Stage: `parent_residual_judgment`
> Movement: no code movement.

---

## Summary

backend.strategy_config.diff.evidence_diff parent residual judgment selects machine_trajectory.

`evidence_diff` remains open after BE-001IJ-01. This round selects the machine
trajectory comparison family first because it has a distinct report schema,
visited-state/transition-hit/terminal-state fields, first-divergence logic, and
signature helper. Risk plane, execution capability, and metrics remain open.

## leaf_split_decision_gate

leaf_split_base_gate

| Rule | Result | Evidence |
| --- | --- | --- |
| white_box_boundary_named | PASS | `machine_trajectory` can be named around `StrategyConfigMachineTrajectoryEvidenceDiff` and its comparison/signature helpers. |
| parent_child_communication_kept | PASS | The child will remain under `evidence_diff`; parent report aggregation remains controlled. |
| equivalence_baseline_freezable | PASS | Existing `strategy_config --lib` evidence diff test asserts machine trajectory difference and first divergence. |

leaf_split_positive_trigger

| Trigger | Result | Evidence |
| --- | --- | --- |
| public_or_handler_boundary | TRUE | The report struct and comparison helper are test-visible and feed the public evidence diff report. |
| state_machine_phase | TRUE | It compares v4 backtest machine trajectory states and transition hits. |
| strategy_branch | TRUE | It branches over visited states, transition counts, terminal state, and first divergence. |
| independent_failure_mode | TRUE | Machine trajectory can diverge independently from risk plane, execution capability, and metrics. |
| reuse_pressure | TRUE | Existing tests inspect machine trajectory output independently. |

leaf_split_stop_condition

| Stop condition | Result | Evidence |
| --- | --- | --- |
| micro_leaf_without_owner | FALSE | The candidate owns a concrete report schema and comparison helper family. |
| communication_cost_rises | FALSE | Moving it reduces mixed evidence comparison helpers without creating sibling shortcuts. |
| local_proof_missing | FALSE | Local evidence diff tests can prove the move. |
| line_count_only | FALSE | Selection is behavior-boundary driven. |

leaf_split_decision_result

`backend.strategy_config.diff.evidence_diff stop_split: false`.

Selected child:
`backend.strategy_config.diff.evidence_diff.machine_trajectory`.

next_recursive_step

BE-001IL-01 backend.strategy_config.diff.evidence_diff.machine_trajectory baseline_plan
## Boundary

**Real files**:
- `src/backend/strategy_config/diff/evidence_diff.rs`
- `src/backend/strategy_config/diff.rs`
- `src/strategy_config_api.rs`
- `src/frontend_api_types.rs`

**Markers**:
- `select machine_trajectory`
- `risk plane remains open`
- `execution capability remains open`
- `metrics remains open`

**Next step**:
BE-001IL-01 backend.strategy_config.diff.evidence_diff.machine_trajectory baseline_plan

---

## Gates

- `git diff --check`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
