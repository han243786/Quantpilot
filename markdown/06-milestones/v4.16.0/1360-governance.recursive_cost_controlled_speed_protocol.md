# v4.16.0 recursive cost-controlled speed protocol

> Batch: GOV-RECURSIVE-COST-CONTROL-01
> Node: `governance.recursive_speed_protocol.cost_controlled_recursive_speed`
> Stage: `protocol_update`
> Movement: No Rust code movement.
> Cursor: Rust recursive cursor remains BE-001ZE-01 `root.contracts.runtime_support.data_module.collection_orchestration` baseline_plan.

---

## Summary

The recursive modularization flow is upgraded from `recursive-high-speed-v1` to `recursive-high-speed-v2`.

The upgrade keeps final quality fixed and reduces cost by allowing a same-parent batch to count as one verifiable step when every child keeps independent evidence. This does not weaken parent-child communication, release-transition protection, module tree coverage, full-feature-tree coverage, or `leaf_split_decision_gate`.

## Three Matrix Impact

| Matrix | Impact |
| --- | --- |
| Process matrix | Adds cost-controlled `same_parent_wave` as a standard recursive step when a frozen parent queue has independent children. |
| Standard matrix | Adds forced precision downgrade triggers for public API, route, schema, persistence, lock, state-machine, security, live-trading, compiler-contract, and sibling-private dependency risk. |
| Guidance matrix | Requires batch white-box child rows so module tree and full-feature-tree evidence remain child-level even when milestone documents are batched. |

## Effect-Non-Regression Invariants

1. Parent-child communication remains hard law; no sibling horizontal link is introduced.
2. AI still cannot propose release transition unless the developer explicitly starts release transition.
3. Public API, route, schema, persistence, lock owner, state-machine semantics, and external contract changes force Precision single-leaf mode.
4. Every child in a batch must keep independent baseline boundary, write set, movement note, equivalence gate, closeout decision, split decision, and residual record.
5. Parent closeout still requires all children closed, residual facade explained, module tree synchronized, full-feature-tree synchronized, and gates passed.

## Execution Modes

| Mode | Use When | Output |
| --- | --- | --- |
| Precision single-leaf | Risk is high or equivalence cannot be shared. | Separate baseline, extraction, closeout, and parent residual documents. |
| Standard same-parent wave | Same parent, frozen queue, independent child write sets, shared gate is sufficient. | One milestone document with child-level white-box rows and one verifiable wave commit. |
| Fast closeout | No code movement or tiny terminal facade with clear `stop_split: true`. | One closeout document or one child row inside the batch document. |

## Batch White-Box Minimum

Every `same_parent_wave` must record:

- child coordinate;
- baseline boundary;
- write set;
- movement;
- equivalence gate;
- closeout decision;
- residuals.

If any child cannot satisfy the row, the batch is split and that child returns to Precision single-leaf.

## Current Application

Current Rust cursor stays unchanged. The next implementation cycle should first evaluate whether the remaining `root.contracts.runtime_support.data_module` children can form one or more same-parent waves. Candidate children are still subject to the downgrade triggers before movement.

## Gates

- `git diff --check`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
