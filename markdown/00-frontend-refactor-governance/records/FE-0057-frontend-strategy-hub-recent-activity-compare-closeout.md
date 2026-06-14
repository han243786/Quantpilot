# FE-0057 Frontend Strategy Hub Recent Activity Compare Closeout

Status: closed.

## Leaf Node

`frontend.strategy_hub.recent_activity_compare`

## Code Changes

- No runtime code change.
- Confirmed `frontend/src/utils/strategyHubRecentRunsView.js` owns recent-run section projection.
- Confirmed `frontend/src/utils/strategyHubRecentBacktestsActions.js` owns recent-backtest action grouping and dispatch.
- Confirmed `frontend/src/utils/strategyHubCompareQueueActions.js` owns compare-queue projection and dispatch.
- Kept the existing utility tests and `StrategyHubPage.test.jsx` as the leaf equivalence baseline.

## Preserved Behavior

- Recent run items still receive derived display tone before rendering.
- Recent backtest detail and compare-toggle controls keep their current labels, aria labels, selection state, and route behavior.
- Compare queue chips, clear action, and compare navigation keep the same availability rules.
- Strategy hub page still renders the inspector activity and compare flow from the selected strategy model.

## Public Inputs

- Recent run view models from inspector projection.
- Recent backtest view models from inspector projection.
- Compare queue view model from inspector projection.
- Selected graph id.
- Compare toggle and clear-selection callbacks.

## Public Outputs

- Recent run section view model.
- Recent backtest action group and dispatcher side effects.
- Compare queue chips, action states, and dispatcher side effects.
- Rendered recent activity and compare sections.

## Verification

- From `frontend/`, `npm.cmd test -- --run src/utils/strategyHubRecentRunsView.test.js src/utils/strategyHubRecentBacktestsActions.test.js src/utils/strategyHubCompareQueueActions.test.js src/pages/StrategyHubPage.test.jsx`: passed, 4 files and 9 tests.
- From repo root, `git diff --check`: passed.
- From repo root, frontend recursive state JSON parse: passed.
- From repo root, `tools/check-full-feature-tree.ps1`: passed.
- From `frontend/`, `npm.cmd run build`: passed.

## Further-Split Decision

`frontend.strategy_hub.recent_activity_compare` does not need a deeper split now. Recent runs, recent backtest actions, and compare queue behavior already have separate pure boundaries and targeted tests; another split would mostly duplicate the existing component/util separation.

## Next Leaf

`frontend.strategy_hub.template_library`
