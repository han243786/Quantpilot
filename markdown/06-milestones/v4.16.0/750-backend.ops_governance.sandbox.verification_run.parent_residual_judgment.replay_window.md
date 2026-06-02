# v4.16.0 backend.ops_governance.sandbox.verification_run parent residual judgment selects replay_window

> Batch: BE-001LX-01
> Node: `backend.ops_governance.sandbox.verification_run`
> Parent: `backend.ops_governance.sandbox`
> Stage: `parent_residual_judgment`
> Movement: No code movement.

---

## Summary

`backend.ops_governance.sandbox.verification_run` returns to parent residual judgment after `proposal_gate` closed with `stop_split: true`.

The next child is fixed as:

`backend.ops_governance.sandbox.verification_run.replay_window`

Selection reasons:

- It owns sandbox run id and replay time-window shaping.
- It includes environment-variable parsing with a stable default.
- It is independent from proposal eligibility, comparison metrics, report assembly, and report commit.
- It has clearer behavior than the remaining report assembly block and can be extracted mechanically.

BE-001LY-01 must establish the replay_window equivalence baseline before any code movement.

## Parent Residual Queue

| Child | Current boundary | Decision |
| --- | --- | --- |
| `backend.ops_governance.sandbox.verification_run.replay_window` | `now_ms`, sandbox run id, env replay-days parsing, `ReplayWindow` generation. | Select for next baseline. |
| `backend.ops_governance.sandbox.verification_run.report_assembly` | `SandboxVerificationReport` construction from computed values. | Keep in parent residual queue. |

Closed children:

- `backend.ops_governance.sandbox.verification_run.report_commit`
- `backend.ops_governance.sandbox.verification_run.proposal_gate`

## Selected Child Boundary

`replay_window` currently contains:

- `current_time_ms()`
- `format!("sbx-run-{}", now_ms)`
- `std::env::var("QUANTPILOT_SANDBOX_REPLAY_WINDOW_DAYS")`
- `.ok().and_then(|v| v.parse().ok()).unwrap_or(30)`
- `ReplayWindow { from_ts, to_ts }`
- `epoch_ms_to_iso8601(now_ms.saturating_sub(replay_days * 24 * 3600 * 1000))`
- `epoch_ms_to_iso8601(now_ms)`

The child should return the generated `now_ms`, `sandbox_run_id`, and `ReplayWindow`.

## Hard Boundaries

BE-001LY-01/02 must not move:

- proposal_gate closed leaf internals;
- comparison metrics;
- metric diff/verdict/warnings helper ownership;
- report assembly;
- report_commit closed leaf internals;
- report_api closed leaf internals;
- disk loader ownership;
- root compatibility bridge exports;
- runtime mutation internals;
- AppState owner or storage lifecycle owner;
- release transition policy.

No sibling shortcut is allowed. Replay window must live under `verification_run` and be called only by its parent runner.

## Next Step

BE-001LY-01 backend.ops_governance.sandbox.verification_run.replay_window baseline_plan

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
