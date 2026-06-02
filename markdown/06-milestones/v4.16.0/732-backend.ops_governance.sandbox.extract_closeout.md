# v4.16.0 backend.ops_governance.sandbox actual extraction complete

> Batch: BE-001LO-02
> Node: `backend.ops_governance.sandbox`
> Parent: `backend.ops_governance`
> Stage: `extract_closeout`
> Movement: Code movement.

---

## Summary

BE-001LO-02 moved the sandbox verification implementation into the `backend.ops_governance.sandbox` child while preserving the root compatibility bridge required by runtime mutation.

Code movement:

- Moved the implementation body into `src/backend/ops_governance/sandbox/handlers.rs`.
- Updated `src/backend/ops_governance/sandbox.rs` to call local handlers.
- Replaced root sandbox module content with a compatibility bridge in `src/sandbox_verification.rs`.
- Preserved `sandbox_verification::run_sandbox_verification` and `sandbox_verification::load_sandbox_report_from_disk` for existing runtime mutation callers.

## Preserved Behavior

The route facade still registers:

- `GET /api/v1/ai/proposals/:proposal_id/sandbox-report`
- `POST /api/v1/ai/proposals/:proposal_id/request-sandbox`

The moved implementation preserves:

- report memory lookup through `state.sandbox_reports`;
- disk fallback through `state.sandbox_report_store_dir`;
- request handler call to `run_sandbox_verification`;
- `RuntimeAiProposalStatus::StaticCheckPassed` gate;
- replay window default and environment override;
- comparison metric diff and verdict logic;
- warning generation;
- transient storage quota check for sandbox reports;
- `persist_json` report persistence;
- `evidence_metrics.report_generation_count` increment;
- proposal id path validation in the disk loader;
- embedded unit tests for metric diff and verdict behavior.

## Compatibility Bridge

`src/sandbox_verification.rs` now only bridges the runtime compatibility APIs:

- `run_sandbox_verification`
- `load_sandbox_report_from_disk`

Route registration now lives in the sandbox child facade. The root bridge keeps `src/runtime/mutation/ai_proposal/sandbox_trigger.rs` behavior unchanged and avoids widening the sandbox extraction into runtime mutation.

## Boundary Confirmation

The extraction did not move:

- hotswap closed leaf internals;
- alerts, snapshots, runbook, or chaos handlers;
- `runtime.mutation.ai_proposal.sandbox_trigger` implementation;
- AppState owner or lock order;
- storage lifecycle owner;
- DTO schema owner in `src/frontend_api_types.rs`;
- test support internals;
- release transition policy.

No sibling shortcut was introduced. Runtime mutation continues to use the root compatibility bridge, and sandbox route ownership now lives under the ops governance sandbox child.

## Next Step

BE-001LO-03 backend.ops_governance.sandbox single_leaf_closeout

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
