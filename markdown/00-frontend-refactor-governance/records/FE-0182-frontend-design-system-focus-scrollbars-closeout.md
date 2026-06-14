# FE-0182 - Frontend Design System Focus Selection Scrollbars Closeout

Status: closed.

## Leaf Node

`frontend.design_system_styles.design_tokens_and_native_controls.focus_selection_scrollbars`

## Code Changes

- Added `frontend/src/design-system/scrollbars.css` for Webkit scrollbar selectors.
- Added `frontend/src/design-system/focus-selection.css` for `:focus-visible` and `::selection`.
- Updated `frontend/src/design-system.css` so all CSS `@import` statements remain at the top of the file.

## Preserved Behavior

- Scrollbar rules still sit after theme tokens and before legacy aliases after import expansion.
- Focus and selection rules still sit after legacy aliases and before app shell styles after import expansion.
- The coupled extraction was required because CSS `@import` must precede non-import rules; leaving a mid-file import produced a PostCSS warning.

## Public Inputs

- Browser focus-visible and selection pseudo-element behavior.
- Webkit scrollbar pseudo-elements.
- Token values supplied by `frontend/src/design-system/theme-tokens.css`.

## Public Outputs

- `frontend/src/design-system/scrollbars.css`
- `frontend/src/design-system/focus-selection.css`
- `frontend/src/design-system.css`

## Equivalence Anchor

- Expanding all local `frontend/src/design-system.css` imports must reproduce `HEAD:frontend/src/design-system.css` after newline normalization.

## Further-Split Decision

No deeper split is useful inside `focus_selection_scrollbars` now. It owns only global focus/selection affordances and scrollbar pseudo-element styling.

## Verification

- CSS expanded equivalence against `HEAD:frontend/src/design-system.css`: passed.
- From `frontend/`, `npm.cmd run build`: passed.
- From repo root, `git diff --check`: passed.
- From repo root, frontend recursive state JSON parse: passed.
- From repo root, `tools/check-full-feature-tree.ps1`: passed.
- From repo root, `tools/check-matrix-governance.ps1`: passed.
