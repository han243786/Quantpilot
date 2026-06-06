# v4.16.0 leaf granularity smart judge

> Batch: GOV-LEAF-GRANULARITY-SMART-JUDGE-01
> Node: `governance.recursive_speed_protocol.leaf_granularity_smart_judge`
> Stage: `protocol_update`
> Movement: No Rust code movement.
> Cursor: Rust recursive cursor remains BE-001ZE-01 `root.contracts.runtime_support.data_module.collection_orchestration` baseline_plan.

---

## Summary

The read-only granularity audit is integrated into recursive governance as a mandatory terminal-leaf scoring rule.

The goal is to keep final modularization quality while preventing over-splitting. A bottom leaf is no longer split because it can be split; it is split only when the score proves that the split improves ownership, verification, coupling, or parent complexity enough to justify governance cost and system overhead.

## Read-Only Findings

The audit found three governance pressure signals:

1. Many already closed recursive children exist, so the flow needs stronger terminal-leaf discipline.
2. Rust file sizes include many small files, so micro leaves can become more costly than useful.
3. v4.16 milestone count is high, so repeated baseline/extract/closeout/parent-residual documents must be replaced by `STOP`, `WAVE`, or batch evidence when split benefit is weak.

## Scoring Model

| Metric | Weight | Meaning |
| --- | --- | --- |
| `split_benefit` | 40 | Independent owner, input/output clarity, testability, parent complexity reduction, coupling reduction. |
| `leaf_size_fit` | 20 | LOC, function count, public surface, and branch density. |
| `risk_penalty` | 20 | Public API, route/schema, state machine, trading semantics, persistence, lock, security, live execution, cross-crate, or compiler-contract risk. |
| `governance_cost` | 15 | Milestone, facade, gate, index, commit, and proof cost. |
| `system_efficiency_penalty` | 5 | Thin wrapper, useless re-export/import, parent-child forwarding, or readability loss. |

## Decision Outputs

| Decision | Rule | Action |
| --- | --- | --- |
| `STOP` | Low net score, LOC under 100 without strong benefit, or helper-only fragment. | Mark `stop_split: true`. |
| `WAVE` | Medium net score with same-parent homogeneous children. | Use `same_parent_wave`, not standalone full governance. |
| `SPLIT` | Strong net score and low risk. | Continue split, preferably batched under the same parent. |
| `PRECISION` | Strong net score with high-risk surface. | Use single-leaf precision governance. |

## Terminal Leaf Size Rule

Ordinary logic leaves should normally end at 150-600 LOC or one cohesive business transaction.

- LOC under 100 defaults to `STOP`.
- LOC under 200 with high governance cost defaults to `WAVE` or `STOP`.
- LOC over 800 with high complexity enters split evaluation.
- Single functions, thin helpers, thin accessors, DTO pockets, pure re-exports, and thin facades do not justify further split unless they own a public contract, state machine, security, persistence, route/schema, live execution, or compiler lowering boundary.

## Governance Impact

This protocol reduces governance load by making the scoring decision part of the recursive gate. It preserves final effect by forcing high-risk surfaces into Precision mode while routing low-value micro-splits to `STOP` or `WAVE`.

## Gates

- `git diff --check`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
