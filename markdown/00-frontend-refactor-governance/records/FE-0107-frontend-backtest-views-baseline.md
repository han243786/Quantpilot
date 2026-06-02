# FE-0107 Frontend Backtest Views Baseline

Status: closed.

## Parent Node

`frontend.backtest_views`

## Scope

This parent owns the strategy backtest index route, persisted backtest detail analysis route, persisted backtest compare route, shared analysis layout primitives, summary metric helpers, drawdown visualization, monthly return heatmap visualization, and the local backtest analysis stylesheet.

Runtime history panels, event stream runtime embedding, graph store persistence, API transport, routing registration, strategy workspace/hub launch points, and global design-system ownership remain outside this parent.

## Owned And Split-Target Files

- `frontend/src/pages/StrategyBacktestsPage.jsx`
- `frontend/src/pages/StrategyBacktestsPage.test.jsx`
- `frontend/src/pages/BacktestDetailPage.jsx`
- `frontend/src/pages/BacktestDetailPage.test.jsx`
- `frontend/src/pages/BacktestComparePage.jsx`
- `frontend/src/pages/BacktestComparePage.test.jsx`
- `frontend/src/pages/BacktestAnalysisLayout.jsx`
- `frontend/src/pages/backtestAnalysisShared.jsx`
- `frontend/src/pages/backtestAnalysisShared.test.jsx`
- `frontend/src/pages/backtest-analysis.css`
- `frontend/src/components/DrawdownChart.jsx`
- `frontend/src/components/MonthlyReturnsHeatmap.jsx`

## Important External Consumers

- `frontend/src/router.js`
- `frontend/src/app/AppRouteHost.jsx`
- `frontend/src/pages/StrategyWorkspacePage.jsx`
- `frontend/src/pages/StrategyWorkspaceOverviewTab.jsx`
- `frontend/src/pages/StrategyWorkspaceResearchTab.jsx`
- `frontend/src/pages/StrategyWorkspaceVersionHistoryCard.jsx`
- `frontend/src/pages/StrategyHubPage.jsx`
- `frontend/src/pages/StrategyHubRecentBacktestsSection.jsx`
- `frontend/src/components/BacktestHistorySection.jsx`
- `frontend/src/components/StrategyBacktestsPanel.jsx`
- `frontend/src/components/EventStreamPanel.jsx`
- `frontend/src/store/graphStore.js`

## Whitebox Contract

### Public Inputs

- Route params for strategy id, backtest id, and comparison ids.
- Graph store selectors for strategies, graphs, saved backtests, runtime artifacts, and backtest refresh actions.
- Backtest history and artifact records supplied by the runtime/store boundary.
- Analysis metric payloads, equity curves, drawdown series, monthly returns, orders, trades, diagnostics, and metadata.
- User actions for refreshing, opening detail views, comparing runs, revealing artifacts, and returning to strategy context.

### Public Outputs

- Strategy-scoped backtest index page with filtered persisted backtest records and compare/detail navigation.
- Persisted backtest detail page with shared analysis shell, summary cards, charts, tables, diagnostics, and artifact actions.
- Backtest comparison page with shared comparison layout, metric deltas, selected records, and strategy context navigation.
- Shared presentation helpers for metric formatting, status labels, section layout, drawdown charts, and monthly heatmaps.

## Initial Child Queue

- `frontend.backtest_views.analysis_layout_shared`
- `frontend.backtest_views.strategy_backtests_index`
- `frontend.backtest_views.detail_page_analysis`
- `frontend.backtest_views.compare_page_analysis`

## Split Decision

The parent is worth splitting now because it mixes route orchestration, shared analysis presentation, strategy-scoped index behavior, detail analysis behavior, comparison behavior, and visualization helpers. The first recursion should extract the shared analysis/layout helpers so later route leaves can depend on a stable parent-mediated presentation surface.

## Verification

- From `frontend/`, backtest views anchor test set: passed, 4 files / 7 tests.
- From repo root, `git diff --check`: passed.
- From repo root, frontend recursive state JSON parse: passed.
- From repo root, `tools/check-full-feature-tree.ps1`: passed.
- From `frontend/`, `npm.cmd run build`: passed.

## Next Child

`frontend.backtest_views.analysis_layout_shared`
