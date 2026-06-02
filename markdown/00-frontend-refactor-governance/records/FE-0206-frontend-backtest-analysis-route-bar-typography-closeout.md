# FE-0206 - Frontend Backtest Analysis Route Bar Typography Closeout

Status: closed.

## Leaf Node

`frontend.design_system_styles.page_style_contracts.backtest_analysis_page_styles.route_bar_and_typography`

## Code Changes

- Added `frontend/src/pages/backtest-analysis/route-bar-and-typography.css`.
- Moved analysis hero main layout, route bar, breadcrumb segment, title/kicker/summary typography, and hero metadata rules out of `frontend/src/pages/backtest-analysis.css`.
- Kept `frontend/src/pages/backtest-analysis.css` as the ordered page-style import aggregator plus remaining summary, section, and responsive rules.

## Preserved Behavior

- Extracted selector bodies and cascade order are unchanged after import expansion.
- Backtest detail and compare pages continue to import only `frontend/src/pages/backtest-analysis.css`.

## Public Inputs

- Backtest analysis route breadcrumb DOM classes.
- Shared analysis page typography and metadata classes.

## Public Outputs

- `frontend/src/pages/backtest-analysis/route-bar-and-typography.css`
- `frontend/src/pages/backtest-analysis.css`

## Further-Split Decision

No deeper split is useful inside `route_bar_and_typography` now. The leaf owns a compact route/typography contract that is ordered between shell chrome and summary cards.

## Verification

- CSS expanded equivalence against `HEAD:frontend/src/pages/backtest-analysis.css`: passed.
- From `frontend/`, `npm.cmd run build`: passed.
- From repo root, `git diff --check`: passed.
- From repo root, frontend recursive state JSON parse: passed.
- From repo root, `tools/check-full-feature-tree.ps1`: passed.
- From repo root, `tools/check-matrix-governance.ps1`: passed.
