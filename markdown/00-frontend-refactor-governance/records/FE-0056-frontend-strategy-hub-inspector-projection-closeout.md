# FE-0056 Frontend Strategy Hub Inspector Projection Closeout

Status: closed.

## Leaf Node

`frontend.strategy_hub.inspector_projection`

## Code Changes

- Updated `frontend/src/hooks/useStrategyHubInspectorData.js` so the hook also projects inspector overview data through `projectStrategyHubInspectorOverview`.
- Updated `frontend/src/pages/StrategyHubInspectorSection.jsx` to pass the projected overview into the overview section.
- Updated `frontend/src/pages/StrategyHubInspectorOverviewSection.jsx` so it renders the supplied overview instead of recalculating projection data locally.

## Preserved Behavior

- Empty inspector text, route bar, health pill, summary metrics, metric rows, and next-move recommendation stay derived from the same projection helper.
- Recent backtests, recent runs, and compare queue projections remain memoized through `useStrategyHubInspectorData`.
- Inspector action groups and dispatch stay under the existing action leaf boundary and are still tested separately.

## Public Inputs

- Selected strategy record.
- Backtest compare selection ids.
- Strategy directory model action methods for inspector controls.

## Public Outputs

- Inspector overview view model.
- Recent backtest view models.
- Recent run view models.
- Compare queue view model.
- Rendered inspector overview shell consuming the projected overview.

## Verification

- From `frontend/`, `npm.cmd test -- --run src/utils/strategyHubInspectorProjection.test.js src/utils/strategyHubInspectorActions.test.js src/pages/StrategyHubPage.test.jsx`: passed, 3 files and 10 tests.
- From repo root, `git diff --check`: passed.
- From repo root, frontend recursive state JSON parse: passed.
- From repo root, `tools/check-full-feature-tree.ps1`: passed.
- From `frontend/`, `npm.cmd run build`: passed.

## Further-Split Decision

`frontend.strategy_hub.inspector_projection` does not need a deeper split now. The overview, recent activity, and compare queue projections are pure and tested, while inspector action dispatch remains a separate existing action boundary.

## Next Leaf

`frontend.strategy_hub.recent_activity_compare`
