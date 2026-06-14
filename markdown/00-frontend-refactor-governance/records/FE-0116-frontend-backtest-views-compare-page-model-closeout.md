# FE-0116 - Frontend Backtest Views Compare Page Model Closeout

Status: closed.

## Scope

- Parent node: `frontend.backtest_views`
- Active child parent: `frontend.backtest_views.compare_page_analysis`
- Closed subchild leaf: `frontend.backtest_views.compare_page_analysis.artifact_model`
- Public surface: `frontend/src/pages/backtestViews/comparePageAnalysis/index.js`

## Extraction

- Added `frontend/src/pages/backtestViews/comparePageAnalysis/backtestComparePageModel.js`.
- Added `frontend/src/pages/backtestViews/comparePageAnalysis/backtestComparePageModel.test.js`.
- Added `frontend/src/pages/backtestViews/comparePageAnalysis/index.js`.
- Updated `frontend/src/pages/BacktestComparePage.jsx` to consume the model for normalized compare ids, summary deltas, strategy identity, and compare metadata.

## Whitebox Contract

- Inputs:
  - Compare route ids from `BacktestComparePage`.
  - Loaded backtest details and summary metrics.
  - Optional strategy id supplied by the route.
- Outputs:
  - Deduplicated two-item compare id list.
  - Formatted summary item descriptors for the analysis hero.
  - Resolved strategy id when both details belong to the same graph.
  - Stable `A vs B` compare metadata text.
- Parent communication:
  - `BacktestComparePage.jsx` stays the route shell and API orchestration owner.
  - `comparePageAnalysis` exposes pure page-analysis helpers through its index file.

## Preserved Behavior

- Compare pages still fetch up to two unique backtest ids.
- Compare summary deltas preserve total-return, drawdown, and trade-count semantics.
- Strategy breadcrumb ownership still prefers the route strategy id and falls back to matching detail graph ids.
- Existing compare page tests remain green.

## Further Split Decision

- `frontend.backtest_views.compare_page_analysis` is worth continuing as an active child parent.
- Remaining UI responsibilities are still mixed in the route shell:
  - `frontend.backtest_views.compare_page_analysis.equity_overlay_chart`
  - `frontend.backtest_views.compare_page_analysis.compare_cards_and_summary`
- The first model leaf is closed; the compare parent is not closed yet.

## Verification

- Targeted Vitest:
  - `npm.cmd test -- --run src/pages/backtestViews/comparePageAnalysis/backtestComparePageModel.test.js src/pages/BacktestComparePage.test.jsx`
  - Result: passed, 2 files / 5 tests.
- Full feature tree gate: passed.
- Recursive state JSON parse: passed.
- Frontend build: passed.
- `git diff --check`: passed.
