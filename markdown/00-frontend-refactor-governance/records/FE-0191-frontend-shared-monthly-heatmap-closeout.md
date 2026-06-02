# FE-0191 - Frontend Shared Monthly Heatmap Closeout

Status: closed.

## Leaf Node

`frontend.design_system_styles.shared_component_primitives.monthly_heatmap_styles`

## Code Changes

- Added `frontend/src/shared/monthly-heatmap.css`.
- Moved monthly heatmap table, header, label, cell, and yearly-return selectors from `frontend/src/shared.css` into the new partial.
- Kept `frontend/src/shared.css` as the ordered shared-style import aggregator.

## Preserved Behavior

- Monthly heatmap selector bodies and order are unchanged after import expansion.
- `frontend/src/pages/backtestViews/shared/MonthlyReturnsHeatmap.jsx` keeps the same class contract.

## Public Inputs

- Token values from `frontend/src/design-system/theme-tokens.css`.
- `monthly-heatmap`, `heatmap-*`, and yearly return class usage from the backtest monthly returns view.

## Public Outputs

- `frontend/src/shared/monthly-heatmap.css`
- `frontend/src/shared.css`

## Equivalence Anchor

- Expanding all local `frontend/src/shared.css` imports must reproduce `HEAD:frontend/src/shared.css` after newline normalization.

## Further-Split Decision

No deeper split is useful inside `monthly_heatmap_styles` now. The leaf is a compact table styling contract with one direct feature consumer.

## Verification

- CSS expanded equivalence against `HEAD:frontend/src/shared.css`: passed.
- From `frontend/`, `npm.cmd run build`: passed.
- From repo root, `git diff --check`: passed.
- From repo root, frontend recursive state JSON parse: passed.
- From repo root, `tools/check-full-feature-tree.ps1`: passed.
- From repo root, `tools/check-matrix-governance.ps1`: passed.
