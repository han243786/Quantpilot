# v4.16.0 backend.ops_governance.sandbox.comparison_metrics equivalence baseline and extraction plan

> Batch: BE-001MF-01
> Node: `backend.ops_governance.sandbox.comparison_metrics`
> Parent: `backend.ops_governance.sandbox`
> Stage: `baseline_plan`
> Movement: No code movement.

---

## Summary

`backend.ops_governance.sandbox.comparison_metrics` is frozen as the sandbox comparison metrics and replay-shape helper boundary.

Current owner file:

- `src/backend/ops_governance/sandbox/handlers.rs`

Current embedded functions:

- `compute_comparison_metrics`
- `backtest_to_sandbox_metrics`
- `compare_v4_backtest_artifact_replay_shape`
- `count_v4_risk_rejections`

Current related test:

- `v4_artifact_replay_shape_marks_lower_fill_rate_as_underperforming`

BE-001MF-02 may move only these functions and this test into a dedicated sandbox child module.

## Function Baseline

`compute_comparison_metrics` must preserve:

- `state.backtests.read().await` lock acquisition;
- filtering backtests by `ai_proposal.graph_id`;
- sorting by descending `created_at_ms`;
- two-or-more backtest behavior:
  - baseline = second newest;
  - candidate = newest;
  - fidelity = `"full"`;
- one-backtest behavior:
  - baseline and candidate from the same metrics value;
  - fidelity = `"partial"`;
- zero-backtest behavior:
  - both metrics default;
  - fidelity = `"partial"`.

`backtest_to_sandbox_metrics` must preserve:

- total return from `summary.total_return_ratio`;
- max drawdown from `summary.drawdown_analysis.max_drawdown_ratio.max(0.001)`;
- sharpe, win rate, profit factor, and calmar source fields;
- fixed `avg_hold_hours: 48.0`;
- fixed `turnover_ratio: 0.0`.

`compare_v4_backtest_artifact_replay_shape` must preserve:

- fill-rate comparison;
- symbol equality;
- trajectory coverage check;
- risk rejection non-worse check;
- `CandidateComparable` / `CandidateUnderperforms` mapping.

`count_v4_risk_rejections` must preserve filtering of unapproved risk plane decisions.

## Parent-Child Boundary

The new child module should be:

- `src/backend/ops_governance/sandbox/comparison_metrics.rs`

The sandbox parent should surface `compute_comparison_metrics` to `verification_run` through the existing parent-controlled boundary.

Internal helpers and tests should remain inside `comparison_metrics`.

## Allowed BE-001MF-02 Movement

BE-001MF-02 may:

- create `src/backend/ops_governance/sandbox/comparison_metrics.rs`;
- add `mod comparison_metrics;` inside `src/backend/ops_governance/sandbox.rs`;
- replace sandbox parent imports so `verification_run` still imports `compute_comparison_metrics` through `super::{...}`;
- move the selected unit test with the selected functions.

BE-001MF-02 must not move:

- metrics_evaluation closed leaf internals;
- `load_or_fetch_ai_proposal`;
- `load_sandbox_report_from_disk`;
- report_api closed leaf internals;
- verification_run closed parent internals;
- root compatibility bridge exports;
- runtime mutation internals;
- release transition policy.

## Split Decision Gate

After BE-001MF-02, BE-001MF-03 must run single-leaf closeout.

Expected decision: `stop_split: false` may be valid if `comparison_metrics` still has separable `backtest_projection` and `v4_replay_shape` owners after extraction. The closeout must apply the split rules instead of closing by habit.

## Next Step

BE-001MF-02 backend.ops_governance.sandbox.comparison_metrics extract_closeout

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
