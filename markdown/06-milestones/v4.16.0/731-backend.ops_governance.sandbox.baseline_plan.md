# v4.16.0 backend.ops_governance.sandbox equivalence baseline and extraction plan

> Batch: BE-001LO-01
> Node: `backend.ops_governance.sandbox`
> Parent: `backend.ops_governance`
> Stage: `baseline_plan`
> Movement: No code movement.

---

## Summary

`backend.ops_governance.sandbox` is frozen as a route facade plus root sandbox verification implementation boundary.

The current child facade lives at `src/backend/ops_governance/sandbox.rs` and delegates to `crate::sandbox_verification::register_sandbox_verification_routes(router)`.

The current implementation owner remains `src/sandbox_verification.rs`.

BE-001LO-02 may move the sandbox implementation under the sandbox child, but it must preserve the existing root compatibility surface because `runtime.mutation.ai_proposal.sandbox_trigger` already calls the sandbox runner and disk loader.

## Route Chain

| Layer | File | Boundary |
| --- | --- | --- |
| app router | `src/app_router.rs` | Calls `interface_boundary::register_sandbox_verification_routes(router)`. |
| interface boundary | `src/backend/interface_boundary.rs` | Bridges sandbox routes into ops governance. |
| ops governance bridge | `src/backend/interface_boundary/ops_governance_bridge.rs` | Calls `crate::backend::ops_governance::register_sandbox_verification_routes(router)`. |
| ops governance parent | `src/backend/ops_governance.rs` | Calls `sandbox::register_routes(router)`. |
| sandbox child facade | `src/backend/ops_governance/sandbox.rs` | Delegates to root sandbox registrar. |
| root implementation owner | `src/sandbox_verification.rs` | Owns route registrar, handlers, runner, metric helpers, disk loader, and tests. |

## Route Baseline

| Route | Handler | Behavior |
| --- | --- | --- |
| `GET /api/v1/ai/proposals/:proposal_id/sandbox-report` | `get_sandbox_report` | Reads `state.sandbox_reports`, falls back to `load_sandbox_report_from_disk`, returns `SandboxVerificationReport` or `not_found` bad request. |
| `POST /api/v1/ai/proposals/:proposal_id/request-sandbox` | `request_sandbox_verification` | Parses `RequestSandboxVerificationRequest`, calls `run_sandbox_verification`, returns `SandboxVerificationReport`. |

## Core Baseline

`run_sandbox_verification` must preserve:

- `load_or_fetch_ai_proposal` lookup behavior.
- `RuntimeAiProposalStatus::StaticCheckPassed` gate.
- `QUANTPILOT_SANDBOX_REPLAY_WINDOW_DAYS` default of 30 days.
- replay window generation with `epoch_ms_to_iso8601`.
- `compute_comparison_metrics`, `compute_metrics_diff`, `determine_sandbox_verdict`, and `compute_sandbox_warnings`.
- storage quota check for `"sandbox-reports"` as transient lifecycle.
- `persist_json` into `state.sandbox_report_store_dir`.
- memory cache insert into `state.sandbox_reports`.
- `evidence_metrics.report_generation_count` increment.

`load_sandbox_report_from_disk` must preserve:

- proposal id path traversal rejection;
- empty id rejection;
- id length cap of 128;
- `not_found` bad request mapping;
- JSON parse error mapping through `internal_error`.

## External Compatibility Boundary

`src/runtime/mutation/ai_proposal/sandbox_trigger.rs` calls:

- `sandbox_verification::load_sandbox_report_from_disk`
- `sandbox_verification::run_sandbox_verification`

BE-001LO-02 must keep that root compatibility surface available. The preferred movement is:

- move implementation into `src/backend/ops_governance/sandbox/handlers.rs`;
- make `src/backend/ops_governance/sandbox.rs` call local handlers for route registration;
- keep `src/sandbox_verification.rs` as a small compatibility bridge for `run_sandbox_verification` and `load_sandbox_report_from_disk`.

This avoids moving runtime mutation internals during the ops governance sandbox extraction.

## Test Baseline

Existing local tests are embedded in `src/sandbox_verification.rs` and cover:

- `compute_metrics_diff`
- `determine_sandbox_verdict`
- all eight diff metrics presence
- v4 artifact replay shape verdict

No dedicated route-level sandbox API smoke test was found in this baseline search.

## Hard Boundaries

BE-001LO-02 must not move:

- hotswap closed leaf internals;
- alerts, snapshots, runbook, or chaos handlers;
- `runtime.mutation.ai_proposal.sandbox_trigger` implementation;
- AppState owner or lock order;
- storage lifecycle owner;
- DTO schema owner in `src/frontend_api_types.rs`;
- test support internals;
- release transition policy.

No sibling shortcut is allowed. Runtime mutation keeps using the root sandbox compatibility bridge until a separate baseline changes that boundary.

## Next Step

BE-001LO-02 backend.ops_governance.sandbox extract_closeout

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
