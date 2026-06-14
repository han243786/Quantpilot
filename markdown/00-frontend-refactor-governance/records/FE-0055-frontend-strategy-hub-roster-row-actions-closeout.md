# FE-0055 Frontend Strategy Hub Roster Row Actions Closeout

Status: closed.

## Leaf Node

`frontend.strategy_hub.roster_row_actions`

## Code Changes

- No runtime code change.
- Confirmed `frontend/src/utils/strategyHubRosterRowActions.js` owns semantic action grouping and action dispatch.
- Confirmed `frontend/src/pages/StrategyHubRosterRowActions.jsx` only owns row-level pending state, more-menu state, and action error feedback.
- Kept `frontend/src/utils/strategyHubRosterRowActions.test.js` and `frontend/src/pages/StrategyHubRosterTableSection.test.jsx` as the leaf equivalence baseline.

## Preserved Behavior

- Row actions still expose workspace, backtest, reveal-file, and delete actions.
- Reveal-file disabled state remains derived from the row file-path flag.
- Workspace and backtest actions still route through the router path helpers.
- Reveal-file and delete actions still call the strategy directory model methods.
- Failed async actions still render row-local alert feedback.

## Public Inputs

- Roster row identity, display name, and file-path availability.
- Strategy directory model methods: `revealGraphFile` and `deleteStrategy`.
- Router path helpers for workspace and backtest destinations.

## Public Outputs

- Grouped row action view models.
- Primary row action button.
- Secondary row action menu.
- Action-specific disabled and pending states.
- Row-local failure message.

## Verification

- From `frontend/`, `npm.cmd test -- --run src/utils/strategyHubRosterRowActions.test.js src/pages/StrategyHubRosterTableSection.test.jsx src/pages/StrategyHubPage.test.jsx`: passed, 3 files and 8 tests.
- From repo root, `git diff --check`: passed.
- From repo root, frontend recursive state JSON parse: passed.
- From repo root, `tools/check-full-feature-tree.ps1`: passed.
- From `frontend/`, `npm.cmd run build`: passed.

## Further-Split Decision

`frontend.strategy_hub.roster_row_actions` does not need a deeper split now. The semantic action policy is pure and tested, while the component state is small, local, and tightly coupled to row UI rendering.

## Next Leaf

`frontend.strategy_hub.inspector_projection`
