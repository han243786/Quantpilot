# v4.16.0 backend.ops_governance.sandbox.comparison_metrics parent residual judgment selects backtest_projection

> Batch: BE-001MI-01
> Node: `backend.ops_governance.sandbox.comparison_metrics`
> Parent: `backend.ops_governance.sandbox`
> Stage: `parent_residual_judgment`
> Movement: No code movement.

---

## Summary

`backend.ops_governance.sandbox.comparison_metrics` returns to parent residual judgment after `v4_replay_shape` closed with `stop_split: true`.

The next child is fixed as:

`backend.ops_governance.sandbox.comparison_metrics.backtest_projection`

Selection reasons:

- It is the remaining concrete owner inside comparison_metrics.
- It owns AppState backtest selection, sorting, and fidelity fallback.
- It owns `BacktestRecord` to `SandboxMetrics` projection.
- It can be extracted while keeping `compute_comparison_metrics` as the parent public boundary.

BE-001MJ-01 must establish the backtest_projection equivalence baseline before any code movement.

## Parent Residual Queue

| Child | Current boundary | Decision |
| --- | --- | --- |
| `backend.ops_governance.sandbox.comparison_metrics.backtest_projection` | AppState backtest selection, metrics projection, and fidelity fallback. | Select for next baseline. |

Closed children:

- `backend.ops_governance.sandbox.comparison_metrics.v4_replay_shape`

## Selected Child Boundary

`backtest_projection` currently contains:

- `state.backtests.read().await`;
- graph id filtering by `ai_proposal.graph_id`;
- descending `created_at_ms` sort;
- baseline/candidate selection;
- fidelity string selection;
- `backtest_to_sandbox_metrics`.

The child should return `(SandboxMetrics, SandboxMetrics, String)`.

## Hard Boundaries

BE-001MJ-01/02 must not move:

- v4_replay_shape closed leaf internals;
- metrics_evaluation closed leaf internals;
- proposal loader;
- disk loader;
- report_api closed leaf internals;
- verification_run closed parent internals;
- root compatibility bridge exports;
- runtime mutation internals;
- release transition policy.

No sibling shortcut is allowed. The selected child must live under `comparison_metrics` and be called only by its parent.

## Next Step

BE-001MJ-01 backend.ops_governance.sandbox.comparison_metrics.backtest_projection baseline_plan

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
