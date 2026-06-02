# v4.16.0 backend.ops_governance.sandbox.verification_run equivalence baseline and extraction plan

> Batch: BE-001LS-01
> Node: `backend.ops_governance.sandbox.verification_run`
> Parent: `backend.ops_governance.sandbox`
> Stage: `baseline_plan`
> Movement: No code movement.

---

## Summary

`backend.ops_governance.sandbox.verification_run` is frozen as the reusable sandbox verification runner boundary.

Current owner file:

- `src/backend/ops_governance/sandbox/handlers.rs`

Current runner:

- `run_sandbox_verification(state, request)`

BE-001LS-02 may move only this runner into a dedicated child file while preserving all helper ownership and side effects.

## Runner Baseline

`run_sandbox_verification` must preserve this sequence:

1. Load proposal through `load_or_fetch_ai_proposal(state, &request.proposal_id)`.
2. Require `RuntimeAiProposalStatus::StaticCheckPassed`.
3. Create `now_ms` through `current_time_ms()`.
4. Create `sandbox_run_id = format!("sbx-run-{}", now_ms)`.
5. Read `QUANTPILOT_SANDBOX_REPLAY_WINDOW_DAYS`, parse as `u64`, default to 30.
6. Build `ReplayWindow` through `epoch_ms_to_iso8601`.
7. Call `compute_comparison_metrics(state, &ai_proposal).await`.
8. Call `compute_metrics_diff`.
9. Call `determine_sandbox_verdict`.
10. Call `compute_sandbox_warnings`.
11. Assemble `SandboxVerificationReport`.
12. Check storage quota with layer `"sandbox-reports"` and `StorageLifecycle::Transient`.
13. Persist report with `persist_json(&state.sandbox_report_store_dir, &report.proposal_id, &report)`.
14. Insert report into `state.sandbox_reports` under `request.proposal_id.clone()`.
15. Increment `state.evidence_metrics.report_generation_count` with `Ordering::Relaxed`.
16. Return the report.

## External Callers

The runner is called by:

- `src/backend/ops_governance/sandbox/report_api.rs`
- `src/runtime/mutation/ai_proposal/sandbox_trigger.rs` through the root compatibility bridge `src/sandbox_verification.rs`

BE-001LS-02 must keep both callers equivalent.

## Parent Bridge Baseline

Verification run may call through sandbox parent-controlled boundaries:

- `load_or_fetch_ai_proposal`
- `compute_comparison_metrics`
- `compute_metrics_diff`
- `determine_sandbox_verdict`
- `compute_sandbox_warnings`

BE-001LS-02 must not import a sibling child directly. If helper visibility changes are needed, expose them through the sandbox parent boundary.

## Allowed BE-001LS-02 Movement

BE-001LS-02 may:

- create `src/backend/ops_governance/sandbox/verification_run.rs`;
- move `run_sandbox_verification` into that file;
- update `src/backend/ops_governance/sandbox.rs` to export the moved runner through the same public boundary;
- adjust helper visibility only enough for parent-controlled calls.

BE-001LS-02 must not:

- move report_api closed leaf internals;
- move metric diff/verdict/warnings helper ownership;
- move replay-shape helper ownership;
- move comparison metrics/proposal lookup ownership;
- move disk loader ownership;
- move root compatibility bridge exports beyond keeping them pointed at the same runner;
- move runtime mutation internals;
- change AppState owner or lock order;
- change storage lifecycle owner;
- change DTO schema owner;
- propose release transition.

## Proof Coverage

Existing embedded tests cover metric diff/verdict/replay-shape helpers, not route-level runner side effects. Therefore BE-001LS-02 must keep the runner movement mechanical and prove equivalence with compile, format, UTF-8, full-tree, and matrix governance gates.

## Next Step

BE-001LS-02 backend.ops_governance.sandbox.verification_run extract_closeout

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
