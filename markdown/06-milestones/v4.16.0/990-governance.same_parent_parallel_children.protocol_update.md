# v4.16.0 governance same_parent_parallel_children protocol update

> Batch: GOV-SAME-PARENT-PARALLEL
> Node: `governance.recursive_speed_protocol.same_parent_parallel_children`
> Stage: `protocol_update`
> Movement: No Rust code movement.

---

## Summary

The recursive flow now allows controlled same-parent child parallelism.

This is not unrestricted concurrency. It only allows child leaves under the same frozen parent queue to proceed as a parallel wave when write sets, parent facade locks, equivalence gates, and rollback boundaries are declared before movement.

## Protocol Changes

Updated:

- `markdown/00-matrix-governance/recursive-speed-protocol.md`
- `markdown/00-matrix-governance/recursive-state.json`

The new allowed speedup is:

- `same_parent_parallel_children`

## Hard Guardrails

Same-parent parallelism must preserve:

- parent-child communication rule;
- no sibling horizontal link;
- independent white-box boundary per child;
- independent `leaf_split_decision_result` per child;
- declared write set per child;
- serialized parent facade merge when children share one parent facade file;
- no release transition unless the developer explicitly starts release transition.

## Operational Rule

Parallel child processing may combine child planning, child extraction, or child closeout into a wave, but every wave must still be verifiable and committed.

Allowed:

- one parent queue;
- multiple child baselines/extractions/closeouts in one wave;
- one final gate set plus child-specific extra gates.

Not allowed:

- mixing children from different parents;
- using a sibling helper directly;
- hiding a failed child behind a successful sibling closeout;
- bypassing `leaf_split_decision_gate`.

## Next Recursive Step

The active Rust recursive cursor remains unchanged:

BE-001RC-01 `root.contracts.qrpc_core.strategy_ir.root_validation.signal_logic_validation` baseline_plan.

## Gates

- `git diff --check`
- `cargo fmt --check`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
