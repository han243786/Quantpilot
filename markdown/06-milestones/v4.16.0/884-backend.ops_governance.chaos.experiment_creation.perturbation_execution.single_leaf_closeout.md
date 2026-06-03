# v4.16.0 backend.ops_governance.chaos.experiment_creation.perturbation_execution single leaf closeout

> Batch: BE-001OM-03
> Node: `backend.ops_governance.chaos.experiment_creation.perturbation_execution`
> Parent: `backend.ops_governance.chaos.experiment_creation`
> Stage: `single_leaf_closeout`
> Movement: No code movement.

---

## Summary

`backend.ops_governance.chaos.experiment_creation.perturbation_execution` is closed after BE-001OM-02.

Decision:

`stop_split: true`

## Split Decision

| Rule | Result |
| --- | --- |
| Independent failure boundary | The child owns one coherent perturbation side-effect boundary: duration resolution plus disk/sleep execution. |
| Route or public boundary density | No route handler or public endpoint lives inside the child. |
| Local proof exists | Local tests cover parseable, missing, and invalid max-duration resolution without mutating process environment. |
| Parent-child communication cost | Further splitting disk pressure from sleep-only variants would increase bridges without isolating a stronger product boundary. |
| Persistence surface | No persistence surface lives inside this child. |
| Line-count-only split | Rejected: deeper split would be based mostly on variant count and file length, not a separate owner. |

## Closed Boundary

`backend.ops_governance.chaos.experiment_creation.perturbation_execution` owns:

- default max-duration constant;
- environment override parsing;
- clamped duration calculation;
- disk pressure temp directory lifecycle;
- 10 MiB pressure file write loop;
- perturbation sleep;
- disk pressure cleanup;
- latency, event loss, and clock skew sleep behavior.

Allowed call paths remain:

- experiment_creation parent create flow -> experiment_creation parent perturbation bridge;
- experiment_creation parent perturbation bridge -> private `perturbation_execution::*`;
- perturbation_execution child -> runtime fs/sleep primitives.

## Remaining Parent Residuals

Return to `backend.ops_governance.chaos.experiment_creation` parent residual judgment.

Current create-flow queue:

- `backend.ops_governance.chaos.experiment_creation.report_projection`;
- `backend.ops_governance.chaos.experiment_creation.memory_commit`.

## Hard Boundaries

Next create-flow residual batches must not move:

- closed `perturbation_execution` internals;
- closed `report_persistence` internals;
- list/detail read handlers;
- route facade;
- chaos schema type definitions;
- closed ops siblings, AppState owner, frontend caller, and release transition logic.

No sibling shortcut is allowed.

## Next Step

BE-001ON-01 backend.ops_governance.chaos.experiment_creation parent_residual_judgment

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p quantpilot chaos`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
