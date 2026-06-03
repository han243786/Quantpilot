# v4.16.0 backend.ops_governance.chaos actual extraction complete

> Batch: BE-001OG-02
> Node: `backend.ops_governance.chaos`
> Parent: `backend.ops_governance`
> Stage: `extract_closeout`
> Movement: Chaos implementation moved into a private backend child module.

---

## Summary

`backend.ops_governance.chaos` now owns the chaos experiment implementation under the backend ops governance tree.

The root `src/chaos_experiment.rs` remains as a compatibility bridge into `backend.ops_governance.chaos`.

## Code Movement

| Previous owner | New owner | Movement |
| --- | --- | --- |
| `src/chaos_experiment.rs` | `src/backend/ops_governance/chaos/handlers.rs` | Route registration, create/list/get handlers, perturbation execution, persistence helpers, ID validation, and tests moved. |
| `src/backend/ops_governance/chaos.rs` | `src/backend/ops_governance/chaos.rs` | Facade now delegates to local handlers instead of root implementation. |
| `src/chaos_experiment.rs` | `src/chaos_experiment.rs` | Reduced to a compatibility bridge delegating to backend chaos facade. |

## Preserved Behavior

| Surface | Preserved behavior |
| --- | --- |
| Route paths | `/api/v1/chaos/experiments` and `/api/v1/chaos/experiments/:experiment_id` remain unchanged. |
| Create route | Creates an experiment ID, toggles chaos mode during execution, records metrics, persists report, and stores report in memory. |
| List route | Lists user-scoped in-memory reports sorted by `executed_at` descending. |
| Detail route | Reads user-scoped in-memory report first, then falls back to disk loading. |
| Disk loading | Validates experiment ID and loads `*.json` from chaos storage. |
| Root compatibility | `src/chaos_experiment.rs` still delegates to backend chaos facade. |

## Boundary Result

No sibling shortcut was introduced.

Allowed call paths:

- ops governance parent -> `backend.ops_governance.chaos::register_routes`;
- chaos facade -> private chaos handlers;
- root compatibility bridge -> backend chaos facade.

The following remain outside this child:

- closed hotswap, sandbox, alerts, snapshots, and runbook internals;
- AppState owner or lock order;
- chaos schema type definitions;
- frontend caller;
- release transition logic.

## Proof

- `cargo check -p quantpilot`
- `cargo test -p quantpilot chaos`

## Next Step

BE-001OG-03 backend.ops_governance.chaos single_leaf_closeout

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p quantpilot chaos`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
