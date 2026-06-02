# FE-0111 Frontend Backtest Views Detail Summary Context Closeout

Status: closed.

## Parent Node

`frontend.backtest_views.detail_page_analysis`

## Closed Subchild

`frontend.backtest_views.detail_page_analysis.summary_and_context`

## Extraction

Backtest detail hero summary projection now lives in the detail analysis model surface:

- `frontend/src/pages/backtestViews/detailPageAnalysis/backtestDetailSummaryModel.js`
- `frontend/src/pages/backtestViews/detailPageAnalysis/backtestDetailSummaryModel.test.js`

`frontend/src/pages/BacktestDetailPage.jsx` now asks the model for `summaryItems` and keeps rendering the same `AnalysisHero` and route/context sections.

## Whitebox Contract

### Public Inputs

- Translation function `t`.
- Metrics summary, manifest, selected summary, final account metrics, trades, and expanded/collapsed state.

### Public Outputs

- Visible summary cards for return, annualized return, Sharpe ratio, max drawdown, and profit factor.
- Expanded summary cards for Sortino, Calmar, annualized volatility, drawdown duration, Alpha/Beta, win rate, trade count, protocol, and final equity.

## Preserved Behavior

- The detail route still owns `summaryExpanded` state and the expand/collapse button.
- Summary card labels, formatted values, tooltip text, and protocol/final-equity fallbacks stay equivalent.
- Route navigation, artifact sections, timeline/report sections, replay previews, explanations, and event stream embedding remain untouched.

## Further-Split Decision

No deeper split is useful inside `summary_and_context` now. The extracted model is a compact pure projection with two direct tests and one route consumer. Continue the active detail child parent through `core_artifact_sections`.

## Verification

- From `frontend/`, detail summary target set: passed, 3 files / 6 tests.
- From repo root, `git diff --check`: passed.
- From repo root, frontend recursive state JSON parse: passed.
- From repo root, `tools/check-full-feature-tree.ps1`: passed.
- From `frontend/`, `npm.cmd run build`: passed.

## Next Child

`frontend.backtest_views.detail_page_analysis.core_artifact_sections`
