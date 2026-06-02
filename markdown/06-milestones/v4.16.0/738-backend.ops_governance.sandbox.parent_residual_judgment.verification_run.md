# v4.16.0 backend.ops_governance.sandbox parent residual judgment selects verification_run

> Batch: BE-001LR-01
> Node: `backend.ops_governance.sandbox`
> Parent: `backend.ops_governance`
> Stage: `parent_residual_judgment`
> Movement: No code movement.

---

## Summary

`backend.ops_governance.sandbox` returns to parent residual judgment after report API was closed in BE-001LQ-03.

The next child is fixed as:

`backend.ops_governance.sandbox.verification_run`

Selection reasons:

- It is the next concrete owner after report API in `src/backend/ops_governance/sandbox/handlers.rs`.
- It exposes the reusable `run_sandbox_verification` boundary used by report API and runtime mutation through the root compatibility bridge.
- It owns verification orchestration side effects: proposal status gate, replay window, metric comparison, report persistence, cache insert, and evidence metric increment.

BE-001LS-01 must establish the verification_run equivalence baseline before any code movement.

## Parent Residual Queue

| Child | Current boundary | Decision |
| --- | --- | --- |
| `backend.ops_governance.sandbox.report_api` | Route registration and GET/POST route handlers. | Closed. |
| `backend.ops_governance.sandbox.verification_run` | `run_sandbox_verification`. | Select for next baseline. |
| `backend.ops_governance.sandbox.metrics_verdict` | `compute_metrics_diff`, `format_diff`, `determine_sandbox_verdict`, and `compute_sandbox_warnings`. | Keep in parent residual queue. |
| `backend.ops_governance.sandbox.replay_shape` | `compare_v4_backtest_artifact_replay_shape` and `count_v4_risk_rejections`. | Keep in parent residual queue. |
| `backend.ops_governance.sandbox.comparison_metrics` | `compute_comparison_metrics`, `backtest_to_sandbox_metrics`, and `load_or_fetch_ai_proposal`. | Keep in parent residual queue. |
| `backend.ops_governance.sandbox.disk_loader` | `load_sandbox_report_from_disk`. | Keep in parent residual queue. |

## Selected Child Boundary

`backend.ops_governance.sandbox.verification_run` currently contains:

- `run_sandbox_verification(state, request)`
- proposal lookup through `load_or_fetch_ai_proposal`
- `RuntimeAiProposalStatus::StaticCheckPassed` gate
- replay window generation
- comparison metrics call
- metric diff/verdict/warnings calls
- `SandboxVerificationReport` assembly
- transient sandbox report storage quota check
- report persistence through `persist_json`
- memory cache insert into `state.sandbox_reports`
- evidence metric increment

The child may call comparison, metric, loader, and persistence helpers through the sandbox parent boundary until later child baselines move those owners.

## Hard Boundaries

BE-001LS-01/02 must not move:

- report_api closed leaf internals;
- metric diff/verdict/warnings helpers;
- replay-shape helper internals;
- comparison metric/proposal lookup internals unless explicitly frozen in the baseline;
- disk loader internals;
- root compatibility bridge exports;
- runtime mutation internals;
- AppState owner or lock order;
- storage lifecycle owner;
- DTO schema owner;
- release transition policy.

No sibling shortcut is allowed. Verification run may call helpers only through sandbox parent-controlled boundaries until those helpers get their own child baselines.

## Next Step

BE-001LS-01 backend.ops_governance.sandbox.verification_run baseline_plan

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
