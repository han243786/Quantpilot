# v4.16.0 backend.ops_governance.hotswap actual extraction complete

> Batch: BE-001LM-02
> Node: `backend.ops_governance.hotswap`
> Parent: `backend.ops_governance`
> Stage: `extract_closeout`
> Movement: Code movement.

---

## Summary

BE-001LM-02 moved the hotswap handler owner from the crate root into the `backend.ops_governance.hotswap` child.

Code movement:

- Added `src/backend/ops_governance/hotswap/handlers.rs`.
- Updated `src/backend/ops_governance/hotswap.rs` to call local child handlers.
- Removed root module declaration `mod hotswap_api;` from `src/lib.rs`.
- Deleted `src/hotswap_api.rs`.

## Preserved Behavior

The route facade still registers:

- `POST /api/hotswap`
- `GET /api/hotswap/list`
- `GET /api/hotswap/:hotswap_id`

The moved handlers preserve:

- `auth::UserId` extraction and `auth::scoped_key` storage lookup.
- `AppState.hotswap_records` read/write owner and lock order.
- `HOTSWAP_NO_TARGETS` validation.
- `HOTSWAP_EMPTY_MODULE_KEY` validation.
- `200 OK` hotswap submit response shape.
- `404 NOT_FOUND` hotswap missing-record problem JSON.
- list projection fields: `hotswap_id`, `status`, `step`, `started_at_ms`, `success`.

## Boundary Confirmation

The extraction did not move:

- sandbox verification routes or handlers;
- alert routes or handlers;
- snapshot routes or handlers;
- runbook routes or handlers;
- chaos routes or handlers;
- AppState owner or lock order;
- runtime/capability/storage security internals;
- test support internals;
- DTO schema owner in `src/frontend_api_types.rs`;
- release transition policy.

No sibling shortcut was introduced. The hotswap child owns only its route facade and local handlers.

## Next Step

BE-001LM-03 backend.ops_governance.hotswap single_leaf_closeout

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
