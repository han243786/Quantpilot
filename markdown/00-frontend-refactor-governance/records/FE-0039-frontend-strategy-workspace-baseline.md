# FE-0039 Frontend Strategy Workspace Baseline

Status: baseline established.

## Parent Node

`frontend.strategy_workspace`

## Current Scope

The strategy workspace is the route-owned page surface for editing, inspecting, diagnosing, monitoring, and versioning a strategy graph. It should own page orchestration, workspace tab routing, workspace-only hooks, issue queue projection, workspace action-bar bridge logic, and workspace layout CSS. Core graph editor components should remain consumers for now and be handled under `frontend.graph_editor`.

## Initial Child Queue

- `frontend.strategy_workspace.route_shell`
- `frontend.strategy_workspace.shared_model_and_page_data`
- `frontend.strategy_workspace.issue_queue_state`
- `frontend.strategy_workspace.workspace_toolbar_bridge`
- `frontend.strategy_workspace.code_mode_shell`
- `frontend.strategy_workspace.dashboard_overview`
- `frontend.strategy_workspace.monitor_research_source_tabs`
- `frontend.strategy_workspace.version_experiment_collaboration_cards`
- `frontend.strategy_workspace.layout_styles`

## Current Owned And Split-Target Files

- `frontend/src/pages/StrategyWorkspacePage.jsx`
- `frontend/src/pages/StrategyWorkspacePage.codeMode.test.jsx`
- `frontend/src/pages/StrategyWorkspacePageSections.jsx`
- `frontend/src/pages/StrategyWorkspacePanelFallbacks.jsx`
- `frontend/src/pages/StrategyWorkspaceDashboard.jsx`
- `frontend/src/pages/StrategyWorkspaceOverviewTab.jsx`
- `frontend/src/pages/StrategyWorkspaceCodeTab.jsx`
- `frontend/src/pages/StrategyWorkspaceDiagnosticsTab.jsx`
- `frontend/src/pages/StrategyWorkspaceResearchTab.jsx`
- `frontend/src/pages/StrategyWorkspaceMonitorTab.jsx`
- `frontend/src/pages/StrategyWorkspaceDebugTab.jsx`
- `frontend/src/pages/StrategyWorkspaceSourceTab.jsx`
- `frontend/src/pages/StrategyWorkspaceIssueQueueCard.jsx`
- `frontend/src/pages/StrategyWorkspaceVersionHistoryCard.jsx`
- `frontend/src/pages/StrategyWorkspaceVersionHistoryCard.test.jsx`
- `frontend/src/pages/StrategyWorkspaceExperimentCard.jsx`
- `frontend/src/pages/StrategyWorkspaceExperimentCard.test.jsx`
- `frontend/src/pages/StrategyWorkspaceCollaborationCard.jsx`
- `frontend/src/pages/StrategyWorkspaceCollaborationCard.test.jsx`
- `frontend/src/pages/strategy-workspace.css`
- `frontend/src/hooks/useStrategyWorkspaceSharedModel.js`
- `frontend/src/hooks/useStrategyWorkspacePageData.js`
- `frontend/src/hooks/useStrategyWorkspaceUiState.js`
- `frontend/src/hooks/useWorkspaceActionBarActions.js`
- `frontend/src/hooks/useWorkspaceActionBarModel.js`
- `frontend/src/hooks/workspaceActionBarShared.js`
- `frontend/src/hooks/workspaceActionSelectors.js`
- `frontend/src/utils/strategyWorkspaceIssueQueue.js`
- `frontend/src/utils/strategyWorkspaceIssueQueue.test.js`
- `frontend/src/utils/workspaceContextLabels.js`
- `frontend/src/components/TopToolbar.jsx`
- `frontend/src/components/TopToolbar.capabilities.test.jsx`
- `frontend/src/components/TopToolbar.exportFailure.test.jsx`
- `frontend/src/components/TopToolbar.failureNotices.test.jsx`
- `frontend/src/components/TopToolbar.formalSourceMode.test.jsx`
- `frontend/src/components/TopToolbar.persistenceFailure.test.jsx`

## Important Consumers

- `frontend/src/components/ModuleSidebar.jsx`
- `frontend/src/components/StrategyCanvas.jsx`
- `frontend/src/components/StrategyCanvasMiniMap.jsx`
- `frontend/src/components/PropertyPanel.jsx`
- `frontend/src/components/StrategyCodePanel.jsx`
- `frontend/src/components/StrategyParamsPanel.jsx`
- `frontend/src/components/DiagnosticsPanel.jsx`
- `frontend/src/components/EventStreamPanel.jsx`
- `frontend/src/components/StrategyResearchConsole.jsx`
- `frontend/src/store/graphStore.js`
- `frontend/src/store/graphStoreEditorActions.js`
- `frontend/src/store/graphStoreCompileActions.js`
- `frontend/src/store/graphStoreRuntimeSessionActions.js`
- `frontend/src/router.js`

## Whitebox Contract

### Public Inputs

- Strategy id from route params.
- Graph store state, runtime state, selected node/edge/diagnostic target, and capability projection.
- Workspace tab state and URL-independent UI state.
- Compile diagnostics, validation issues, runtime history, backtest history, graph versions, and audit history.
- Workspace action requests for save, compile, export, run, backtest, and source-mode operations.

### Public Outputs

- Route-owned workspace shell and tab surface.
- Dashboard, overview, code, diagnostics, research, monitor, source, and debug tab mounts.
- Workspace issue queue grouping, filtering, selection, and focus handoff.
- Workspace action-bar model and blocked/action title projection.
- Version, experiment, and collaboration cards.
- Workspace layout CSS classes.

## Equivalence Anchors

- `frontend/src/pages/StrategyWorkspacePage.codeMode.test.jsx`
- `frontend/src/utils/strategyWorkspaceIssueQueue.test.js`
- `frontend/src/pages/StrategyWorkspaceVersionHistoryCard.test.jsx`
- `frontend/src/pages/StrategyWorkspaceExperimentCard.test.jsx`
- `frontend/src/pages/StrategyWorkspaceCollaborationCard.test.jsx`
- `frontend/src/components/TopToolbar.capabilities.test.jsx`
- `frontend/src/components/TopToolbar.formalSourceMode.test.jsx`
- `frontend/src/components/TopToolbar.persistenceFailure.test.jsx`
- Frontend build.

## Baseline Verification

- From `frontend/`, `npm.cmd test -- --run src/pages/StrategyWorkspacePage.codeMode.test.jsx src/utils/strategyWorkspaceIssueQueue.test.js src/pages/StrategyWorkspaceVersionHistoryCard.test.jsx src/pages/StrategyWorkspaceExperimentCard.test.jsx src/pages/StrategyWorkspaceCollaborationCard.test.jsx src/components/TopToolbar.capabilities.test.jsx src/components/TopToolbar.formalSourceMode.test.jsx src/components/TopToolbar.persistenceFailure.test.jsx`: passed, 8 test files and 23 tests.

## Split Rules

- Do not move graph editor primitives into this parent unless the current leaf is explicitly a workspace-only shell wrapper.
- Keep workspace capability gating sourced from `frontend.capabilities`.
- Keep route and navigation helpers owned by `frontend.routing`.
- Do not change user-visible tab availability without a capability projection test.
- Keep `StrategyWorkspacePage.jsx` as the public route gateway until its route shell leaf is closed.

## First Leaf

`frontend.strategy_workspace.route_shell`
