# v4.16.0 backend.ops_governance.sandbox.comparison_metrics actual extraction complete

> Batch: BE-001MF-02
> Node: `backend.ops_governance.sandbox.comparison_metrics`
> Parent: `backend.ops_governance.sandbox`
> Stage: `extract_closeout`
> Movement: Code movement completed.

---

## Summary

`backend.ops_governance.sandbox.comparison_metrics` has been extracted into a dedicated sandbox child module.

New owner file:

- `src/backend/ops_governance/sandbox/comparison_metrics.rs`

Updated parent files:

- `src/backend/ops_governance/sandbox.rs`
- `src/backend/ops_governance/sandbox/handlers.rs`

The sandbox parent now imports `compute_comparison_metrics` from `comparison_metrics` and continues surfacing it to `verification_run` through the existing parent-controlled boundary.

## Preserved Behavior

BE-001MF-02 preserves:

- graph-id filtering and descending `created_at_ms` sort;
- two-or-more, one, and zero backtest comparison behavior;
- `"full"` and `"partial"` fidelity semantics;
- `BacktestRecord` to `SandboxMetrics` projection;
- v4 replay-shape helper behavior and risk rejection counting;
- direct v4 replay-shape unit test.

## Parent-Child Boundary

`comparison_metrics` is private to `sandbox`.

It is surfaced only by `src/backend/ops_governance/sandbox.rs` for child use by `verification_run`.

No sibling child imports were introduced.

## Non-Movement

BE-001MF-02 did not move:

- metrics_evaluation closed leaf internals;
- `load_or_fetch_ai_proposal`;
- `load_sandbox_report_from_disk`;
- report_api closed leaf internals;
- verification_run closed parent internals;
- root compatibility bridge exports;
- runtime mutation internals;
- release transition policy.

## Next Step

BE-001MF-03 backend.ops_governance.sandbox.comparison_metrics single_leaf_closeout

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
