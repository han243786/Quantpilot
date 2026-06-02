# FE-0054 Frontend Strategy Hub Roster Projection Closeout

Status: closed.

## Leaf Node

`frontend.strategy_hub.roster_projection`

## Code Changes

- No runtime code change.
- Confirmed the existing `frontend/src/hooks/useStrategyHubRosterData.js` bridge already delegates roster, toolbar, and activity view-model projection to `frontend/src/utils/strategyHubRosterProjection.js`.
- Kept `frontend/src/utils/strategyHubRosterProjection.test.js` as the leaf equivalence baseline.

## Preserved Behavior

- Strategy roster filtering output, selected strategy counts, workspace availability, activity bucketing, and row display labels keep the same public shape.
- `StrategyHubPage.test.jsx` still covers rendered hub behavior around roster availability and strategy hub actions.
- Activity cards remain split from roster projection and stay queued under later activity leaves.

## Public Inputs

- `model.activityTimeline`.
- `model.filteredStrategies`.
- `model.selectedStrategyCount`.
- `model.selectedForWorkspace`.
- `model.selectedStrategyIds`.
- `model.selectedStrategy`.

## Public Outputs

- Backtest and run activity item lists with `createdAtLabel`.
- Roster toolbar labels and action availability flags.
- Roster row view models with health, activity, count, return, selected, active, and file-path flags.

## Verification

- From `frontend/`, `npm.cmd test -- --run src/utils/strategyHubRosterProjection.test.js src/pages/StrategyHubPage.test.jsx`: passed, 2 files and 7 tests.
- From repo root, `git diff --check`: passed.
- From repo root, frontend recursive state JSON parse: passed.
- From repo root, `tools/check-full-feature-tree.ps1`: passed.
- From `frontend/`, `npm.cmd run build`: passed.

## Further-Split Decision

`frontend.strategy_hub.roster_projection` does not need a deeper split now. The leaf has a pure projection utility, a memo-only hook boundary, and targeted tests. Deeper splits would duplicate the existing roster/action/activity separation without reducing coupling.

## Next Leaf

`frontend.strategy_hub.roster_row_actions`
