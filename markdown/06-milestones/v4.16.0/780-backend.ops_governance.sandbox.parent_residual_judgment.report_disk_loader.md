# v4.16.0 backend.ops_governance.sandbox parent residual judgment selects report_disk_loader

> Batch: BE-001MN-01
> Node: `backend.ops_governance.sandbox`
> Parent: `backend.ops_governance`
> Stage: `parent_residual_judgment`
> Movement: No code movement.

---

## Summary

`backend.ops_governance.sandbox` returns to parent residual judgment after `proposal_loader` closed with `stop_split: true`.

The next child is fixed as:

`backend.ops_governance.sandbox.report_disk_loader`

Selection reasons:

- It is the remaining concrete owner inside `handlers.rs`.
- It owns sandbox report proposal id validation.
- It owns sandbox report JSON disk read and parse error mapping.
- It is reused through both report_api and the root compatibility bridge, so it needs a stable parent-controlled boundary.

BE-001MO-01 must establish the report_disk_loader equivalence baseline before any code movement.

## Parent Residual Queue

| Child | Current boundary | Decision |
| --- | --- | --- |
| `backend.ops_governance.sandbox.report_disk_loader` | Proposal id path guard, report file path construction, disk read, not_found mapping, and JSON parse mapping. | Select for next baseline. |

Closed children:

- `backend.ops_governance.sandbox.report_api`
- `backend.ops_governance.sandbox.verification_run`
- `backend.ops_governance.sandbox.metrics_evaluation`
- `backend.ops_governance.sandbox.comparison_metrics`
- `backend.ops_governance.sandbox.proposal_loader`

## Selected Child Boundary

`report_disk_loader` currently contains:

- `load_sandbox_report_from_disk`;
- proposal id guard for `..`, `/`, `\`, empty ids, and length greater than 128;
- `store_dir.join(format!("{}.json", proposal_id))`;
- `fs::read`;
- `json_bad_request("not_found", ...)`;
- `serde_json::from_slice`;
- `internal_error(anyhow::anyhow!("{}", error))`.

The child should return `Result<SandboxVerificationReport, (StatusCode, String)>`.

## Hard Boundaries

BE-001MO-01/02 must not move:

- proposal_loader closed leaf internals;
- report_api closed leaf internals;
- verification_run closed parent internals;
- metrics_evaluation closed leaf internals;
- comparison_metrics closed parent internals;
- root compatibility bridge exports;
- runtime mutation internals;
- AppState owner;
- release transition policy.

No sibling shortcut is allowed. The selected child must live under `sandbox` and be surfaced only through the sandbox parent boundary.

## Next Step

BE-001MO-01 backend.ops_governance.sandbox.report_disk_loader baseline_plan

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
