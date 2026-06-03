# v4.16.0 backend.ops_governance.chaos.experiment_creation single leaf closeout continues split

> Batch: BE-001OK-03
> Node: `backend.ops_governance.chaos.experiment_creation`
> Parent: `backend.ops_governance.chaos`
> Stage: `single_leaf_closeout`
> Movement: No code movement.

---

## Summary

`backend.ops_governance.chaos.experiment_creation` is equivalent after BE-001OK-02, but should continue splitting internally.

Decision:

`stop_split: false`

## Split Decision

| Rule | Result |
| --- | --- |
| Independent failure boundary | The child still mixes state lifecycle, side-effect perturbation execution, metric projection, pass/fail criteria, report assembly, and memory insertion. |
| Route or public boundary density | Route-facing extraction remains parent-owned; deeper split should not create new public route surfaces. |
| Local proof exists | Current chaos tests prove the existing type/persistence surface, but create-flow internals still lack targeted local proof. |
| Parent-child communication cost | A perturbation child can be parent-mediated from the create-flow parent without introducing sibling shortcuts. |
| Persistence surface | Persistence is already isolated in the closed `report_persistence` child and must stay parent-mediated. |
| Line-count-only split | Rejected: the next split is justified by side-effect ownership and failure isolation, not file length alone. |

## Residual Queue

| Child | Decision |
| --- | --- |
| `backend.ops_governance.chaos.experiment_creation.perturbation_execution` | Select next. Owns max-duration resolution and the disk pressure, latency, event loss, and clock skew perturbation side effects. |
| `backend.ops_governance.chaos.experiment_creation.report_projection` | Keep in queue. Owns metrics before/during/after, pass criteria, alerts, degradation actions, and report assembly. |
| `backend.ops_governance.chaos.experiment_creation.memory_commit` | Keep in queue. Owns scoped in-memory insertion after persistence succeeds. |

## Current Closed Surface

The extracted create-flow child owns:

- route create implementation behind the parent bridge;
- experiment ID generation;
- chaos mode enable/disable lifecycle;
- evidence metric sampling;
- perturbation dispatch;
- metric/report projection;
- parent-mediated persistence;
- in-memory insertion.

## Hard Boundaries

Next create-flow residual batches must not move:

- closed `report_persistence` internals;
- list/detail read handlers;
- route facade;
- chaos schema type definitions;
- closed ops siblings, AppState owner, frontend caller, and release transition logic.

No sibling shortcut is allowed.

## Next Step

BE-001OL-01 backend.ops_governance.chaos.experiment_creation parent_residual_judgment

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p quantpilot chaos`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
