# FE-0120 - Frontend Backtest Views Parent Closeout

Status: closed.

## Parent Node

`frontend.backtest_views`

## Closed Children

- `frontend.backtest_views.analysis_layout_shared`
- `frontend.backtest_views.strategy_backtests_index`
- `frontend.backtest_views.detail_page_analysis`
- `frontend.backtest_views.compare_page_analysis`

## Final Parent Boundary

`frontend.backtest_views` now owns strategy-scoped backtest list views, persisted backtest detail analysis, backtest comparison, shared analysis layout primitives, result metric projections, drawdown and monthly-return visualizations, detail page artifact sections, compare page chart/cards/sidebar leaves, and compatibility facades for legacy imports.

Strategy workspace, strategy hub, runtime panels, global store migration, router contracts, and backend API transport remain outside this parent.

## Whitebox Contract

### Public Inputs

- Route-owned strategy ids and selected backtest ids.
- Backend backtest history/detail responses loaded through existing API facades.
- Shared router paths and navigation callbacks supplied by route shells.
- Runtime artifact shapes, including wrapped artifact `points` lists for chart inputs.

### Public Outputs

- `StrategyBacktestsPage`, `BacktestDetailPage`, and `BacktestComparePage` route shells.
- Shared analysis layout primitives and compatibility facades.
- Backtest list projections and detail/compare page model helpers.
- Detail page core artifact, evidence/report, replay/output/explanation sections.
- Compare page model helpers, equity overlay chart, cards section, and summary sidebar.

## Preserved Behavior

- Strategy backtest list, detail page, and compare page route tests remain covered.
- Existing `data-testid` contracts for compare cards, summary, hero, and detail navigation remain stable.
- Detail page artifact sections keep their public component entry points under `detailPageAnalysis`.
- Compare page leaves keep parent-to-child communication through props and callbacks.
- No child-to-child shortcut was introduced during this parent.

## Further-Split Decision

No further split is useful inside `frontend.backtest_views` now. All planned child leaves and child parents are closed. Remaining route-shell responsibilities are stable orchestration boundaries or compatibility facades. Additional splitting should wait for a concrete backtest UI feature, a backend artifact contract change, or a new design-system/style parent decision.

## Verification

- Backtest views parent anchor test set: passed, 14 files / 32 tests.
- From repo root, `git diff --check`: passed.
- From repo root, frontend recursive state JSON parse: passed.
- From repo root, `tools/check-full-feature-tree.ps1`: passed.
- From repo root, `tools/check-matrix-governance.ps1`: passed.
- From `frontend/`, `npm.cmd run build`: passed.

## Next Parent Candidate

`frontend.store`
