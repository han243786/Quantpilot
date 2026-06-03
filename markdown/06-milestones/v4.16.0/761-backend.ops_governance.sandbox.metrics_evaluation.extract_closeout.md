# v4.16.0 backend.ops_governance.sandbox.metrics_evaluation actual extraction complete

> Batch: BE-001MD-02
> Node: `backend.ops_governance.sandbox.metrics_evaluation`
> Parent: `backend.ops_governance.sandbox`
> Stage: `extract_closeout`
> Movement: Code movement completed.

---

## Summary

`backend.ops_governance.sandbox.metrics_evaluation` has been extracted into a dedicated sandbox child module.

New owner file:

- `src/backend/ops_governance/sandbox/metrics_evaluation.rs`

Updated parent files:

- `src/backend/ops_governance/sandbox.rs`
- `src/backend/ops_governance/sandbox/handlers.rs`

The sandbox parent now imports diff/verdict/warnings functions from `metrics_evaluation` and continues surfacing them to `verification_run` through the existing parent-controlled boundary.

## Preserved Behavior

BE-001MD-02 preserves:

- all eight `compute_metrics_diff` fields and candidate-minus-baseline arithmetic;
- `format_diff` sign and 4-decimal formatting;
- `determine_sandbox_verdict` improved-count and severe-degradation semantics;
- `compute_sandbox_warnings` partial fidelity, turnover, and drawdown warnings;
- warning order;
- three selected unit tests under the extracted child.

## Parent-Child Boundary

`metrics_evaluation` is private to `sandbox`.

It is surfaced only by `src/backend/ops_governance/sandbox.rs` for child use by `verification_run`.

No sibling child imports were introduced.

## Non-Movement

BE-001MD-02 did not move:

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

## Next Step

BE-001MD-03 backend.ops_governance.sandbox.metrics_evaluation single_leaf_closeout

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
