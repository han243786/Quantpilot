# v4.16.0 backend.ops_governance.sandbox.metrics_evaluation equivalence baseline and extraction plan

> Batch: BE-001MD-01
> Node: `backend.ops_governance.sandbox.metrics_evaluation`
> Parent: `backend.ops_governance.sandbox`
> Stage: `baseline_plan`
> Movement: No code movement.

---

## Summary

`backend.ops_governance.sandbox.metrics_evaluation` is frozen as the pure sandbox metrics diff/verdict/warnings evaluation boundary.

Current owner file:

- `src/backend/ops_governance/sandbox/handlers.rs`

Current embedded functions:

- `compute_metrics_diff`
- `format_diff`
- `determine_sandbox_verdict`
- `compute_sandbox_warnings`

Current related tests:

- `computes_metrics_diff_correctly`
- `verdict_candidate_outperforms_when_most_metrics_improve`
- `check_all_eight_metrics_included_in_diff`

BE-001MD-02 may move only these functions and these tests into a dedicated sandbox child module.

## Function Baseline

`compute_metrics_diff` must preserve:

- all eight metric fields;
- candidate minus baseline arithmetic for each field;
- `format_diff` formatting for each field.

`format_diff` must preserve:

- `+{:.4}` formatting for non-negative values;
- `{:.4}` formatting for negative values.

`determine_sandbox_verdict` must preserve:

- improved count for total return, sharpe, win rate, profit factor, and calmar when parsed value is greater than 0;
- max drawdown and turnover improvement when parsed value is less than 0;
- severe degradation when max drawdown or turnover parsed value is greater than 0.2;
- `CandidateOutperformsBaseline` when improved >= 5 and no severe degradation;
- `CandidateComparable` when improved >= 3 and no severe degradation;
- `CandidateUnderperforms` otherwise.

`compute_sandbox_warnings` must preserve:

- partial replay fidelity warning text;
- turnover warning when parsed turnover diff is greater than 0.05;
- drawdown warning when parsed max drawdown diff is greater than 0.03;
- warning order.

## Parent-Child Boundary

The new child module should be:

- `src/backend/ops_governance/sandbox/metrics_evaluation.rs`

The sandbox parent should surface the functions to `verification_run` through the existing parent-controlled boundary:

- `compute_metrics_diff`
- `determine_sandbox_verdict`
- `compute_sandbox_warnings`

`format_diff` must remain private to `metrics_evaluation`.

## Allowed BE-001MD-02 Movement

BE-001MD-02 may:

- create `src/backend/ops_governance/sandbox/metrics_evaluation.rs`;
- add `mod metrics_evaluation;` inside `src/backend/ops_governance/sandbox.rs`;
- replace sandbox parent imports so `verification_run` still imports through `super::{...}`;
- move the three selected unit tests with the selected functions.

BE-001MD-02 must not move:

- `compute_comparison_metrics`;
- `backtest_to_sandbox_metrics`;
- `compare_v4_backtest_artifact_replay_shape`;
- `count_v4_risk_rejections`;
- `load_or_fetch_ai_proposal`;
- `load_sandbox_report_from_disk`;
- report_api closed leaf internals;
- verification_run closed parent internals;
- root compatibility bridge exports;
- runtime mutation internals;
- release transition policy.

## Split Decision Gate

After BE-001MD-02, BE-001MD-03 must run single-leaf closeout.

Expected decision: `stop_split: true`, because the child will own one pure evaluation boundary with local tests. Continue splitting only if extraction reveals a concrete owner with independent behavior beyond metric diff/verdict/warnings evaluation.

## Next Step

BE-001MD-02 backend.ops_governance.sandbox.metrics_evaluation extract_closeout

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
