# v4.16.0 backend.ops_governance.sandbox.comparison_metrics.backtest_projection single leaf closeout

> Batch: BE-001MJ-03
> Node: `backend.ops_governance.sandbox.comparison_metrics.backtest_projection`
> Parent: `backend.ops_governance.sandbox.comparison_metrics`
> Stage: `single_leaf_closeout`
> Movement: No code movement.

---

## Summary

`backend.ops_governance.sandbox.comparison_metrics.backtest_projection` is closed as a completed single leaf.

Decision:

`stop_split: true`

The extracted child owns one AppState-backed projection boundary:

- backtest read lock;
- graph id filtering;
- descending timestamp sort;
- full/partial/default fidelity fallback;
- `BacktestRecord` to `SandboxMetrics` projection.

## Split Gate Result

Further splitting is rejected by the recursive split rules:

| Rule | Result |
| --- | --- |
| Concrete owner exists? | No separate owner remains inside the child. |
| Independent IO or state failure mode? | No. Read lock, selection, and projection form one comparison input boundary. |
| Parent-child communication would improve? | No. Splitting would force the parent to coordinate partial metrics selection state. |
| Local proof would improve? | No additional local proof exists without broader test generation. |
| Line count only? | Rejected. Additional splitting would be line-count driven. |

## Closed Boundary

The closed leaf remains private to `backend.ops_governance.sandbox.comparison_metrics`.

Allowed future changes:

- update projection or fidelity semantics only through a new baseline;
- add local tests in this child when projection behavior changes.

Forbidden changes:

- exposing the child directly through sandbox facade;
- moving v4 replay-shape back into this closed leaf without reopening the boundary;
- release transition shortcut.

## Next Step

BE-001MK-01 backend.ops_governance.sandbox.comparison_metrics parent_residual_judgment

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
