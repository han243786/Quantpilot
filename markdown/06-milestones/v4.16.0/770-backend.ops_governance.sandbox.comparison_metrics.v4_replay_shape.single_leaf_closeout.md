# v4.16.0 backend.ops_governance.sandbox.comparison_metrics.v4_replay_shape single leaf closeout

> Batch: BE-001MH-03
> Node: `backend.ops_governance.sandbox.comparison_metrics.v4_replay_shape`
> Parent: `backend.ops_governance.sandbox.comparison_metrics`
> Stage: `single_leaf_closeout`
> Movement: No code movement.

---

## Summary

`backend.ops_governance.sandbox.comparison_metrics.v4_replay_shape` is closed as a completed single leaf.

Decision:

`stop_split: true`

The extracted child owns one pure artifact comparison boundary:

- v4 artifact replay-shape comparison;
- risk rejection counting;
- direct unit test for lower fill-rate underperformance.

## Split Gate Result

Further splitting is rejected by the recursive split rules:

| Rule | Result |
| --- | --- |
| Concrete owner exists? | No separate owner remains inside the child. |
| Independent IO or state failure mode? | No. The child is pure artifact comparison. |
| Parent-child communication would improve? | No. Splitting would separate risk counting from the only helper that consumes it. |
| Local proof would improve? | No. The direct test already sits with the extracted boundary. |
| Line count only? | Rejected. Additional splitting would be line-count driven. |

## Closed Boundary

The closed leaf remains private to `backend.ops_governance.sandbox.comparison_metrics`.

Allowed future changes:

- update v4 replay-shape semantics only through a new baseline;
- update the direct unit test in the same child when helper behavior changes.

Forbidden changes:

- exposing the child directly through sandbox facade;
- moving backtest projection into this closed leaf without reopening the boundary;
- release transition shortcut.

## Next Step

BE-001MI-01 backend.ops_governance.sandbox.comparison_metrics parent_residual_judgment

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
