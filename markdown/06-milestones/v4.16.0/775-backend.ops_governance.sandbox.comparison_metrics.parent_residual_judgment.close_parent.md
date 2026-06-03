# v4.16.0 backend.ops_governance.sandbox.comparison_metrics parent residual judgment closes parent

> Batch: BE-001MK-01
> Node: `backend.ops_governance.sandbox.comparison_metrics`
> Parent: `backend.ops_governance.sandbox`
> Stage: `parent_residual_judgment`
> Movement: No code movement.

---

## Summary

`backend.ops_governance.sandbox.comparison_metrics` is closed as a completed parent node.

Decision:

`close_parent: true`

Closed children:

- `backend.ops_governance.sandbox.comparison_metrics.v4_replay_shape`
- `backend.ops_governance.sandbox.comparison_metrics.backtest_projection`

The remaining code in `comparison_metrics.rs` is parent bridge wiring:

- declare the two private child modules;
- surface `compute_comparison_metrics` to the sandbox parent;
- delegate the AppState-backed projection to `backtest_projection`;
- keep v4 replay-shape implementation private to its closed child.

## Residual Judgment

No additional child is selected inside `comparison_metrics`.

Rejected residual candidate:

| Candidate | Rejection reason |
| --- | --- |
| `comparison_bridge` | The remaining function only delegates to the closed `backtest_projection` child. Splitting it would create a wrapper around a wrapper and trigger communication_cost_rises and line_count_only. |

The actual proposal loader, disk report loader, metric evaluation helpers, route API, and verification runner remain owned by the higher `backend.ops_governance.sandbox` parent residual process.

## Closed Parent Boundary

`comparison_metrics` now owns only the comparison metrics parent boundary.

Its closed children remain private child modules:

- `src/backend/ops_governance/sandbox/comparison_metrics/v4_replay_shape.rs`
- `src/backend/ops_governance/sandbox/comparison_metrics/backtest_projection.rs`

Forbidden future changes without a new baseline:

- exposing either child directly through the sandbox facade;
- moving proposal loading or disk report loading into this parent;
- direct sibling calls from verification_run into comparison_metrics children;
- release transition shortcut.

## Next Step

BE-001ML-01 backend.ops_governance.sandbox parent_residual_judgment

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
