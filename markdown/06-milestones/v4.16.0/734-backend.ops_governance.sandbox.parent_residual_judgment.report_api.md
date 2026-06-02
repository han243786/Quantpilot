# v4.16.0 backend.ops_governance.sandbox parent residual judgment selects report_api

> Batch: BE-001LP-01
> Node: `backend.ops_governance.sandbox`
> Parent: `backend.ops_governance`
> Stage: `parent_residual_judgment`
> Movement: No code movement.

---

## Summary

`backend.ops_governance.sandbox` returns to parent residual judgment after BE-001LO-03 confirmed `stop_split: false`.

The next child is fixed as:

`backend.ops_governance.sandbox.report_api`

Selection reasons:

- It is the first concrete owner in `src/backend/ops_governance/sandbox/handlers.rs`.
- It owns route registration plus GET/POST route handlers.
- It bridges API requests into the already frozen runner and disk loader without owning their internals.

BE-001LQ-01 must establish the report API equivalence baseline before any code movement.

## Parent Residual Queue

| Child | Current boundary | Decision |
| --- | --- | --- |
| `backend.ops_governance.sandbox.report_api` | `register_routes`, `get_sandbox_report`, and `request_sandbox_verification`. | Select for next baseline. |
| `backend.ops_governance.sandbox.verification_run` | `run_sandbox_verification` plus proposal gate, replay window, persistence, cache update, and evidence metric side effect. | Keep in parent residual queue. |
| `backend.ops_governance.sandbox.metrics_verdict` | `compute_metrics_diff`, `format_diff`, `determine_sandbox_verdict`, and `compute_sandbox_warnings`. | Keep in parent residual queue. |
| `backend.ops_governance.sandbox.replay_shape` | `compare_v4_backtest_artifact_replay_shape` and `count_v4_risk_rejections`. | Keep in parent residual queue. |
| `backend.ops_governance.sandbox.comparison_metrics` | `compute_comparison_metrics`, `backtest_to_sandbox_metrics`, and `load_or_fetch_ai_proposal`. | Keep in parent residual queue. |
| `backend.ops_governance.sandbox.disk_loader` | `load_sandbox_report_from_disk`. | Keep in parent residual queue. |

## Selected Child Boundary

`backend.ops_governance.sandbox.report_api` currently contains:

- `register_routes(router: Router<AppState>) -> Router<AppState>`
- `GET /api/v1/ai/proposals/:proposal_id/sandbox-report`
- `POST /api/v1/ai/proposals/:proposal_id/request-sandbox`
- `get_sandbox_report`
- `request_sandbox_verification`

The child may call:

- `load_sandbox_report_from_disk`
- `run_sandbox_verification`

It must not own or move those helpers during the report API baseline.

## Hard Boundaries

BE-001LQ-01/02 must not move:

- verification runner internals;
- metric diff/verdict/warnings internals;
- replay-shape helper internals;
- comparison metric/proposal lookup internals;
- disk loader internals;
- root compatibility bridge exports;
- runtime mutation internals;
- AppState owner or lock order;
- storage lifecycle owner;
- DTO schema owner;
- release transition policy.

No sibling shortcut is allowed. Report API may call runner and disk loader only through the current sandbox parent boundary until the dedicated child baselines move those owners.

## Next Step

BE-001LQ-01 backend.ops_governance.sandbox.report_api baseline_plan

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
