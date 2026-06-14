# FE-0190 - Frontend Shared Loading Skeleton Motion Closeout

Status: closed.

## Leaf Node

`frontend.design_system_styles.shared_component_primitives.loading_skeleton_motion`

## Code Changes

- Added `frontend/src/shared/loading-skeleton-motion.css`.
- Moved reduced-motion overrides, app loading shell skeleton styles, skeleton block variants, and `skeleton-pulse` keyframes from `frontend/src/shared.css` into the new partial.
- Kept `frontend/src/shared.css` as the ordered shared-style aggregator plus the remaining monthly heatmap section for the next leaf.

## Preserved Behavior

- Reduced-motion and skeleton selector bodies and order are unchanged.
- `frontend/src/app/AppShellFallback.jsx` keeps the same skeleton class contract.
- Monthly heatmap styles remain outside this leaf.

## Public Inputs

- Token values from `frontend/src/design-system/theme-tokens.css`.
- `app-loading-shell__skeleton` and `skeleton-block*` usage from app shell fallback.

## Public Outputs

- `frontend/src/shared/loading-skeleton-motion.css`
- `frontend/src/shared.css`

## Equivalence Anchor

- Expanding all local `frontend/src/shared.css` imports must reproduce `HEAD:frontend/src/shared.css` after newline normalization.

## Further-Split Decision

No deeper split is useful inside `loading_skeleton_motion` now. The reduced-motion overrides and skeleton keyframes are a compact loading affordance contract.

## Verification

- CSS expanded equivalence against `HEAD:frontend/src/shared.css`: passed.
- From `frontend/`, `npm.cmd run build`: passed.
- From repo root, `git diff --check`: passed.
- From repo root, frontend recursive state JSON parse: passed.
- From repo root, `tools/check-full-feature-tree.ps1`: passed.
- From repo root, `tools/check-matrix-governance.ps1`: passed.
