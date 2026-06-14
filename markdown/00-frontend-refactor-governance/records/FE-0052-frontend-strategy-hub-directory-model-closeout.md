# FE-0052 Frontend Strategy Hub Directory Model Closeout

Status: closed.

## Leaf Node

`frontend.strategy_hub.directory_model`

## Code Changes

- Added `frontend/src/hooks/strategyDirectoryModelProjection.js`.
- Added `frontend/src/hooks/strategyDirectoryModelProjection.test.js`.
- Updated `frontend/src/hooks/useStrategyDirectoryModel.js` so the hook imports pure directory projection helpers instead of defining them inline.

## Preserved Behavior

- `useStrategyDirectoryModel` still owns graph-store subscriptions, refresh triggers, local selected-strategy state, template application, workspace navigation, blank workspace creation, compare selection actions, file reveal, and delete confirmation.
- `StrategyHubPage.jsx` still receives the same model shape from `useStrategyDirectoryModel`.
- Hub page rendering and action flow remain covered by `StrategyHubPage.test.jsx`.

## Projection Corrections

- Fixed current graph merge inside directory projection: when graph index already contains the current graph, the current graph now keeps `isCurrent`, runnable/compilable state, issue count, protocol, compile id, and config hash.
- Fixed dataset label projection so duplicate labels are removed before final display truncation.

## Public Inputs

- Graph metadata and validation/compile summary.
- Graph index entries.
- Runtime run history and backtest history.
- Backtest compare selection ids.

## Public Outputs

- Available graph id set.
- Visible run and backtest records.
- Strategy directory entries with health, activity label, recent runs, recent backtests, dataset labels, and latest return ratio.
- Recent activity timeline.
- Hub summary metrics.

## Verification

- From `frontend/`, `npm.cmd test -- --run src/hooks/strategyDirectoryModelProjection.test.js src/pages/StrategyHubPage.test.jsx`: passed, 2 files and 8 tests.
- From repo root, `git diff --check`: passed.
- From repo root, frontend recursive state JSON parse: passed.
- From repo root, `tools/check-full-feature-tree.ps1`: passed.
- From `frontend/`, `npm.cmd run build`: passed.

## Further-Split Decision

`frontend.strategy_hub.directory_model` does not need a deeper split yet. Pure projection is now isolated and tested, while store subscriptions and user actions stay in the hook until a later store-parent extraction or a concrete hub action leaf needs deeper separation.

## Next Leaf

`frontend.strategy_hub.hero_summary`
