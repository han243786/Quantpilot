# FE-0051 Frontend Strategy Hub Route Shell Closeout

Status: closed.

## Leaf Node

`frontend.strategy_hub.route_shell`

## Code Changes

- Added `frontend/src/pages/strategyHubRouteShell.js`.
- Added `frontend/src/pages/strategyHubRouteShell.test.js`.
- Updated `frontend/src/pages/StrategyHubPage.jsx` to consume route shell props, fallback props, route heading, and visually hidden heading style from the extracted shell helper.

## Preserved Behavior

- `StrategyHubPage.jsx` remains the public route gateway.
- The page root still renders `className="strategy-hub-page"` and `data-testid="strategy-hub-page"`.
- The hidden route heading remains `策略中心`.
- Hero and body sections remain lazy-loaded through the same `Suspense` boundaries.
- Fallback titles remain `策略中心总览` and `策略中心工作区`.
- Strategy directory model creation still happens in `StrategyHubPage.jsx`; the directory model leaf will handle deeper model extraction.

## Public Inputs

- Existing strategy hub route heading and fallback labels.
- `useStrategyDirectoryModel` output.
- `StrategyHubHeroSection` and `StrategyHubBodySection` lazy children.
- `StrategyHubSectionFallback`.

## Public Outputs

- Stable strategy hub page shell props.
- Stable route heading and hidden heading style.
- Section fallback props for the hero and body lazy sections.

## Verification

- From `frontend/`, `npm.cmd test -- --run src/pages/strategyHubRouteShell.test.js src/pages/StrategyHubPage.test.jsx`: passed, 2 files and 6 tests.
- From repo root, `git diff --check`: passed.
- From repo root, frontend recursive state JSON parse: passed.
- From repo root, `tools/check-full-feature-tree.ps1`: passed.
- From `frontend/`, `npm.cmd run build`: passed.

## Further-Split Decision

`frontend.strategy_hub.route_shell` does not need a deeper split now. The leaf only owns route-level shell constants and fallback props; the larger state and model concerns are explicitly queued under `frontend.strategy_hub.directory_model`.

## Next Leaf

`frontend.strategy_hub.directory_model`
