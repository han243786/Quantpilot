# v4.16.0 backend.ops_governance.sandbox.comparison_metrics parent residual judgment selects v4_replay_shape

> Batch: BE-001MG-01
> Node: `backend.ops_governance.sandbox.comparison_metrics`
> Parent: `backend.ops_governance.sandbox`
> Stage: `parent_residual_judgment`
> Movement: No code movement.

---

## Summary

`backend.ops_governance.sandbox.comparison_metrics` returns to parent residual judgment after BE-001MF-03 confirmed `stop_split: false`.

The next child is fixed as:

`backend.ops_governance.sandbox.comparison_metrics.v4_replay_shape`

Selection reasons:

- It is a pure v4 artifact comparison owner.
- It has an existing direct unit test.
- It is independent from AppState backtest selection and `BacktestRecord` projection.
- It can be extracted without changing the public `compute_comparison_metrics` boundary.

BE-001MH-01 must establish the v4_replay_shape equivalence baseline before any code movement.

## Parent Residual Queue

| Child | Current boundary | Decision |
| --- | --- | --- |
| `backend.ops_governance.sandbox.comparison_metrics.v4_replay_shape` | v4 artifact replay-shape helper, risk rejection counter, and direct test. | Select for next baseline. |
| `backend.ops_governance.sandbox.comparison_metrics.backtest_projection` | AppState backtest selection, metrics projection, and fidelity fallback. | Keep in parent residual queue. |

## Selected Child Boundary

`v4_replay_shape` currently contains:

- `compare_v4_backtest_artifact_replay_shape`
- `count_v4_risk_rejections`
- test:
  - `v4_artifact_replay_shape_marks_lower_fill_rate_as_underperforming`

The child should not own `compute_comparison_metrics` or `backtest_to_sandbox_metrics`.

## Hard Boundaries

BE-001MH-01/02 must not move:

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

No sibling shortcut is allowed. The selected child must live under `comparison_metrics` and be surfaced only through that parent if needed.

## Next Step

BE-001MH-01 backend.ops_governance.sandbox.comparison_metrics.v4_replay_shape baseline_plan

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
