# v4.16.0 backend.ops_governance.sandbox.comparison_metrics.backtest_projection equivalence baseline and extraction plan

> Batch: BE-001MJ-01
> Node: `backend.ops_governance.sandbox.comparison_metrics.backtest_projection`
> Parent: `backend.ops_governance.sandbox.comparison_metrics`
> Stage: `baseline_plan`
> Movement: No code movement.

---

## Summary

`backend.ops_governance.sandbox.comparison_metrics.backtest_projection` is frozen as the AppState-backed sandbox backtest selection and metrics projection boundary.

Current owner file:

- `src/backend/ops_governance/sandbox/comparison_metrics.rs`

Current embedded functions:

- `compute_comparison_metrics`
- `backtest_to_sandbox_metrics`

BE-001MJ-02 may move only the internal AppState-backed projection body into a dedicated child module while keeping `compute_comparison_metrics` available through the comparison_metrics parent.

## Function Baseline

The extracted child must preserve:

- `state.backtests.read().await`;
- filtering backtests by `b.graph_id == ai_proposal.graph_id`;
- sorting by descending `created_at_ms`;
- when two or more backtests exist:
  - baseline from `graph_backtests[1]`;
  - candidate from `graph_backtests[0]`;
  - fidelity `"full"`;
- when one backtest exists:
  - metrics from `graph_backtests[0]`;
  - `metrics.clone()` for baseline;
  - original metrics for candidate;
  - fidelity `"partial"`;
- when no backtests exist:
  - default baseline metrics;
  - default candidate metrics;
  - fidelity `"partial"`.

`backtest_to_sandbox_metrics` must preserve:

- `total_return_ratio` from `summary.total_return_ratio`;
- `max_drawdown_ratio` from `summary.drawdown_analysis.max_drawdown_ratio.max(0.001)`;
- `sharpe_ratio` from `summary.risk_adjusted.sharpe_ratio`;
- `win_rate` from `summary.win_rate`;
- `avg_hold_hours: 48.0`;
- `turnover_ratio: 0.0`;
- `profit_factor` from `summary.trade_analysis.profit_factor`;
- `calmar_ratio` from `summary.risk_adjusted.calmar_ratio`.

## Parent-Child Boundary

The new child module should be:

- `src/backend/ops_governance/sandbox/comparison_metrics/backtest_projection.rs`

The comparison_metrics parent may keep `compute_comparison_metrics` as the function surfaced to sandbox parent and delegate its body to `backtest_projection`.

## Allowed BE-001MJ-02 Movement

BE-001MJ-02 may:

- create `src/backend/ops_governance/sandbox/comparison_metrics/backtest_projection.rs`;
- add `mod backtest_projection;` inside `src/backend/ops_governance/sandbox/comparison_metrics.rs`;
- move `backtest_to_sandbox_metrics`;
- delegate `compute_comparison_metrics` to the child while preserving its parent-level public boundary.

BE-001MJ-02 must not move:

- v4_replay_shape closed leaf internals;
- metrics_evaluation closed leaf internals;
- proposal loader;
- disk loader;
- report_api closed leaf internals;
- verification_run closed parent internals;
- root compatibility bridge exports;
- runtime mutation internals;
- release transition policy.

## Split Decision Gate

After BE-001MJ-02, BE-001MJ-03 must run single-leaf closeout.

Expected decision: `stop_split: true`, because the child will own one AppState-backed projection boundary.

## Next Step

BE-001MJ-02 backend.ops_governance.sandbox.comparison_metrics.backtest_projection extract_closeout

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
