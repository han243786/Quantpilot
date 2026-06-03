# v4.16.0 backend.ops_governance.chaos.experiment_creation.memory_commit actual extraction complete

> Batch: BE-001OQ-02
> Node: `backend.ops_governance.chaos.experiment_creation.memory_commit`
> Parent: `backend.ops_governance.chaos.experiment_creation`
> Stage: `extract_closeout`
> Movement: Chaos in-memory commit moved into a private child module.

---

## Summary

`backend.ops_governance.chaos.experiment_creation.memory_commit` now owns scoped-key construction and post-persistence in-memory insertion.

The experiment_creation parent keeps a local `commit_experiment_to_memory` bridge and continues to own create-flow orchestration and persistence ordering.

## Code Movement

| Previous owner | New owner | Movement |
| --- | --- | --- |
| `src/backend/ops_governance/chaos/handlers/experiment_creation.rs` | `src/backend/ops_governance/chaos/handlers/experiment_creation/memory_commit.rs` | Scoped-key construction and in-memory map insertion moved. |
| `src/backend/ops_governance/chaos/handlers/experiment_creation.rs` | `src/backend/ops_governance/chaos/handlers/experiment_creation.rs` | Parent declares the private child and keeps the local memory commit bridge. |

## Preserved Behavior

| Surface | Preserved behavior |
| --- | --- |
| Ordering | Memory insert still happens only after `persist_chaos_report` succeeds. |
| Scoped key | Key remains `auth::scoped_key(user_id, experiment_id)`. |
| Lock | The same `chaos_experiments` write lock is acquired. |
| Value | The inserted value is still a cloned `ChaosExperimentReport`. |
| Response | The API response still returns the original report after memory insert. |

## Boundary Result

No sibling shortcut was introduced.

Allowed call paths:

- experiment_creation parent create flow -> experiment_creation parent memory commit bridge;
- experiment_creation parent memory commit bridge -> private `memory_commit::*`;
- memory_commit child -> existing chaos experiment map lock.

The following remain outside this child:

- route bridge;
- experiment ID generation;
- chaos mode lifecycle;
- evidence metric sampling;
- closed perturbation_execution;
- closed report_projection;
- persistence and closed report_persistence;
- read routes, route facade, closed ops siblings, AppState owner, frontend caller, and release transition logic.

## Proof

- `cargo check -p quantpilot`
- `cargo test -p quantpilot chaos`

## Next Step

BE-001OQ-03 backend.ops_governance.chaos.experiment_creation.memory_commit single_leaf_closeout

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p quantpilot chaos`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
