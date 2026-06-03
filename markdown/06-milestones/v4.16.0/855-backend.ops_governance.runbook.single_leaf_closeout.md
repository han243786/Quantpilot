# v4.16.0 backend.ops_governance.runbook single leaf closeout continues split

> Batch: BE-001NX-03
> Node: `backend.ops_governance.runbook`
> Parent: `backend.ops_governance`
> Stage: `single_leaf_closeout`
> Movement: No code movement.

---

## Summary

`backend.ops_governance.runbook` is equivalent after BE-001NX-02, but should continue splitting internally.

Decision:

`stop_split: false`

## Split Decision

| Rule | Result |
| --- | --- |
| Independent failure boundary | The default scenario catalog is an independent data/catalog owner. |
| Route or public boundary density | Route facade plus list/detail handlers are still mixed with catalog construction. |
| Local proof exists | Existing runbook tests already target catalog size, integrity, and ID uniqueness. |
| Parent-child communication cost | A parent bridge from read handlers to scenario catalog is cheap and clear. |
| Persistence surface | No persistence responsibility belongs here. |
| Line-count-only split | Accepted only with boundary evidence: the large block is static catalog data, not arbitrary length. |

## Current Residuals

| Candidate | Decision |
| --- | --- |
| `backend.ops_governance.runbook.scenario_catalog` | Select next. Owns default runbook construction and catalog tests. |
| `backend.ops_governance.runbook.read_routes` | Keep in queue. Owns list/detail handlers after catalog is separated. |
| `backend.ops_governance.runbook.route_facade` | Keep as parent-owned route surface for now. |

## Hard Boundaries

Next runbook child batches must not move:

- chaos route or handler owner;
- closed hotswap, sandbox, alerts, or snapshots internals;
- AppState owner or lock order;
- alert severity or runbook schema type definitions;
- runtime/capability/storage security internals;
- frontend caller;
- release transition logic.

No sibling shortcut is allowed. Read handlers must consume scenario catalog through the runbook parent bridge until their own baseline changes ownership.

## Next Step

BE-001NY-01 backend.ops_governance.runbook parent_residual_judgment

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test -p quantpilot runbook`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
