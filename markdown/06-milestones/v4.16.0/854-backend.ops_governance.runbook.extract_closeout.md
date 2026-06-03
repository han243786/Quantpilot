# v4.16.0 backend.ops_governance.runbook actual extraction complete

> Batch: BE-001NX-02
> Node: `backend.ops_governance.runbook`
> Parent: `backend.ops_governance`
> Stage: `extract_closeout`
> Movement: Runbook route and scenario catalog handler owner moved under backend ops governance.

---

## Summary

`backend.ops_governance.runbook` now owns runbook route registration, default scenario catalog construction, list/detail handlers, and embedded catalog tests.

`src/runbook.rs` remains only as a root compatibility bridge into the backend ops child owner.

## Code Movement

| Previous owner | New owner | Movement |
| --- | --- | --- |
| `src/runbook.rs` | `src/backend/ops_governance/runbook/handlers.rs` | Runbook route registration, catalog builder, list/detail handlers, and tests moved. |
| `src/backend/ops_governance/runbook.rs` | `src/backend/ops_governance/runbook.rs` | Child facade now delegates to private local handlers. |
| `src/runbook.rs` | `src/runbook.rs` | Root file shrunk to a compatibility bridge calling `backend.ops_governance.runbook::register_routes`. |

## Preserved Behavior

| Surface | Preserved behavior |
| --- | --- |
| Route facade | `GET /api/v1/runbook` still lists all default scenarios. |
| Detail route | `GET /api/v1/runbook/:scenario_id` still returns one matching scenario. |
| Not found | Missing scenario still returns `json_bad_request("not_found", ...)`. |
| Catalog size | Default runbook still contains six scenarios. |
| Catalog integrity | Each scenario still has diagnostic steps, recovery steps, and verification text. |
| Scenario identity | Scenario IDs remain unique. |
| Root compatibility | Existing root callers can still use `src/runbook.rs` compatibility bridge. |

## Boundary Result

No sibling shortcut was introduced.

Allowed call paths:

- `backend.ops_governance.runbook::register_routes` -> private `runbook::handlers::register_runbook_routes`;
- `src/runbook.rs` compatibility bridge -> `backend.ops_governance.runbook::register_routes`.

The following remain outside this child:

- chaos route and handler owner;
- closed hotswap, sandbox, alerts, and snapshots internals;
- AppState owner or lock order;
- alert severity and runbook schema type definitions;
- runtime/capability/storage security internals;
- frontend caller;
- release transition logic.

## Proof

- `cargo check -p quantpilot`

## Next Step

BE-001NX-03 backend.ops_governance.runbook single_leaf_closeout

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p quantpilot runbook`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
