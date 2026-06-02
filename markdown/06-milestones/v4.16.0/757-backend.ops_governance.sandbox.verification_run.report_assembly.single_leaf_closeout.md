# v4.16.0 backend.ops_governance.sandbox.verification_run.report_assembly single leaf closeout

> Batch: BE-001MA-03
> Node: `backend.ops_governance.sandbox.verification_run.report_assembly`
> Parent: `backend.ops_governance.sandbox.verification_run`
> Stage: `single_leaf_closeout`
> Movement: No code movement.

---

## Summary

`backend.ops_governance.sandbox.verification_run.report_assembly` is closed as a completed single leaf.

Decision:

`stop_split: true`

The extracted child owns one compact DTO assembly boundary:

- receive already computed values;
- clone only `request.proposal_id`;
- move all remaining fields into `SandboxVerificationReport`;
- return the assembled report to the parent runner.

## Split Gate Result

Further splitting is rejected by the recursive split rules:

| Rule | Result |
| --- | --- |
| Concrete owner exists? | No separate owner remains inside the child. |
| Independent IO or state failure mode? | No. The child performs pure DTO field mapping. |
| Parent-child communication would improve? | No. Splitting would make the parent coordinate partial report fields. |
| Local proof would improve? | No. Compile and governance gates already cover the mechanical extraction. |
| Line count only? | Rejected. The child is already a compact leaf. |

## Closed Boundary

The closed leaf remains private to `backend.ops_governance.sandbox.verification_run`.

Allowed future changes:

- update report field mapping only through a new baseline;
- update DTO schema through the appropriate schema owner before changing this child.

Forbidden changes:

- exposing the child through sandbox facade;
- direct sibling calls from proposal_gate, replay_window, report_commit, report_api, or other sandbox children;
- changing field mapping without a new baseline;
- release transition shortcut.

## Next Step

BE-001MB-01 backend.ops_governance.sandbox.verification_run parent_residual_judgment

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
