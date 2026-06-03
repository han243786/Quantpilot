# v4.16.0 backend.ops_governance.sandbox parent residual judgment selects metrics_evaluation

> Batch: BE-001MC-01
> Node: `backend.ops_governance.sandbox`
> Parent: `backend.ops_governance`
> Stage: `parent_residual_judgment`
> Movement: No code movement.

---

## Summary

`backend.ops_governance.sandbox` returns to parent residual judgment after `verification_run` closed as a parent node.

The next child is fixed as:

`backend.ops_governance.sandbox.metrics_evaluation`

Selection reasons:

- It owns pure sandbox metric diff, verdict, and warning evaluation.
- It already has embedded unit tests in `src/backend/ops_governance/sandbox/handlers.rs`.
- It can be extracted mechanically without AppState, disk IO, route, report commit, or runtime mutation changes.
- It reduces `handlers.rs` from a mixed helper bucket into a named evaluation child.

BE-001MD-01 must establish the metrics_evaluation equivalence baseline before any code movement.

## Parent Residual Queue

| Child | Current boundary | Decision |
| --- | --- | --- |
| `backend.ops_governance.sandbox.metrics_evaluation` | `compute_metrics_diff`, `format_diff`, `determine_sandbox_verdict`, `compute_sandbox_warnings`, and related tests. | Select for next baseline. |
| `backend.ops_governance.sandbox.comparison_metrics` | Backtest selection and `BacktestRecord` to `SandboxMetrics` projection. | Keep in parent residual queue. |
| `backend.ops_governance.sandbox.proposal_loader` | Memory-first AI proposal load/fetch. | Keep in parent residual queue. |
| `backend.ops_governance.sandbox.report_disk_loader` | Sandbox report disk path validation and JSON load. | Keep in parent residual queue. |

Closed children:

- `backend.ops_governance.sandbox.report_api`
- `backend.ops_governance.sandbox.verification_run`

## Selected Child Boundary

`metrics_evaluation` currently contains:

- `compute_metrics_diff`
- `format_diff`
- `determine_sandbox_verdict`
- `compute_sandbox_warnings`
- tests:
  - `computes_metrics_diff_correctly`
  - `verdict_candidate_outperforms_when_most_metrics_improve`
  - `check_all_eight_metrics_included_in_diff`

The child should not own backtest selection, report assembly, report commit, proposal loading, or disk report loading.

## Hard Boundaries

BE-001MD-01/02 must not move:

- `compute_comparison_metrics`;
- `backtest_to_sandbox_metrics`;
- v4 artifact replay-shape dead-code helpers;
- `load_or_fetch_ai_proposal`;
- `load_sandbox_report_from_disk`;
- report_api closed leaf internals;
- verification_run closed parent internals;
- root compatibility bridge exports;
- runtime mutation internals;
- AppState owner or storage lifecycle owner;
- release transition policy.

No sibling shortcut is allowed. The selected child must live under `sandbox` and be surfaced only through the sandbox parent boundary.

## Next Step

BE-001MD-01 backend.ops_governance.sandbox.metrics_evaluation baseline_plan

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
