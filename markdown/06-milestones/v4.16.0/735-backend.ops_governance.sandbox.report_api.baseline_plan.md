# v4.16.0 backend.ops_governance.sandbox.report_api equivalence baseline and extraction plan

> Batch: BE-001LQ-01
> Node: `backend.ops_governance.sandbox.report_api`
> Parent: `backend.ops_governance.sandbox`
> Stage: `baseline_plan`
> Movement: No code movement.

---

## Summary

`backend.ops_governance.sandbox.report_api` is frozen as the sandbox route registration and route handler boundary.

Current owner file:

- `src/backend/ops_governance/sandbox/handlers.rs`

Current report API functions:

- `register_routes`
- `get_sandbox_report`
- `request_sandbox_verification`

BE-001LQ-02 may move only these report API functions into a dedicated child file.

## Route Baseline

| Route | Handler | Behavior |
| --- | --- | --- |
| `GET /api/v1/ai/proposals/:proposal_id/sandbox-report` | `get_sandbox_report` | Reads `state.sandbox_reports` and returns the first report whose `proposal_id` matches the path. If not found in memory, calls `load_sandbox_report_from_disk(&state.sandbox_report_store_dir, &proposal_id)`. Disk miss maps to `json_bad_request("not_found", ...)`. |
| `POST /api/v1/ai/proposals/:proposal_id/request-sandbox` | `request_sandbox_verification` | Parses `Json<RequestSandboxVerificationRequest>`, calls `run_sandbox_verification(&state, &request)`, and returns `Json<SandboxVerificationReport>`. The path `proposal_id` remains ignored in the current implementation. |

## Parent Bridge Baseline

Report API may call through the sandbox parent boundary:

- `load_sandbox_report_from_disk`
- `run_sandbox_verification`

Report API must not own:

- `run_sandbox_verification` internals;
- `load_sandbox_report_from_disk` internals;
- metric diff/verdict/warnings helpers;
- v4 replay-shape helper;
- comparison metrics/proposal lookup;
- root compatibility bridge exports.

## Allowed BE-001LQ-02 Movement

BE-001LQ-02 may:

- create `src/backend/ops_governance/sandbox/report_api.rs`;
- move `register_routes`, `get_sandbox_report`, and `request_sandbox_verification` into that file;
- update `src/backend/ops_governance/sandbox.rs` to call `report_api::register_routes`;
- make report API call runner and disk loader through the sandbox parent boundary.

BE-001LQ-02 must not:

- change route paths or HTTP methods;
- bind the ignored path `proposal_id` to request body behavior;
- change memory-first report lookup;
- change disk fallback mapping;
- change `run_sandbox_verification` behavior;
- move root compatibility bridge exports;
- move runtime mutation internals;
- move storage lifecycle owner, DTO schema owner, AppState owner, or lock order;
- propose release transition.

## Proof Gap

No dedicated route-level sandbox API smoke test was found. BE-001LQ-02 must therefore keep the movement mechanical and prove equivalence with compile and governance gates.

## Next Step

BE-001LQ-02 backend.ops_governance.sandbox.report_api extract_closeout

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
