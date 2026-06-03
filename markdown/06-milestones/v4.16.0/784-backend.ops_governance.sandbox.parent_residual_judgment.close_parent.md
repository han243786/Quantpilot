# v4.16.0 backend.ops_governance.sandbox parent residual judgment closes parent

> Batch: BE-001MP-01
> Node: `backend.ops_governance.sandbox`
> Parent: `backend.ops_governance`
> Stage: `parent_residual_judgment`
> Movement: No code movement.

---

## Summary

`backend.ops_governance.sandbox` is closed as a completed parent node.

Decision:

`close_parent: true`

Closed children:

- `backend.ops_governance.sandbox.report_api`
- `backend.ops_governance.sandbox.verification_run`
- `backend.ops_governance.sandbox.metrics_evaluation`
- `backend.ops_governance.sandbox.comparison_metrics`
- `backend.ops_governance.sandbox.proposal_loader`
- `backend.ops_governance.sandbox.report_disk_loader`

The remaining code in `sandbox.rs` is parent facade wiring:

- declare private child modules;
- re-export `run_sandbox_verification`;
- re-export `load_sandbox_report_from_disk`;
- provide parent-controlled helper imports to verification_run;
- delegate route registration to `report_api`.

## Residual Judgment

No additional child is selected inside `sandbox`.

Rejected residual candidates:

| Candidate | Rejection reason |
| --- | --- |
| `facade_wiring` | The remaining code is the parent boundary itself. Extracting it would only wrap module declarations, re-exports, and one route registration delegate. |
| `handlers_legacy_shell` | `src/backend/ops_governance/sandbox/handlers.rs` is drained and contains no concrete helper logic. Removing it would require wider historical path cleanup, not a sandbox behavior split. |

## Closed Parent Boundary

`sandbox` now owns only the sandbox facade boundary.

Closed child implementation files include:

- `src/backend/ops_governance/sandbox/report_api.rs`
- `src/backend/ops_governance/sandbox/verification_run.rs`
- `src/backend/ops_governance/sandbox/metrics_evaluation.rs`
- `src/backend/ops_governance/sandbox/comparison_metrics.rs`
- `src/backend/ops_governance/sandbox/proposal_loader.rs`
- `src/backend/ops_governance/sandbox/report_disk_loader.rs`

Forbidden future changes without a new baseline:

- direct sibling calls into closed children;
- bypassing the sandbox parent re-export from the root compatibility bridge;
- moving runtime mutation internals into sandbox;
- removing the drained historical shell without a path-cleanup baseline;
- release transition shortcut.

## Next Step

BE-001MQ-01 backend.ops_governance parent_residual_judgment

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
