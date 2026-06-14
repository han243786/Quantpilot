# FE-0185 - Frontend Design System Overlays Resizers Motion Closeout

Status: closed.

## Leaf Node

`frontend.design_system_styles.design_tokens_and_native_controls.overlays_resizers_motion`

## Code Changes

- Added `frontend/src/design-system/overlays-resizers-motion.css`.
- Moved skip-link, offline banner, panel divider, command palette, and reduced-motion styles from `frontend/src/design-system.css` into the new partial.
- Reduced `frontend/src/design-system.css` to an ordered design-system import aggregator.

## Preserved Behavior

- Overlay, accessibility skip-link, divider drag affordance, command palette, and reduced-motion rules keep the same selector bodies and order after the previous design-system partials.
- `frontend/src/design-system.css` still imports every design-system partial before any rule statements.
- No selector body was changed.

## Public Inputs

- `ad-*` class usage from global overlays, command palette affordances, resizable panels, sidebar, tabbar, and fade-in surfaces.
- Token values from `frontend/src/design-system/theme-tokens.css`.

## Public Outputs

- `frontend/src/design-system/overlays-resizers-motion.css`
- `frontend/src/design-system.css`

## Equivalence Anchor

- Expanding all local `frontend/src/design-system.css` imports must reproduce `HEAD:frontend/src/design-system.css` after newline normalization.

## Further-Split Decision

No deeper split is useful inside `overlays_resizers_motion` now. The remaining design-system rules are compact global affordances with shared overlay/resizer/motion behavior and no separate component ownership large enough to justify another recursive layer.

## Verification

- CSS expanded equivalence against `HEAD:frontend/src/design-system.css`: passed.
- From `frontend/`, `npm.cmd run build`: passed.
- From repo root, `git diff --check`: passed.
- From repo root, frontend recursive state JSON parse: passed.
- From repo root, `tools/check-full-feature-tree.ps1`: passed.
- From repo root, `tools/check-matrix-governance.ps1`: passed.
