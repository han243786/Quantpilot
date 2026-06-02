# v4.16.0 backend.ops_governance.sandbox.verification_run.replay_window actual extraction complete

> Batch: BE-001LY-02
> Node: `backend.ops_governance.sandbox.verification_run.replay_window`
> Parent: `backend.ops_governance.sandbox.verification_run`
> Stage: `extract_closeout`
> Movement: Code movement completed.

---

## Summary

`backend.ops_governance.sandbox.verification_run.replay_window` has been extracted into a private child module under the verification runner.

New owner file:

- `src/backend/ops_governance/sandbox/verification_run/replay_window.rs`

Updated parent file:

- `src/backend/ops_governance/sandbox/verification_run.rs`

The parent runner now calls `replay_window::build_replay_window()` and receives `(now_ms, sandbox_run_id, replay_window)` for report assembly.

## Preserved Behavior

BE-001LY-02 preserves:

- `current_time_ms()` timestamp creation;
- `format!("sbx-run-{}", now_ms)` sandbox run id;
- `QUANTPILOT_SANDBOX_REPLAY_WINDOW_DAYS` env var name;
- `.ok().and_then(|v| v.parse().ok()).unwrap_or(30)` parse/default behavior;
- 30-day default;
- `now_ms.saturating_sub(replay_days * 24 * 3600 * 1000)` arithmetic;
- `epoch_ms_to_iso8601` conversion for both `from_ts` and `to_ts`;
- parent runner report assembly semantics.

## Parent-Child Boundary

`replay_window` is private to `verification_run`.

It is not exposed by:

- `src/backend/ops_governance/sandbox.rs`
- `src/backend/ops_governance/sandbox/report_api.rs`
- `src/sandbox_verification.rs`

No sibling child imports were introduced.

## Non-Movement

BE-001LY-02 did not move:

- proposal_gate closed leaf internals;
- comparison metric computation;
- metric diff, verdict, or warning computation;
- `SandboxVerificationReport` assembly;
- report_commit closed leaf internals;
- route handler behavior;
- disk report loader behavior;
- runtime mutation trigger behavior;
- AppState owner or storage lifecycle owner;
- release transition policy.

## Next Step

BE-001LY-03 backend.ops_governance.sandbox.verification_run.replay_window single_leaf_closeout

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
