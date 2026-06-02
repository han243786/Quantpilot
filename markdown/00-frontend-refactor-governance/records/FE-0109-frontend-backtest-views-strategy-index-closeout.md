# FE-0109 Frontend Backtest Views Strategy Index Closeout

Status: closed.

## Child Node

`frontend.backtest_views.strategy_backtests_index`

## Extraction

The strategy-scoped backtest index route now has a dedicated whitebox model surface:

- `frontend/src/pages/backtestViews/strategyBacktestsIndex/index.js`
- `frontend/src/pages/backtestViews/strategyBacktestsIndex/strategyBacktestsIndexModel.js`
- `frontend/src/pages/backtestViews/strategyBacktestsIndex/strategyBacktestsIndexModel.test.js`

`frontend/src/pages/StrategyBacktestsPage.jsx` remains the route shell and still owns hooks, route actions, notice state, and the `StrategyBacktestsPanel` insertion point.

## Whitebox Contract

### Public Inputs

- Route `strategyId`.
- Strategy graph metadata and compile summary from `useStrategyWorkspaceSharedModel`.
- Backtest selector projection from `useStrategyResearchSelectors`, including filtered backtests and compare selection.

### Public Outputs

- Strategy display name for the route hero and sidebar.
- Hero summary metric items.
- Compare button enabled/disabled state.
- Graph-loading state for the route banner.
- Dataset label text for the strategy context card.

## Preserved Behavior

- The page still loads strategy graph context when the current graph id does not match the route id.
- The page still routes back to strategy list, workspace, compare, and detail paths through the existing router helpers.
- `StrategyBacktestsPanel` remains outside this child and receives the same selector/action props.
- Runtime panel/history internals remain untouched.

## Further-Split Decision

No deeper split is useful for this leaf now. The model is a pure projection with one small route shell consumer. Further work should move to `detail_page_analysis`; revisit this leaf only if the index page grows independent filtering, sorting, or route-state persistence.

## Verification

- From `frontend/`, strategy backtests index target set: passed, 2 files / 3 tests.
- From repo root, `git diff --check`: passed.
- From repo root, frontend recursive state JSON parse: passed.
- From repo root, `tools/check-full-feature-tree.ps1`: passed.
- From `frontend/`, `npm.cmd run build`: passed.

## Next Child

`frontend.backtest_views.detail_page_analysis`
