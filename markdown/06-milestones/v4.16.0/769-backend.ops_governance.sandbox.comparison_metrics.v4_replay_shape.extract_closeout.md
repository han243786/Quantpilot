# v4.16.0 backend.ops_governance.sandbox.comparison_metrics.v4_replay_shape actual extraction complete

> Batch: BE-001MH-02
> Node: `backend.ops_governance.sandbox.comparison_metrics.v4_replay_shape`
> Parent: `backend.ops_governance.sandbox.comparison_metrics`
> Stage: `extract_closeout`
> Movement: Code movement completed.

---

## Summary

`backend.ops_governance.sandbox.comparison_metrics.v4_replay_shape` has been extracted into a private child module under comparison metrics.

New owner file:

- `src/backend/ops_governance/sandbox/comparison_metrics/v4_replay_shape.rs`

Updated parent file:

- `src/backend/ops_governance/sandbox/comparison_metrics.rs`

The parent comparison metrics module now keeps backtest selection/projection, while the v4 artifact replay-shape helper and its direct test live in the private child.

## Preserved Behavior

BE-001MH-02 preserves:

- fill-rate extraction and defaulting;
- symbol equality check;
- trajectory coverage check;
- risk rejection non-worse check;
- `f64::EPSILON` fill-rate comparison;
- `CandidateComparable` and `CandidateUnderperforms` mapping;
- risk rejection count filter;
- direct v4 replay-shape unit test.

## Parent-Child Boundary

`v4_replay_shape` is private to `comparison_metrics`.

It is not exposed by:

- `src/backend/ops_governance/sandbox.rs`
- `src/backend/ops_governance/sandbox/verification_run.rs`
- `src/sandbox_verification.rs`

No sibling child imports were introduced.

## Non-Movement

BE-001MH-02 did not move:

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

## Next Step

BE-001MH-03 backend.ops_governance.sandbox.comparison_metrics.v4_replay_shape single_leaf_closeout

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
