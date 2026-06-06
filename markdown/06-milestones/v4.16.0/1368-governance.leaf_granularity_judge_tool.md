# v4.16.0 leaf granularity judge tool

> Batch: GOV-LEAF-GRANULARITY-JUDGE-TOOL-01
> Node: `governance.recursive_speed_protocol.leaf_granularity_judge_tool`
> Stage: `protocol_update`
> Movement: No Rust code movement.
> Cursor: Rust recursive cursor remains BE-001ZG-03 `root.contracts.runtime_support.data_module.exchange_surface_wave` single_leaf_closeout.

---

## Summary

The read-only granularity findings are now operationalized as an automatic judge script.

`tools/evaluate-leaf-granularity.ps1` reads candidate leaf files, calculates weighted metrics, and emits one fixed decision: `STOP`, `WAVE`, `SPLIT`, or `PRECISION`. The script is read-only. It cannot move the recursive cursor, edit governance files, or replace developer judgment.

## Three Matrix Impact

| Matrix | Impact |
| --- | --- |
| Process matrix | Adds a repeatable pre-closeout scoring step so bottom-leaf decisions do not depend on memory or mood. |
| Standard matrix | Keeps high-risk surfaces in `PRECISION` and keeps low-value micro leaves in `STOP` or `WAVE`. |
| Guidance matrix | Adds a concrete tool path and score fields that milestone documents can cite as evidence. |

## Weighted Formula

Each metric is scored on a 0-100 scale.

`weighted_delta = 0.40*split_benefit + 0.20*leaf_size_fit - 0.20*risk_penalty - 0.15*governance_cost - 0.05*system_efficiency_penalty`

`normalized_split_score = clamp(40 + weighted_delta, 0, 100)`

The `40` neutral baseline prevents medium or small leaves from splitting by default. A split must earn its way above the threshold.

## Decision Rules

| Decision | Rule | Action |
| --- | --- | --- |
| `STOP` | Score under 40, helper-only micro leaf, or medium score without same-parent wave candidate. | Set `stop_split: true` unless stronger ownership evidence is provided. |
| `WAVE` | Score 40-64 with same-parent homogeneous children. | Batch under one parent wave; keep child white-box evidence. |
| `SPLIT` | Score 65+ without high-risk trigger. | Continue splitting, preferably as a standard same-parent wave. |
| `PRECISION` | Score 65+ with high-risk surface. | Use precision single-leaf governance. |

## Script Usage

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File tools\evaluate-leaf-granularity.ps1 `
  -LeafId root.contracts.runtime_support.data_module.exchange_surface_wave `
  -ParentId root.contracts.runtime_support.data_module `
  -Path qrpc_runtime/src/data_module/exchange_surface.rs `
  -Depth 4 `
  -SameParentWaveCandidate
```

Optional overrides exist for calibrated manual evidence:

- `-SplitBenefitOverride`
- `-LeafSizeFitOverride`
- `-RiskPenaltyOverride`
- `-GovernanceCostOverride`
- `-SystemEfficiencyPenaltyOverride`

Manual override is allowed only when the milestone records the reason.

## Calibration Evidence

Current smoke run:

| Candidate | Score | Decision | Meaning |
| --- | --- | --- | --- |
| `root.contracts.runtime_support.data_module.exchange_surface_wave` | 61 | `WAVE` | Medium split pressure; should stay in same-parent wave governance instead of becoming multiple standalone micro leaves. |

This matches the intended effect: the tool finds real structure, but still discourages over-splitting bottom leaves when governance and communication costs are high.

## Gates

- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\evaluate-leaf-granularity.ps1 -LeafId root.contracts.runtime_support.data_module.exchange_surface_wave -ParentId root.contracts.runtime_support.data_module -Path qrpc_runtime/src/data_module/exchange_surface.rs -Depth 4 -SameParentWaveCandidate`
- `git diff --check`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-pre-commit-hook.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\run-smart-pre-commit.ps1`

## Next Step

BE-001ZG-03 `root.contracts.runtime_support.data_module.exchange_surface_wave` single_leaf_closeout should cite the script result or explain any manual override.
