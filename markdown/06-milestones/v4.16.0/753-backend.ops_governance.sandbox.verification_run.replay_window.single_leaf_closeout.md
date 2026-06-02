# v4.16.0 backend.ops_governance.sandbox.verification_run.replay_window single leaf closeout

> Batch: BE-001LY-03
> Node: `backend.ops_governance.sandbox.verification_run.replay_window`
> Parent: `backend.ops_governance.sandbox.verification_run`
> Stage: `single_leaf_closeout`
> Movement: No code movement.

---

## Summary

`backend.ops_governance.sandbox.verification_run.replay_window` is closed as a completed single leaf.

Decision:

`stop_split: true`

The extracted child owns one compact replay-window shape boundary:

- current timestamp generation;
- sandbox run id generation;
- replay-days env parsing and defaulting;
- `ReplayWindow` from/to timestamp construction.

## Split Gate Result

Further splitting is rejected by the recursive split rules:

| Rule | Result |
| --- | --- |
| Concrete owner exists? | No separate owner remains inside the child. |
| Independent IO or state failure mode? | No. Env parsing and replay window construction are one shape boundary. |
| Parent-child communication would improve? | No. Splitting would return partial time/window values to the parent. |
| Local proof would improve? | No. Compile and governance gates already cover the mechanical extraction. |
| Line count only? | Rejected. The child is already a compact leaf. |

## Closed Boundary

The closed leaf remains private to `backend.ops_governance.sandbox.verification_run`.

Allowed future changes:

- update replay-day semantics only through a new baseline;
- update time/window shape through the verification_run parent boundary.

Forbidden changes:

- exposing the child through sandbox facade;
- direct sibling calls from proposal_gate, report_commit, report_api, or other sandbox children;
- changing env var/default semantics without a new baseline;
- release transition shortcut.

## Next Step

BE-001LZ-01 backend.ops_governance.sandbox.verification_run parent_residual_judgment

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
