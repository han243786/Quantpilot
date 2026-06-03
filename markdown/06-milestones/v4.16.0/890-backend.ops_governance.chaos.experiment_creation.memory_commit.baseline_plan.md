# v4.16.0 backend.ops_governance.chaos.experiment_creation.memory_commit equivalence baseline and extraction plan

> Batch: BE-001OQ-01
> Node: `backend.ops_governance.chaos.experiment_creation.memory_commit`
> Parent: `backend.ops_governance.chaos.experiment_creation`
> Stage: `baseline_plan`
> Movement: No code movement.

---

## Summary

`backend.ops_governance.chaos.experiment_creation.memory_commit` is frozen as the post-persistence in-memory commit owner for chaos experiment creation.

BE-001OQ-01 does not move code. It defines the exact baseline and allowed movement for BE-001OQ-02.

## Current Owner

Current implementation owner:

- `src/backend/ops_governance/chaos/handlers/experiment_creation.rs`

Current selected boundary:

- scoped key construction with `auth::scoped_key`;
- write lock acquisition on the existing `chaos_experiments` map;
- insertion of a cloned report under the scoped key.

The parent bridge must remain:

- create-flow parent calls a local memory commit bridge after persistence succeeds;
- memory_commit child receives only the existing map lock reference and does not own AppState.

## Frozen Behavior

| Surface | Frozen behavior |
| --- | --- |
| Ordering | Memory insert still happens only after `persist_chaos_report` succeeds. |
| Scoped key | Key remains `auth::scoped_key(user_id, experiment_id)`. |
| Lock | The same `chaos_experiments` write lock is acquired. |
| Value | The inserted value is still a cloned `ChaosExperimentReport`. |
| Response | The API response still returns the original report after memory insert. |

## Allowed BE-001OQ-02 Movement

BE-001OQ-02 may:

- create `src/backend/ops_governance/chaos/handlers/experiment_creation/memory_commit.rs`;
- add a private `mod memory_commit;` declaration in `src/backend/ops_governance/chaos/handlers/experiment_creation.rs`;
- move only scoped-key construction and in-memory map insertion into that private child;
- keep a parent-owned `commit_experiment_to_memory` bridge in `experiment_creation.rs`;
- pass only the existing `chaos_experiments` map lock reference, user id, experiment id, and report.

## Forbidden BE-001OQ-02 Movement

BE-001OQ-02 must not move or rewrite:

- create-flow route bridge;
- experiment ID generation;
- chaos mode lifecycle;
- evidence metric sampling;
- closed `perturbation_execution` internals;
- closed `report_projection` internals;
- parent-mediated persistence;
- closed `report_persistence` internals;
- list/detail read handlers;
- route facade;
- chaos schema type definitions;
- AppState field ownership or lock order beyond this existing write lock;
- closed ops siblings, frontend caller, and release transition logic.

## Parent-Child Rule

Allowed call paths:

- experiment_creation parent create flow -> experiment_creation parent memory commit bridge;
- experiment_creation parent memory commit bridge -> private `memory_commit::*`;
- memory_commit child -> existing chaos experiment map lock.

Forbidden call path:

Any memory_commit child importing perturbation execution, report projection, report persistence, read routes, route facade, or ops-governance siblings.

## Proof

BE-001OQ-02 must prove equivalence with:

- `cargo test -p quantpilot chaos`
- `cargo check -p quantpilot`

## Next Step

BE-001OQ-02 backend.ops_governance.chaos.experiment_creation.memory_commit extract_closeout

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p quantpilot chaos`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
