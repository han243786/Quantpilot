# v4.16.0 backend.ops_governance.chaos equivalence baseline and extraction plan

> Batch: BE-001OG-01
> Node: `backend.ops_governance.chaos`
> Parent: `backend.ops_governance`
> Stage: `baseline_plan`
> Movement: No code movement.

---

## Summary

`backend.ops_governance.chaos` is frozen as the chaos experiment route and report owner.

BE-001OG-01 does not move code. It defines the exact baseline and allowed movement for BE-001OG-02.

## Current Owner

Current facade owner:

- `src/backend/ops_governance/chaos.rs`

Current implementation owner:

- `src/chaos_experiment.rs`

Current selected boundary:

- chaos route registration;
- create/list/get experiment handlers;
- chaos mode toggling;
- perturbation execution for four experiment types;
- steady-state metric projection;
- report assembly;
- report persistence and disk loading;
- experiment ID validation;
- chaos enum/spec tests.

## Frozen Behavior

| Surface | Frozen behavior |
| --- | --- |
| Route paths | `/api/v1/chaos/experiments` and `/api/v1/chaos/experiments/:experiment_id` remain unchanged. |
| Create route | Creates an experiment ID, toggles chaos mode during execution, records metrics, persists report, and stores report in memory. |
| List route | Lists user-scoped in-memory reports sorted by `executed_at` descending. |
| Detail route | Reads user-scoped in-memory report first, then falls back to disk loading. |
| Disk loading | Validates experiment ID and loads `*.json` from chaos storage. |
| Root compatibility | `src/chaos_experiment.rs` remains callable as the legacy bridge. |

## Allowed BE-001OG-02 Movement

BE-001OG-02 may:

- create `src/backend/ops_governance/chaos/handlers.rs`;
- move chaos route registration, create/list/get handlers, persistence helpers, ID validation, and tests into that private backend child;
- update `src/backend/ops_governance/chaos.rs` to delegate to local handlers;
- reduce `src/chaos_experiment.rs` to a compatibility bridge delegating to `backend.ops_governance.chaos`;
- preserve all route paths, handler signatures, persistence path rules, and test expectations.

## Forbidden BE-001OG-02 Movement

BE-001OG-02 must not move or rewrite:

- closed hotswap internals;
- closed sandbox internals;
- closed alerts internals;
- closed snapshots internals;
- closed runbook internals;
- AppState owner or lock order;
- chaos schema type definitions;
- frontend caller;
- release transition logic.

## Parent-Child Rule

Allowed call paths:

- ops governance parent -> `backend.ops_governance.chaos::register_routes`;
- chaos facade -> private chaos handlers;
- root compatibility bridge -> backend chaos facade.

Forbidden call path:

Any closed ops sibling importing or calling chaos handlers directly.

## Proof

BE-001OG-02 must prove equivalence with:

- `cargo test -p quantpilot chaos`
- `cargo check -p quantpilot`

## Next Step

BE-001OG-02 backend.ops_governance.chaos extract_closeout

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p quantpilot chaos`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
