# v4.16.0 backend.ops_governance.sandbox.report_api actual extraction complete

> Batch: BE-001LQ-02
> Node: `backend.ops_governance.sandbox.report_api`
> Parent: `backend.ops_governance.sandbox`
> Stage: `extract_closeout`
> Movement: Code movement.

---

## Summary

BE-001LQ-02 moved the sandbox report API route owner into a dedicated child file.

Code movement:

- Added `src/backend/ops_governance/sandbox/report_api.rs`.
- Moved `register_routes`, `get_sandbox_report`, and `request_sandbox_verification` out of `src/backend/ops_governance/sandbox/handlers.rs`.
- Updated `src/backend/ops_governance/sandbox.rs` to call `report_api::register_routes`.
- Kept runner and disk loader ownership in the sandbox parent implementation boundary.

## Preserved Behavior

The report API still registers:

- `GET /api/v1/ai/proposals/:proposal_id/sandbox-report`
- `POST /api/v1/ai/proposals/:proposal_id/request-sandbox`

The moved handlers preserve:

- memory-first lookup through `state.sandbox_reports`;
- disk fallback through `load_sandbox_report_from_disk`;
- `json_bad_request("not_found", ...)` mapping on disk miss;
- ignored path `proposal_id` for POST request body behavior;
- `run_sandbox_verification(&state, &request)` call;
- `Json<SandboxVerificationReport>` response type.

## Boundary Confirmation

The extraction did not move:

- `run_sandbox_verification`;
- `load_sandbox_report_from_disk`;
- metric diff/verdict/warnings helpers;
- v4 replay-shape helper;
- comparison metrics/proposal lookup;
- root compatibility bridge exports;
- runtime mutation internals;
- AppState owner or lock order;
- storage lifecycle owner;
- DTO schema owner;
- release transition policy.

No sibling shortcut was introduced. Report API calls runner and disk loader through the sandbox parent boundary.

## Next Step

BE-001LQ-03 backend.ops_governance.sandbox.report_api single_leaf_closeout

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
