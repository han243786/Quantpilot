# FE-0189 - Frontend Shared Legacy QP Aliases Closeout

Status: closed.

## Leaf Node

`frontend.design_system_styles.shared_component_primitives.legacy_qp_aliases`

## Code Changes

- Added `frontend/src/shared/legacy-qp-aliases.css`.
- Moved legacy `qp-*` compatibility selectors from `frontend/src/shared.css` into the new partial.
- Kept `frontend/src/shared.css` as the ordered shared-style aggregator plus the remaining skeleton/motion and heatmap sections for later leaves.

## Preserved Behavior

- `qp-*` selector bodies and order are unchanged.
- Legacy operational pages that still use `qp-card`, `qp-badge`, `qp-btn`, `qp-table`, `qp-input`, `qp-timeline`, and `qp-fade-in` keep the same compatibility surface.
- `ad-*` primitives remain in `frontend/src/shared/ad-core-primitives.css`.

## Public Inputs

- Token values from `frontend/src/design-system/theme-tokens.css`.
- Legacy `qp-*` class usage from operational pages and approval/alert/runbook/snapshot surfaces.

## Public Outputs

- `frontend/src/shared/legacy-qp-aliases.css`
- `frontend/src/shared.css`

## Equivalence Anchor

- Expanding all local `frontend/src/shared.css` imports must reproduce `HEAD:frontend/src/shared.css` after newline normalization.

## Further-Split Decision

No deeper split is useful inside `legacy_qp_aliases` now. The aliases form one compatibility contract and should move as a unit until callers are intentionally migrated.

## Verification

- CSS expanded equivalence against `HEAD:frontend/src/shared.css`: passed.
- From `frontend/`, `npm.cmd run build`: passed.
- From repo root, `git diff --check`: passed.
- From repo root, frontend recursive state JSON parse: passed.
- From repo root, `tools/check-full-feature-tree.ps1`: passed.
- From repo root, `tools/check-matrix-governance.ps1`: passed.
