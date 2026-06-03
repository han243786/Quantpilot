# v4.16.0 backend.ops_governance.sandbox.comparison_metrics.v4_replay_shape equivalence baseline and extraction plan

> Batch: BE-001MH-01
> Node: `backend.ops_governance.sandbox.comparison_metrics.v4_replay_shape`
> Parent: `backend.ops_governance.sandbox.comparison_metrics`
> Stage: `baseline_plan`
> Movement: No code movement.

---

## Summary

`backend.ops_governance.sandbox.comparison_metrics.v4_replay_shape` is frozen as the pure v4 backtest artifact replay-shape comparison boundary.

Current owner file:

- `src/backend/ops_governance/sandbox/comparison_metrics.rs`

Current embedded functions:

- `compare_v4_backtest_artifact_replay_shape`
- `count_v4_risk_rejections`

Current related test:

- `v4_artifact_replay_shape_marks_lower_fill_rate_as_underperforming`

BE-001MH-02 may move only these functions and this test into a dedicated child module under `comparison_metrics`.

## Function Baseline

`compare_v4_backtest_artifact_replay_shape` must preserve:

- baseline fill-rate extraction from optional `microstructure_metrics`;
- candidate fill-rate extraction from optional `microstructure_metrics`;
- missing fill-rate default of `0.0`;
- `baseline.symbols == candidate.symbols`;
- trajectory coverage check using `candidate.machine_trajectory.len() >= baseline.machine_trajectory.len().saturating_div(2)`;
- risk rejection non-worse check through `count_v4_risk_rejections`;
- fill-rate comparison with `candidate_fill_rate + f64::EPSILON >= baseline_fill_rate`;
- `SandboxVerdict::CandidateComparable` only when all checks pass;
- `SandboxVerdict::CandidateUnderperforms` otherwise.

`count_v4_risk_rejections` must preserve:

- iteration over `artifact.risk_plane_decisions`;
- counting decisions where `!decision.approved`.

## Parent-Child Boundary

The new child module should be:

- `src/backend/ops_governance/sandbox/comparison_metrics/v4_replay_shape.rs`

The child does not need to expose a public boundary outside `comparison_metrics` unless future parent logic uses the helper.

## Allowed BE-001MH-02 Movement

BE-001MH-02 may:

- create `src/backend/ops_governance/sandbox/comparison_metrics/v4_replay_shape.rs`;
- add `mod v4_replay_shape;` inside `src/backend/ops_governance/sandbox/comparison_metrics.rs`;
- move the selected unit test with the selected functions.

BE-001MH-02 must not move:

- `compute_comparison_metrics`;
- `backtest_to_sandbox_metrics`;
- metrics_evaluation closed leaf internals;
- proposal loader;
- disk loader;
- report_api closed leaf internals;
- verification_run closed parent internals;
- root compatibility bridge exports;
- runtime mutation internals;
- release transition policy.

## Split Decision Gate

After BE-001MH-02, BE-001MH-03 must run single-leaf closeout.

Expected decision: `stop_split: true`, because the child will own one pure artifact comparison helper and its direct test.

## Next Step

BE-001MH-02 backend.ops_governance.sandbox.comparison_metrics.v4_replay_shape extract_closeout

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
