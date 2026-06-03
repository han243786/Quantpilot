# v4.16.0 backend.ops_governance parent residual judgment selects chaos

> Batch: BE-001OF-01
> Node: `backend.ops_governance`
> Parent: `backend`
> Stage: `parent_residual_judgment`
> Movement: No code movement.

---

## Summary

`backend.ops_governance` returns to parent residual judgment after `runbook` closed as a completed parent module.

The next child is fixed as:

`backend.ops_governance.chaos`

## Parent Residual Queue

| Child | Current boundary | Decision |
| --- | --- | --- |
| `backend.ops_governance.chaos` | Chaos route facade plus experiment creation, listing, detail loading, persistence, ID validation, and tests still live behind `src/chaos_experiment.rs`. | Select for next baseline. |
| parent-level ops governance wiring | Shared facade registration after all child modules close. | Keep in parent residual queue until chaos closes. |

## Selected Child Boundary

`backend.ops_governance.chaos` currently spans:

- `src/backend/ops_governance/chaos.rs` facade;
- `src/chaos_experiment.rs` compatibility implementation owner;
- chaos route registration;
- create/list/get experiment handlers;
- chaos mode toggling and perturbation execution;
- report persistence and disk loading;
- experiment ID validation;
- chaos enum/spec smoke tests.

## Hard Boundaries

BE-001OG-01/02 must not move:

- closed hotswap internals;
- closed sandbox internals;
- closed alerts internals;
- closed snapshots internals;
- closed runbook internals;
- AppState owner or lock order;
- schema type definitions, frontend caller, or release transition logic.

No sibling shortcut is allowed.

## Next Step

BE-001OG-01 backend.ops_governance.chaos baseline_plan

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p quantpilot chaos`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
