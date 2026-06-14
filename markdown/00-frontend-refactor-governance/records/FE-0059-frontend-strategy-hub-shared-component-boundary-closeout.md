# FE-0059 Frontend Strategy Hub Shared Component Boundary Closeout

Status: closed.

## Leaf Node

`frontend.strategy_hub.shared_component_boundary`

## Code Changes

- Moved the implementation behind `frontend/src/pages/StrategyHubSharedComponents.jsx` to `frontend/src/components/strategySharedComponents.jsx`.
- Moved the implementation behind `frontend/src/utils/strategyHubFormatters.js` to `frontend/src/utils/strategyFormatters.js`.
- Retained the two hub-named files as compatibility re-export aliases so existing global documentation path checks do not require hot-file edits during the isolated frontend recursion.
- Updated strategy hub, strategy workspace, generic component, and backtest-analysis imports to consume the neutral shared boundaries.

## Preserved Behavior

- `StrategyCardNote`, `StrategyMetricCard`, `StrategyOpsCard`, `StrategyTaskGroup`, and `ActivityListCard` keep the same exported names and rendering behavior.
- `formatTime`, `formatCount`, and `formatPercent` keep the same formatting behavior.
- Hub, workspace, generic component, and backtest-analysis consumers keep the same public UI and formatter outputs.
- No remaining frontend source import references the hub-named shared aliases.

## Public Inputs

- Shared note/card/task/activity component props.
- Timestamp, count, and percent formatter values.
- Existing hub, workspace, component, and backtest-analysis consumers.

## Public Outputs

- Neutral shared component module at `frontend/src/components/strategySharedComponents.jsx`.
- Neutral shared formatter module at `frontend/src/utils/strategyFormatters.js`.
- Compatibility aliases at `frontend/src/pages/StrategyHubSharedComponents.jsx` and `frontend/src/utils/strategyHubFormatters.js`.
- Updated import graph without cross-parent imports from hub-named shared files.

## Verification

- From `frontend/`, `npm.cmd test -- --run src/pages/StrategyHubPage.test.jsx src/pages/StrategyWorkspacePage.codeMode.test.jsx src/components/ModuleSidebar.test.jsx src/components/RuntimeDiagnosticsPanel.test.jsx src/components/StrategyResearchConsole.test.jsx src/components/PropertyPanel.layout.test.jsx src/components/PropertyPanel.compileSummary.test.jsx src/components/PropertyPanel.strategyIr.test.jsx src/pages/backtestAnalysisShared.test.jsx src/utils/strategyHubHeroSummary.test.js src/utils/strategyHubRosterProjection.test.js src/utils/strategyHubInspectorProjection.test.js`: passed, 12 files and 39 tests.
- From repo root, no remaining `StrategyHubSharedComponents` or `strategyHubFormatters` source content references outside the compatibility aliases: passed.
- From repo root, `git diff --check`: passed.
- From repo root, frontend recursive state JSON parse: passed.
- From repo root, `tools/check-full-feature-tree.ps1`: passed.
- From `frontend/`, `npm.cmd run build`: passed.

## Further-Split Decision

`frontend.strategy_hub.shared_component_boundary` does not need a deeper split now. The reusable UI primitives and shared formatters have neutral names and are no longer owned by a hub-only parent. Future deeper work can happen under a dedicated shared component parent if the global frontend module tree introduces one.

## Next Leaf

`frontend.strategy_hub.layout_styles`
