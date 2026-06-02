# v4.16.0 backend.ops_governance.sandbox.verification_run actual extraction complete

> Batch: BE-001LS-02
> Node: `backend.ops_governance.sandbox.verification_run`
> Parent: `backend.ops_governance.sandbox`
> Stage: `extract_closeout`
> Movement: Code movement.

---

## Summary

BE-001LS-02 moved the reusable sandbox verification runner into a dedicated child file.

Code movement:

- Added `src/backend/ops_governance/sandbox/verification_run.rs`.
- Moved `run_sandbox_verification` out of `src/backend/ops_governance/sandbox/handlers.rs`.
- Updated `src/backend/ops_governance/sandbox.rs` to export the runner from `verification_run`.
- Exposed runner dependencies through sandbox parent-controlled helper re-exports.

## Preserved Behavior

The moved runner preserves:

- proposal lookup through `load_or_fetch_ai_proposal`;
- `RuntimeAiProposalStatus::StaticCheckPassed` gate;
- `QUANTPILOT_SANDBOX_REPLAY_WINDOW_DAYS` parsing and 30 day default;
- replay window generation;
- `compute_comparison_metrics` call;
- metric diff, verdict, and warnings calls;
- `SandboxVerificationReport` assembly;
- transient quota check for `"sandbox-reports"`;
- `persist_json` report persistence;
- `state.sandbox_reports` cache insert;
- `report_generation_count` increment with `Ordering::Relaxed`.

## Boundary Confirmation

The extraction did not move:

- report_api closed leaf internals;
- metric diff/verdict/warnings helper ownership;
- replay-shape helper ownership;
- comparison metrics/proposal lookup ownership;
- disk loader ownership;
- root compatibility bridge exports;
- runtime mutation internals;
- AppState owner or lock order;
- storage lifecycle owner;
- DTO schema owner;
- release transition policy.

No sibling shortcut was introduced. Verification run calls helper functions through sandbox parent-controlled boundaries.

## Next Step

BE-001LS-03 backend.ops_governance.sandbox.verification_run single_leaf_closeout

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
