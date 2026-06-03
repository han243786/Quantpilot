# v4.16.0 backend.ops_governance.chaos single leaf closeout continues split

> Batch: BE-001OG-03
> Node: `backend.ops_governance.chaos`
> Parent: `backend.ops_governance`
> Stage: `single_leaf_closeout`
> Movement: No code movement.

---

## Summary

`backend.ops_governance.chaos` is equivalent after BE-001OG-02, but should continue splitting internally.

Decision:

`stop_split: false`

## Split Decision

| Rule | Result |
| --- | --- |
| Independent failure boundary | Report persistence, disk loading, and experiment ID validation are a separate file-I/O failure boundary. |
| Route or public boundary density | Route facade and create/list/detail handlers are still mixed in one handler owner. |
| Local proof exists | Existing chaos tests moved with the owner and `cargo test -p quantpilot chaos` passes. |
| Parent-child communication cost | Parent bridges can keep create/detail handlers from calling persistence children directly. |
| Persistence surface | Persistence is present and should be isolated before deeper create/read handler work. |
| Line-count-only split | Accepted only with boundary evidence: this is I/O and validation responsibility, not arbitrary line count. |

## Current Residuals

| Candidate | Decision |
| --- | --- |
| `backend.ops_governance.chaos.report_persistence` | Select next. Owns chaos report persistence, disk loading, and experiment ID validation. |
| `backend.ops_governance.chaos.experiment_creation` | Keep in queue. Owns create handler, chaos mode toggling, perturbation execution, metrics, and report assembly. |
| `backend.ops_governance.chaos.read_routes` | Keep in queue. Owns list/detail read handlers after persistence is separated. |
| `backend.ops_governance.chaos.route_facade` | Keep in queue. Owns route registration after handlers are separated. |

## Hard Boundaries

Next chaos child batches must not move:

- closed hotswap internals;
- closed sandbox internals;
- closed alerts internals;
- closed snapshots internals;
- closed runbook internals;
- AppState owner or lock order;
- chaos schema type definitions;
- frontend caller;
- release transition logic.

No sibling shortcut is allowed. Create/detail handlers must consume persistence through the chaos parent bridge until their own baseline changes ownership.

## Next Step

BE-001OH-01 backend.ops_governance.chaos parent_residual_judgment

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p quantpilot chaos`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
