# v4.16.0 terminal leaf control v2

> Batch: GOV-TERMINAL-LEAF-CONTROL-V2-01
> Node: `governance.recursive_speed_protocol.terminal_leaf_control_v2`
> Stage: `protocol_update`
> Movement: No Rust code movement.
> Cursor: Rust recursive cursor remains BE-001ZI-03 `root.contracts.runtime_support.data_module.normalization` single_leaf_closeout.

---

## Summary

This batch integrates the read-only governance findings into the active recursive governance system.

The finding is simple: bottom leaves that are too small can make the system slower to govern, harder to review, and less efficient to navigate. The recursive flow must therefore decide leaf size with a weighted rule instead of continuing to split just because a function or helper can be isolated.

## Read-Only Findings Integrated

| Finding | Governance response |
| --- | --- |
| Bottom leaves can become too fine. | `terminal_leaf_control_v2` defines size buckets and a target ordinary leaf range of 150-600 LOC. |
| Governance can become heavier than the refactor. | `governance_mode` routes low-value leaves to `stop_split` or `same_parent_wave`. |
| Standalone four-step leaf governance is expensive. | `standalone_full_governance_allowed` is true only for `precision_single_leaf`. |
| High-risk surfaces still need careful handling. | `PRECISION` remains mandatory for public contract, route/schema, state machine, persistence, lock, security, live execution, or compiler contract risk. |
| The rule needs automation, not memory. | `tools/evaluate-leaf-granularity.ps1` now emits a `terminal_leaf_control` block. |

## Script Output Contract

The leaf granularity judge must emit:

| Field | Required meaning |
| --- | --- |
| `target_terminal_loc_range` | `150-600` |
| `size_bucket` | One of `micro_under_100`, `small_100_149`, `terminal_target_150_600`, `split_pressure_601_800`, `oversized_over_800` |
| `governance_mode` | One of `stop_split`, `same_parent_wave`, `standard_same_parent_wave`, `precision_single_leaf` |
| `standalone_full_governance_allowed` | True only for `precision_single_leaf` |
| `micro_leaf_default_stop` | True when micro leaf split value is not strong enough |
| `high_cost_leaf_needs_wave_or_stop` | True when a small leaf should not receive standalone four-step governance |

## Decision Rules

| Decision | Governance mode | Effect |
| --- | --- | --- |
| `STOP` | `stop_split` | Close the leaf and do not create another baseline/extract/closeout chain. |
| `WAVE` | `same_parent_wave` | Batch homogeneous same-parent children while preserving child white-box evidence. |
| `SPLIT` | `standard_same_parent_wave` | Continue splitting, but prefer same-parent wave governance over standalone leaf governance. |
| `PRECISION` | `precision_single_leaf` | Use full single-leaf governance because risk is high enough to justify it. |

## Calibration Evidence

Current smoke run:

| Candidate | Score | Decision | Governance mode | Meaning |
| --- | --- | --- | --- | --- |
| `root.contracts.runtime_support.data_module.normalization` | 30 | `STOP` | `stop_split` | Small cohesive normalization leaf should close instead of becoming another micro-split chain. |

## Three Matrix Impact

| Matrix | Impact |
| --- | --- |
| Process matrix | Adds a mandatory terminal leaf control block before bottom-leaf closeout or further split. |
| Standard matrix | Turns leaf size, governance cost, and system efficiency into explicit weighted constraints. |
| Guidance matrix | Updates recursive state, protocol docs, module tree references, and matrix gate tokens. |

## Gates

- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\evaluate-leaf-granularity.ps1 -LeafId root.contracts.runtime_support.data_module.normalization -ParentId root.contracts.runtime_support.data_module -Path qrpc_runtime/src/data_module/normalization.rs -Depth 4`
- `git diff --check`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\run-smart-pre-commit.ps1`
