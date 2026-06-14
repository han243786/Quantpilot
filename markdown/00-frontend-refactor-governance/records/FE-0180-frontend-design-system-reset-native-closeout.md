# FE-0180 - Frontend Design System Reset And Native Controls Closeout

Status: closed.

## Leaf Node

`frontend.design_system_styles.design_tokens_and_native_controls.reset_and_native_controls`

## Code Changes

- Added `frontend/src/design-system/reset-and-native-controls.css`.
- Converted the top of `frontend/src/design-system.css` into an ordered CSS aggregator import for this reset/native partial.

## Preserved Behavior

- The extracted partial preserves the original design-system header, global reset, root/body sizing, body typography, and native `input`/`select`/`textarea`/`button` defaults.
- The `@import` remains at the top of `frontend/src/design-system.css`, so cascade order is unchanged after import expansion.
- No token, theme, shell, page, or shared primitive selectors were edited.

## Public Inputs

- `frontend/src/styleEntrypoint.js` importing `frontend/src/design-system.css`.
- Browser and Vite CSS `@import` handling.

## Public Outputs

- `frontend/src/design-system.css`
- `frontend/src/design-system/reset-and-native-controls.css`

## Equivalence Anchor

- Expanding `frontend/src/design-system.css` local imports must reproduce `HEAD:frontend/src/design-system.css` after newline normalization.

## Further-Split Decision

No deeper split is useful inside `reset_and_native_controls` now. It contains one reset policy and one native-control default policy; separating those would create tiny fragments without improving ownership clarity.

## Verification

- CSS expanded equivalence against `HEAD:frontend/src/design-system.css`: passed.
- From `frontend/`, `npm.cmd run build`: passed.
- From repo root, `git diff --check`: passed.
- From repo root, frontend recursive state JSON parse: passed.
- From repo root, `tools/check-full-feature-tree.ps1`: passed.
- From repo root, `tools/check-matrix-governance.ps1`: passed.
