# FE-0118 - Frontend Backtest Views Compare Cards Summary Closeout

Status: closed.

## Scope

- Active child parent: `frontend.backtest_views.compare_page_analysis`
- Closed subchild leaf: `frontend.backtest_views.compare_page_analysis.compare_cards_and_summary`
- Public surface: `frontend/src/pages/backtestViews/comparePageAnalysis/index.js`

## Extraction

- Added `frontend/src/pages/backtestViews/comparePageAnalysis/BacktestCompareCardsAndSummary.jsx`.
- Added `frontend/src/pages/backtestViews/comparePageAnalysis/BacktestCompareCardsAndSummary.test.jsx`.
- Updated `frontend/src/pages/backtestViews/comparePageAnalysis/index.js`.
- Updated `frontend/src/pages/BacktestComparePage.jsx` to render extracted compare cards and summary sidebar components.

## Whitebox Contract

- Inputs:
  - Loaded compare `details`.
  - Compare summary deltas from the parent model leaf.
  - Resolved strategy id.
  - `onOpenDetail(backtestId)` callback owned by the route shell.
- Outputs:
  - Compare cards section with per-run metrics and detail actions.
  - Compare summary sidebar with strategy id, deltas, and compared backtest ids.
  - Pure card and summary models for testable data projection.
- Parent communication:
  - `BacktestComparePage.jsx` keeps route navigation ownership.
  - The child leaf receives a callback instead of importing router navigation directly.
  - Card metrics, dataset labels, execution assumption labels, and summary sidebar rows are owned inside the child leaf.

## Preserved Behavior

- The compare card grid retains existing `data-testid` contracts.
- Detail buttons still navigate through the route shell to the existing detail path.
- Summary sidebar text still includes the resolved strategy id and compared backtest id pair.
- The chart section remains between the cards section and the summary sidebar in the grid.

## Further Split Decision

- `frontend.backtest_views.compare_page_analysis.compare_cards_and_summary` is closed.
- `frontend.backtest_views.compare_page_analysis` has no remaining queued subchildren.
- The next recursive step is a parent closeout for `frontend.backtest_views.compare_page_analysis`.

## Verification

- Targeted Vitest:
  - `npm.cmd test -- --run src/pages/backtestViews/comparePageAnalysis/BacktestCompareCardsAndSummary.test.jsx src/pages/backtestViews/comparePageAnalysis/backtestComparePageModel.test.js src/pages/BacktestComparePage.test.jsx`
  - Result: passed, 3 files / 8 tests.
- Full feature tree gate: passed.
- Recursive state JSON parse: passed.
- Frontend build: passed.
- `git diff --check`: passed.
