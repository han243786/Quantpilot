# FE-0117 - Frontend Backtest Views Compare Equity Overlay Chart Closeout

Status: closed.

## Scope

- Active child parent: `frontend.backtest_views.compare_page_analysis`
- Closed subchild leaf: `frontend.backtest_views.compare_page_analysis.equity_overlay_chart`
- Public surface: `frontend/src/pages/backtestViews/comparePageAnalysis/index.js`

## Extraction

- Added `frontend/src/pages/backtestViews/comparePageAnalysis/BacktestCompareEquityOverlayChart.jsx`.
- Added `frontend/src/pages/backtestViews/comparePageAnalysis/BacktestCompareEquityOverlayChart.test.jsx`.
- Updated `frontend/src/pages/backtestViews/comparePageAnalysis/index.js`.
- Updated `frontend/src/pages/BacktestComparePage.jsx` to call the extracted chart component.

## Whitebox Contract

- Inputs:
  - Two backtest detail records from the compare page route shell.
  - `backtest_artifacts.equity_curve` as either a raw point array or an artifact object with `points`.
  - Optional `backtest_artifacts.benchmark_equity_curve` in the same shapes.
- Outputs:
  - Overlay chart rows with `cycle`, `a`, `b`, and `benchmark`.
  - Stable short labels for the left and right runs.
  - Benchmark visibility flag.
  - Rendered compare equity overlay chart or no-data fallback.
- Parent communication:
  - `BacktestComparePage.jsx` passes `state.details` only.
  - The chart leaf owns curve normalization, merge shape, labels, benchmark detection, and Recharts wiring.

## Preserved Behavior

- The compare route shell still controls loading, error, cards, summary sidebar, and navigation.
- The chart stays under the existing "equity compare" analysis section.
- The no-data fallback remains available when no equity points can be resolved.

## Boundary Finding

- The extracted whitebox model now accepts both raw arrays and artifact `points` wrappers.
- This keeps artifact shape handling inside the chart leaf and prevents route-shell coupling to backend artifact packaging.

## Further Split Decision

- `frontend.backtest_views.compare_page_analysis.equity_overlay_chart` is closed.
- The active compare child parent still has one queued subchild:
  - `frontend.backtest_views.compare_page_analysis.compare_cards_and_summary`
- The compare parent is not closed until that queue is finished and the parent closeout confirms no further useful split.

## Verification

- Targeted Vitest:
  - `npm.cmd test -- --run src/pages/backtestViews/comparePageAnalysis/BacktestCompareEquityOverlayChart.test.jsx src/pages/BacktestComparePage.test.jsx`
  - Result: passed, 2 files / 5 tests.
- Full feature tree gate: passed.
- Recursive state JSON parse: passed.
- Frontend build: passed.
- `git diff --check`: passed.
