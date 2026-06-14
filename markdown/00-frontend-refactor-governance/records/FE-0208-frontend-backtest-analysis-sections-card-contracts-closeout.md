# FE-0208 - Frontend Backtest Analysis Sections Card Contracts Closeout

Status: closed.

## Leaf Node

`frontend.design_system_styles.page_style_contracts.backtest_analysis_page_styles.analysis_sections_and_card_contracts`

## Code Changes

- Added `frontend/src/pages/backtest-analysis/analysis-sections-card-contracts.css`.
- Moved analysis section shell, section header/body typography, card grid, status banner, follow-up section, toolbar alignment, and shared backtest card color overrides out of `frontend/src/pages/backtest-analysis.css`.
- Kept `frontend/src/pages/backtest-analysis.css` as the ordered page-style import aggregator plus remaining responsive and compare motion rules.

## Preserved Behavior

- Extracted selector bodies and cascade order are unchanged after import expansion.
- Backtest detail and compare pages continue to import only `frontend/src/pages/backtest-analysis.css`.

## Public Inputs

- Backtest analysis section DOM classes.
- Shared backtest card, open-order, account metric, key-value, and toolbar classes rendered inside analysis pages.

## Public Outputs

- `frontend/src/pages/backtest-analysis/analysis-sections-card-contracts.css`
- `frontend/src/pages/backtest-analysis.css`

## Further-Split Decision

No deeper split is useful inside `analysis_sections_and_card_contracts` now. The leaf owns one compact, ordered contract for section composition and shared card presentation before responsive overrides take over.

## Verification

- CSS expanded equivalence against `HEAD:frontend/src/pages/backtest-analysis.css`: passed.
- From `frontend/`, `npm.cmd run build`: passed.
- From repo root, `git diff --check`: passed.
- From repo root, frontend recursive state JSON parse: passed.
- From repo root, `tools/check-full-feature-tree.ps1`: passed.
- From repo root, `tools/check-matrix-governance.ps1`: passed.
