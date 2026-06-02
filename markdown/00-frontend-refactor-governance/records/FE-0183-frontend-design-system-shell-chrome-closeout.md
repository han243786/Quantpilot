# FE-0183 - Frontend Design System Shell Chrome Styles Closeout

Status: closed.

## Leaf Node

`frontend.design_system_styles.design_tokens_and_native_controls.shell_chrome_styles`

## Code Changes

- Added `frontend/src/design-system/shell-chrome.css`.
- Moved app main content, custom titlebar, sidebar, sidebar brand, sidebar sections, and sidebar item styles from `frontend/src/design-system.css` into the new partial.
- Kept `frontend/src/design-system.css` as the ordered design-system aggregator with all imports before rule statements.

## Preserved Behavior

- Shell chrome rules remain after reset, tokens, scrollbar, alias, and focus/selection imports.
- Workspace header, tabbar, pill, command bar, overlay, resizer, and motion rules remain outside this leaf.
- No selector body was changed.

## Public Inputs

- `ad-*` class usage from app shell, route host, titlebar, sidebar, and shell navigation components.
- Token values from `frontend/src/design-system/theme-tokens.css`.

## Public Outputs

- `frontend/src/design-system/shell-chrome.css`
- `frontend/src/design-system.css`

## Equivalence Anchor

- Expanding all local `frontend/src/design-system.css` imports must reproduce `HEAD:frontend/src/design-system.css` after newline normalization.

## Further-Split Decision

No deeper split is useful inside `shell_chrome_styles` now. It is one coherent app chrome leaf: main content offset, titlebar, and sidebar behavior.

## Verification

- CSS expanded equivalence against `HEAD:frontend/src/design-system.css`: passed.
- From `frontend/`, `npm.cmd run build`: passed.
- From repo root, `git diff --check`: passed.
- From repo root, frontend recursive state JSON parse: passed.
- From repo root, `tools/check-full-feature-tree.ps1`: passed.
- From repo root, `tools/check-matrix-governance.ps1`: passed.
