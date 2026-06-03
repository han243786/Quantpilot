# v4.16.0 backend.ops_governance.sandbox.comparison_metrics.backtest_projection actual extraction complete

> Batch: BE-001MJ-02
> Node: `backend.ops_governance.sandbox.comparison_metrics.backtest_projection`
> Parent: `backend.ops_governance.sandbox.comparison_metrics`
> Stage: `extract_closeout`
> Movement: Code movement completed.

---

## Summary

`backend.ops_governance.sandbox.comparison_metrics.backtest_projection` has been extracted into a private child module under comparison metrics.

New owner file:

- `src/backend/ops_governance/sandbox/comparison_metrics/backtest_projection.rs`

Updated parent file:

- `src/backend/ops_governance/sandbox/comparison_metrics.rs`

The comparison_metrics parent keeps the surfaced `compute_comparison_metrics` bridge and delegates its body to `backtest_projection`.

## Preserved Behavior

BE-001MJ-02 preserves:

- AppState backtest read lock;
- graph id filtering;
- descending `created_at_ms` sort;
- two-or-more backtest `"full"` behavior;
- one-backtest `"partial"` behavior with `metrics.clone()`;
- zero-backtest default `"partial"` behavior;
- all `BacktestRecord` to `SandboxMetrics` field mappings.

## Parent-Child Boundary

`backtest_projection` is private to `comparison_metrics`.

It is not exposed by:

- `src/backend/ops_governance/sandbox.rs`
- `src/backend/ops_governance/sandbox/verification_run.rs`
- `src/sandbox_verification.rs`

No sibling child imports were introduced.

## Non-Movement

BE-001MJ-02 did not move:

- v4_replay_shape closed leaf internals;
- metrics_evaluation closed leaf internals;
- proposal loader;
- disk loader;
- report_api closed leaf internals;
- verification_run closed parent internals;
- root compatibility bridge exports;
- runtime mutation internals;
- release transition policy.

## Next Step

BE-001MJ-03 backend.ops_governance.sandbox.comparison_metrics.backtest_projection single_leaf_closeout

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
