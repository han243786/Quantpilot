# v4.16.0 backend.ops_governance.sandbox.verification_run.replay_window equivalence baseline and extraction plan

> Batch: BE-001LY-01
> Node: `backend.ops_governance.sandbox.verification_run.replay_window`
> Parent: `backend.ops_governance.sandbox.verification_run`
> Stage: `baseline_plan`
> Movement: No code movement.

---

## Summary

`backend.ops_governance.sandbox.verification_run.replay_window` is frozen as the sandbox replay window shaping boundary.

Current owner file:

- `src/backend/ops_governance/sandbox/verification_run.rs`

Current embedded block:

- `current_time_ms()`
- `format!("sbx-run-{}", now_ms)`
- `QUANTPILOT_SANDBOX_REPLAY_WINDOW_DAYS` env parsing
- 30-day default replay window
- `ReplayWindow` construction

BE-001LY-02 may move only this block into a dedicated child module under `verification_run`.

## White-Box Boundary

The child must receive no inputs.

The child must return:

- `now_ms: u64`
- `sandbox_run_id: String`
- `ReplayWindow`

The parent runner must continue using those returned values for report assembly.

## Replay Window Baseline

The extracted child must preserve this sequence exactly:

1. Create `now_ms` through `current_time_ms()`.
2. Create `sandbox_run_id = format!("sbx-run-{}", now_ms)`.
3. Read `std::env::var("QUANTPILOT_SANDBOX_REPLAY_WINDOW_DAYS")`.
4. Convert env value with `.ok().and_then(|v| v.parse().ok()).unwrap_or(30)`.
5. Build `ReplayWindow.from_ts` with:
   - `now_ms.saturating_sub(replay_days * 24 * 3600 * 1000)`
   - `epoch_ms_to_iso8601(...)`
6. Build `ReplayWindow.to_ts` with `epoch_ms_to_iso8601(now_ms)`.
7. Return `(now_ms, sandbox_run_id, replay_window)`.

The child must preserve the env var name, parse behavior, default value, arithmetic shape, and `saturating_sub` behavior.

## Parent-Child Boundary

`replay_window` may call root time and formatting helpers through `crate::*`.

It must not import or call:

- `proposal_gate`;
- `report_commit`;
- `report_api`;
- comparison metrics helpers;
- metric diff/verdict/warnings helpers;
- runtime mutation trigger;
- root compatibility bridge.

## Allowed BE-001LY-02 Movement

BE-001LY-02 may:

- create `src/backend/ops_governance/sandbox/verification_run/replay_window.rs`;
- add `mod replay_window;` inside `src/backend/ops_governance/sandbox/verification_run.rs`;
- replace the embedded replay block with `replay_window::build_replay_window()`;
- keep `replay_window` private to the `verification_run` parent.

BE-001LY-02 must not:

- move proposal_gate closed leaf internals;
- move comparison metric computation;
- move metric diff/verdict/warnings helper ownership;
- move `SandboxVerificationReport` assembly;
- move report_commit closed leaf internals;
- expose `replay_window` through the sandbox parent facade;
- change env var or default-day semantics;
- propose release transition.

## Split Decision Gate

After BE-001LY-02, BE-001LY-03 must run single-leaf closeout.

Expected decision: `stop_split: true`, because the child will own one compact time-window shape. Continue splitting only if extraction reveals a concrete owner with independent behavior beyond env parsing and replay window construction.

## Next Step

BE-001LY-02 backend.ops_governance.sandbox.verification_run.replay_window extract_closeout

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
