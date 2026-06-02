# FE-0188 - Frontend Shared AD Core Primitives Closeout

Status: closed.

## Leaf Node

`frontend.design_system_styles.shared_component_primitives.ad_core_primitives`

## Code Changes

- Added `frontend/src/shared/ad-core-primitives.css`.
- Moved the shared `ad-*` card, badge, button, state, table, metric, input, divider, timeline, page, and `ad-fade-in` animation styles from `frontend/src/shared.css` into the new partial.
- Kept `frontend/src/shared.css` as the shared-style aggregator plus the remaining legacy alias, skeleton, and heatmap sections for later leaves.

## Preserved Behavior

- `ad-*` selector bodies and order are unchanged.
- Legacy `qp-*` aliases, reduced-motion/skeleton styles, and monthly heatmap styles remain outside this leaf.
- `frontend/src/styleEntrypoint.js` continues importing `frontend/src/shared.css`.

## Public Inputs

- Token values from `frontend/src/design-system/theme-tokens.css`.
- `ad-*` class usage from app shell, components, and pages.

## Public Outputs

- `frontend/src/shared/ad-core-primitives.css`
- `frontend/src/shared.css`

## Equivalence Anchor

- Expanding all local `frontend/src/shared.css` imports must reproduce `HEAD:frontend/src/shared.css` after newline normalization.

## Further-Split Decision

No deeper split is useful inside `ad_core_primitives` now. The moved selectors are the base shared `ad-*` primitive set and are best kept together so consumers inherit one compact shared component contract.

## Verification

- CSS expanded equivalence against `HEAD:frontend/src/shared.css`: passed.
- From `frontend/`, `npm.cmd run build`: passed.
- From repo root, `git diff --check`: passed.
- From repo root, frontend recursive state JSON parse: passed.
- From repo root, `tools/check-full-feature-tree.ps1`: passed.
- From repo root, `tools/check-matrix-governance.ps1`: passed.
