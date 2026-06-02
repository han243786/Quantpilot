# v4.16.0 backend.ops_governance parent residual judgment selects hotswap

> Batch: BE-001LL-01
> Node: `backend.ops_governance`
> Parent: `backend`
> Stage: `parent_residual_judgment`
> Movement: No code movement.

---

## Summary

`backend.ops_governance` returns to parent residual judgment after BE-001LK-03 confirmed `stop_split: false`.

The next child is fixed as:

`backend.ops_governance.hotswap`

Selection reasons:

- It is the first real child facade in the parent route order.
- It owns a direct route facade instead of delegating to another aggregate registrar.
- Its current route surface is small enough to freeze before deciding whether the root handler owner should move.

BE-001LM-01 must establish the hotswap equivalence baseline before any code movement.

## Parent Residual Queue

| Child | Current boundary | Decision |
| --- | --- | --- |
| `backend.ops_governance.hotswap` | `src/backend/ops_governance/hotswap.rs` registers `/api/hotswap`, `/api/hotswap/list`, and `/api/hotswap/:hotswap_id`. | Select for next baseline. |
| `backend.ops_governance.sandbox` | Delegates sandbox verification route registration to `crate::sandbox_verification`. | Keep in parent residual queue. |
| `backend.ops_governance.alerts` | Delegates alert route registration to `crate::alert_engine`. | Keep in parent residual queue. |
| `backend.ops_governance.snapshots` | Delegates snapshot route registration to `crate::snapshot_service`. | Keep in parent residual queue. |
| `backend.ops_governance.runbook` | Delegates runbook route registration to `crate::runbook`. | Keep in parent residual queue. |
| `backend.ops_governance.chaos` | Delegates chaos route registration to `crate::chaos_experiment`. | Keep in parent residual queue. |

## Selected Child Boundary

`backend.ops_governance.hotswap` currently contains:

- `MODULE_ID = "backend.ops_governance.hotswap"`
- `register_routes(router: Router<AppState>) -> Router<AppState>`
- `POST /api/hotswap -> crate::hotswap_api::submit_hotswap`
- `GET /api/hotswap/list -> crate::hotswap_api::list_hotswaps`
- `GET /api/hotswap/:hotswap_id -> crate::hotswap_api::get_hotswap_status`

The child is only a route facade today. The handler owner remains `src/hotswap_api.rs`.

## Hard Boundaries

BE-001LM-01/02 must not move:

- `src/sandbox_verification.rs`
- `src/alert_engine.rs`
- `src/snapshot_service.rs`
- `src/runbook.rs`
- `src/chaos_experiment.rs`
- AppState owner or lock order
- runtime/capability/storage security internals
- test support internals
- sibling ops route handlers
- release transition policy

No sibling shortcut is allowed. Hotswap may communicate outward only through its parent route facade or root handler boundary until a dedicated baseline changes ownership.

## Next Step

BE-001LM-01 backend.ops_governance.hotswap baseline_plan

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
