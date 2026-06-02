# v4.16.0 backend.ops_governance parent residual judgment selects sandbox

> Batch: BE-001LN-01
> Node: `backend.ops_governance`
> Parent: `backend`
> Stage: `parent_residual_judgment`
> Movement: No code movement.

---

## Summary

`backend.ops_governance` returns to parent residual judgment after hotswap was closed in BE-001LM-03.

The next child is fixed as:

`backend.ops_governance.sandbox`

Selection reasons:

- It is the next route function after hotswap in the ops governance parent facade.
- The child facade currently delegates to a root registrar, so handler owner is still outside the child boundary.
- The sandbox routes are part of AI proposal governance and deserve an isolated baseline before movement.

## Parent Residual Queue

| Child | Current boundary | Decision |
| --- | --- | --- |
| `backend.ops_governance.hotswap` | Route facade and handlers moved under hotswap child. | Closed. |
| `backend.ops_governance.sandbox` | `src/backend/ops_governance/sandbox.rs` delegates to `crate::sandbox_verification::register_sandbox_verification_routes`. | Select for next baseline. |
| `backend.ops_governance.alerts` | Delegates alert route registration to `crate::alert_engine`. | Keep in parent residual queue. |
| `backend.ops_governance.snapshots` | Delegates snapshot route registration to `crate::snapshot_service`. | Keep in parent residual queue. |
| `backend.ops_governance.runbook` | Delegates runbook route registration to `crate::runbook`. | Keep in parent residual queue. |
| `backend.ops_governance.chaos` | Delegates chaos route registration to `crate::chaos_experiment`. | Keep in parent residual queue. |

## Selected Child Boundary

`backend.ops_governance.sandbox` currently contains:

- `MODULE_ID = "backend.ops_governance.sandbox"`
- `register_routes(router: Router<AppState>) -> Router<AppState>`
- parent delegation to `crate::sandbox_verification::register_sandbox_verification_routes(router)`

The root sandbox registrar currently owns:

- `GET /api/v1/ai/proposals/:proposal_id/sandbox-report`
- `POST /api/v1/ai/proposals/:proposal_id/request-sandbox`

The next baseline must freeze report lookup, request creation, persisted report behavior, AppState access, and proposal governance boundaries before any code movement.

## Hard Boundaries

BE-001LO-01/02 must not move:

- hotswap closed leaf internals;
- alert, snapshot, runbook, or chaos handlers;
- runtime/capability/storage security internals;
- AppState owner or lock order;
- test support internals;
- release transition policy.

No sibling shortcut is allowed. Sandbox may communicate outward only through its parent route facade and its frozen root handler boundary until the dedicated baseline changes ownership.

## Next Step

BE-001LO-01 backend.ops_governance.sandbox baseline_plan

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
