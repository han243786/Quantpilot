# FE-0108 Frontend Backtest Views Analysis Layout Shared Closeout

Status: closed.

## Child Node

`frontend.backtest_views.analysis_layout_shared`

## Extraction

The shared backtest analysis presentation layer now lives behind a dedicated whitebox entry:

- `frontend/src/pages/backtestViews/shared/index.js`

The extracted files are:

- `frontend/src/pages/backtestViews/shared/BacktestAnalysisLayout.jsx`
- `frontend/src/pages/backtestViews/shared/BacktestAnalysisLayout.test.jsx`
- `frontend/src/pages/backtestViews/shared/backtestAnalysisShared.jsx`
- `frontend/src/pages/backtestViews/shared/backtestAnalysisShared.test.jsx`
- `frontend/src/pages/backtestViews/shared/DrawdownChart.jsx`
- `frontend/src/pages/backtestViews/shared/MonthlyReturnsHeatmap.jsx`

## Compatibility Facades

The previous parent-level import paths remain active as compatibility facades:

- `frontend/src/pages/BacktestAnalysisLayout.jsx`
- `frontend/src/pages/backtestAnalysisShared.jsx`
- `frontend/src/components/DrawdownChart.jsx`
- `frontend/src/components/MonthlyReturnsHeatmap.jsx`

This keeps external consumers such as strategy workspace, strategy hub, and runtime panels parent-mediated instead of forcing them to import the child module directly.

## Whitebox Contract

### Public Inputs

- Route item descriptors for strategy/backtest navigation breadcrumbs.
- Analysis hero text, metadata, action nodes, status variants, and summary metric items.
- Backtest detail/compare summary payloads, execution assumption payloads, datasets, risk metrics, trade metrics, drawdown metrics, and benchmark metrics.
- Equity curve points and monthly period-return rows.

### Public Outputs

- `StrategyRouteBar`
- `AnalysisHero`
- `AnalysisSection`
- `AnalysisStatusBanner`
- `MetricPair`
- Backtest formatting and metric projection helpers.
- `DrawdownChart`
- `MonthlyReturnsHeatmap`

## Preserved Behavior

- Backtest detail, compare, and strategy-scoped backtest index pages now import the shared layer from `./backtestViews/shared`.
- External parent consumers can still import `StrategyRouteBar`, `maxDrawdownFromSummary`, `DrawdownChart`, and `MonthlyReturnsHeatmap` through their old facade paths.
- Shared backtest CSS remains owned by the parent page stylesheet path and is imported by the extracted layout module.
- No child-to-child shortcut was introduced; external modules still communicate through the parent-level facades.

## Further-Split Decision

No deeper split is useful for this leaf now. Layout primitives, metric projection helpers, and two focused chart widgets are already separated by file and covered by the shared entry tests plus page anchor tests. Revisit a chart-specific child only if drawdown/monthly widgets gain independent state, data loading, or cross-parent reuse.

## Verification

- From `frontend/`, backtest shared analysis target set: passed, 6 files / 12 tests.
- From repo root, `git diff --check`: passed.
- From repo root, frontend recursive state JSON parse: passed.
- From repo root, `tools/check-full-feature-tree.ps1`: passed.
- From `frontend/`, `npm.cmd run build`: passed.

## Next Child

`frontend.backtest_views.strategy_backtests_index`
