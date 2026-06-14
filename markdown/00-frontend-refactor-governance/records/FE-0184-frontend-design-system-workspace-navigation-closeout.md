# FE-0184 - Frontend Design System Workspace Navigation Styles Closeout

Status: closed.

## Leaf Node

`frontend.design_system_styles.design_tokens_and_native_controls.workspace_navigation_primitives`

## Code Changes

- Added `frontend/src/design-system/workspace-navigation.css`.
- Moved workspace header, tabbar, tab focus pulse, status pill, and floating command bar styles from `frontend/src/design-system.css` into the new partial.
- Kept `frontend/src/design-system.css` as the ordered design-system aggregator with all imports before rule statements.

## Preserved Behavior

- Workspace navigation rules remain after reset, tokens, scrollbar, legacy alias, focus/selection, and shell chrome imports.
- Skip-link, offline banner, panel divider, command palette, and reduced-motion rules remain outside this leaf for the next overlay/resizer/motion leaf.
- No selector body was changed.

## Public Inputs

- `ad-*` class usage from workspace headers, tab navigation, status pills, and command bar affordances.
- Token values from `frontend/src/design-system/theme-tokens.css`.

## Public Outputs

- `frontend/src/design-system/workspace-navigation.css`
- `frontend/src/design-system.css`

## Equivalence Anchor

- Expanding all local `frontend/src/design-system.css` imports must reproduce `HEAD:frontend/src/design-system.css` after newline normalization.

## Further-Split Decision

No deeper split is useful inside `workspace_navigation_primitives` now. Header, tabbar, pill, and command bar styles are one compact workspace-navigation primitive set and share the same design-token contract.

## Verification

- CSS expanded equivalence against `HEAD:frontend/src/design-system.css`: passed.
- From `frontend/`, `npm.cmd run build`: passed.
- From repo root, `git diff --check`: passed.
- From repo root, frontend recursive state JSON parse: passed.
- From repo root, `tools/check-full-feature-tree.ps1`: passed.
- From repo root, `tools/check-matrix-governance.ps1`: passed.
