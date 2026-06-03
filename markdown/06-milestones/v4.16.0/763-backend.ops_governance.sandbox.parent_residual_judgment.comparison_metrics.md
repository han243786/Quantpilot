# v4.16.0 backend.ops_governance.sandbox parent residual judgment selects comparison_metrics

> Batch: BE-001ME-01
> Node: `backend.ops_governance.sandbox`
> Parent: `backend.ops_governance`
> Stage: `parent_residual_judgment`
> Movement: No code movement.

---

## Summary

`backend.ops_governance.sandbox` returns to parent residual judgment after `metrics_evaluation` closed with `stop_split: true`.

The next child is fixed as:

`backend.ops_governance.sandbox.comparison_metrics`

Selection reasons:

- It owns backtest selection and projection into `SandboxMetrics`.
- It owns the remaining v4 replay-shape comparison helper and its direct unit test.
- It is independent from proposal loading and sandbox report disk loading.
- It can be extracted while keeping `verification_run` connected only through the sandbox parent boundary.

BE-001MF-01 must establish the comparison_metrics equivalence baseline before any code movement.

## Parent Residual Queue

| Child | Current boundary | Decision |
| --- | --- | --- |
| `backend.ops_governance.sandbox.comparison_metrics` | Backtest selection, `BacktestRecord` to `SandboxMetrics` projection, v4 replay-shape helper, and related test. | Select for next baseline. |
| `backend.ops_governance.sandbox.proposal_loader` | Memory-first AI proposal load/fetch. | Keep in parent residual queue. |
| `backend.ops_governance.sandbox.report_disk_loader` | Sandbox report disk path validation and JSON load. | Keep in parent residual queue. |

Closed children:

- `backend.ops_governance.sandbox.report_api`
- `backend.ops_governance.sandbox.verification_run`
- `backend.ops_governance.sandbox.metrics_evaluation`

## Selected Child Boundary

`comparison_metrics` currently contains:

- `compute_comparison_metrics`
- `backtest_to_sandbox_metrics`
- `compare_v4_backtest_artifact_replay_shape`
- `count_v4_risk_rejections`
- test:
  - `v4_artifact_replay_shape_marks_lower_fill_rate_as_underperforming`

The child should not own metric diff/verdict/warnings evaluation, proposal loading, disk report loading, route handlers, report assembly, or report commit.

## Hard Boundaries

BE-001MF-01/02 must not move:

- metrics_evaluation closed leaf internals;
- `load_or_fetch_ai_proposal`;
- `load_sandbox_report_from_disk`;
- report_api closed leaf internals;
- verification_run closed parent internals;
- root compatibility bridge exports;
- runtime mutation internals;
- AppState owner;
- release transition policy.

No sibling shortcut is allowed. The selected child must live under `sandbox` and be surfaced only through the sandbox parent boundary.

## Next Step

BE-001MF-01 backend.ops_governance.sandbox.comparison_metrics baseline_plan

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
