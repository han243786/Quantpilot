# FE-0046 Frontend Strategy Workspace Monitor Research Source Tabs Closeout

Status: closed.

## Leaf Node

`frontend.strategy_workspace.monitor_research_source_tabs`

## Code Changes

- Added `frontend/src/pages/strategyWorkspaceAuxiliaryTabsShell.js`.
- Added `frontend/src/pages/strategyWorkspaceAuxiliaryTabsShell.test.js`.
- Updated `frontend/src/pages/StrategyWorkspaceMonitorTab.jsx` to delegate runtime kind labeling, monitor number/count formatting, event source selection, strip pill projection, and monitor metric projection to the extracted shell module.
- Updated `frontend/src/pages/StrategyWorkspaceResearchTab.jsx` to delegate research strip title and pill projection to the extracted shell module while keeping the research console lazy boundary in the tab.
- Updated `frontend/src/pages/StrategyWorkspaceSourceTab.jsx` to delegate source scenario request construction, HTTP error truncation, step icon/color projection, and failed-step `actual` extraction to the extracted shell module while keeping fetch side effects in the tab.

## Preserved Behavior

- Monitor tab still renders the same runtime, account, risk/execution, and recent-event cards.
- Runtime timeline remains preferred over runtime events, and only the five most recent events render in reverse chronological order.
- Research tab still lazy-loads `StrategyResearchConsole` and routes backtest details through the existing router.
- Source tab still loads QuantScript on `graphId`, runs `/api/test/scenario/run`, truncates HTTP error text to 300 characters, and renders the same passed/failed/skipped step symbols.

## Public Inputs

- Runtime state, graph nodes, recent runs, issue queue, formatter callback, and translation callback.
- Research strip translation callback.
- QuantScript source text, HTTP status/text, scenario step status, and step message.

## Public Outputs

- `formatWorkspaceMonitorNumber(value, digits)`.
- `formatWorkspaceMonitorCount(value)`.
- `resolveWorkspaceRuntimeKindLabel(kind, t)`.
- `selectWorkspaceRuntimeEvents(runtime)`.
- `buildWorkspaceMonitorModel({ graph, runtime, recentRuns, issueQueue, formatTime, t })`.
- `buildWorkspaceResearchStripModel(t)`.
- `buildSourceScenarioRunRequest(source)`.
- `buildSourceScenarioHttpError(status, text)`.
- `buildSourceScenarioStepPresentation(status)`.
- `extractSourceScenarioActualValue(message)`.

## Verification

- From `frontend/`, `npm.cmd test -- --run src/pages/strategyWorkspaceAuxiliaryTabsShell.test.js src/pages/StrategyWorkspacePage.codeMode.test.jsx`: passed, 2 test files and 11 tests.

## Further-Split Decision

`frontend.strategy_workspace.monitor_research_source_tabs` does not need a deeper split yet. Monitor, research, and source now expose their reusable shell projections, and source-side network effects remain intentionally local to the tab until a broader source authoring workflow is requested.

## Residuals

- Continue with `frontend.strategy_workspace.version_experiment_collaboration_cards`.
