# v4.16.0 backend.ops_governance.alerts actual extraction complete

> Batch: BE-001MR-02
> Node: `backend.ops_governance.alerts`
> Parent: `backend.ops_governance`
> Stage: `extract_closeout`
> Movement: Code movement.

---

## Summary

BE-001MR-02 moved the alert handler owner from the crate root into the `backend.ops_governance.alerts` child.

Code movement:

- Added `src/backend/ops_governance/alerts/handlers.rs`.
- Updated `src/backend/ops_governance/alerts.rs` to call local child handlers.
- Replaced root `src/alert_engine.rs` with a compatibility bridge for startup initialization.

## Preserved Behavior

The route facade still registers:

- `GET /api/v1/alerts`
- `GET /api/v1/alerts/rules`
- `POST /api/v1/alerts/:firing_id/acknowledge`
- `POST /api/v1/alerts/check`

The moved handlers preserve:

- user-scoped alert firing lookup;
- alert rule listing;
- acknowledge-to-resolved behavior on repeated acknowledgment;
- alert check deduplication;
- automatic resolved firing cleanup;
- alert firing persistence outside write locks;
- default alert rule count and rule names;
- storage quota and atomic write behavior.

## Startup Compatibility

`src/alert_engine.rs` remains as a root compatibility bridge:

- `alert_engine::init_alert_rules(&state).await` still works for the backend startup path;
- the bridge delegates to `backend.ops_governance.alerts::init_alert_rules`;
- route registration no longer depends on the root handler owner.

## Boundary Confirmation

The extraction did not move:

- snapshots route or handler owner;
- runbook route or handler owner;
- chaos route or handler owner;
- closed hotswap internals;
- closed sandbox internals;
- AppState owner or lock order;
- DTO schema owner in `src/frontend_api_types.rs`;
- runtime/capability/storage security internals;
- release transition policy.

No sibling shortcut was introduced. The alerts child owns only its route facade, local handlers, alert helper logic, and startup initialization target.

## Next Step

BE-001MR-03 backend.ops_governance.alerts single_leaf_closeout

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p quantpilot backend::ops_governance::alerts::handlers::tests`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
