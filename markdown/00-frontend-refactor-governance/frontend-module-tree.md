# Frontend Module Tree

Status: initialized empty from frontend-local truth.

This is the frontend-only module tree for parallel refactor work. It is not copied from the global tree and must not be treated as merged global truth.

## Root

- `root.frontend`

## Active Parent

- `frontend.strategy_workspace`
  - Status: parent baseline established.
  - Record: `markdown/00-frontend-refactor-governance/records/FE-0039-frontend-strategy-workspace-baseline.md`
  - Current owned and split-target files:
    - `frontend/src/pages/StrategyWorkspacePage.jsx`
    - `frontend/src/pages/StrategyWorkspacePage.codeMode.test.jsx`
    - `frontend/src/pages/strategyWorkspaceRouteShell.js`
    - `frontend/src/pages/strategyWorkspaceRouteShell.test.js`
    - `frontend/src/pages/StrategyWorkspacePageSections.jsx`
    - `frontend/src/pages/StrategyWorkspacePanelFallbacks.jsx`
    - `frontend/src/pages/StrategyWorkspaceDashboard.jsx`
    - `frontend/src/pages/StrategyWorkspaceOverviewTab.jsx`
    - `frontend/src/pages/StrategyWorkspaceCodeTab.jsx`
    - `frontend/src/pages/strategyWorkspaceCodeModeShell.js`
    - `frontend/src/pages/strategyWorkspaceCodeModeShell.test.js`
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
    - `frontend/src/hooks/strategyWorkspacePageDataProjection.js`
    - `frontend/src/hooks/strategyWorkspacePageDataProjection.test.js`
    - `frontend/src/hooks/useStrategyWorkspaceUiState.js`
    - `frontend/src/hooks/strategyWorkspaceIssueQueueState.js`
    - `frontend/src/hooks/strategyWorkspaceIssueQueueState.test.js`
    - `frontend/src/hooks/useWorkspaceActionBarActions.js`
    - `frontend/src/hooks/useWorkspaceActionBarModel.js`
    - `frontend/src/hooks/workspaceActionBarShared.js`
    - `frontend/src/hooks/workspaceActionSelectors.js`
    - `frontend/src/utils/strategyWorkspaceIssueQueue.js`
    - `frontend/src/utils/strategyWorkspaceIssueQueue.test.js`
    - `frontend/src/utils/workspaceContextLabels.js`
    - `frontend/src/components/TopToolbar.jsx`
    - `frontend/src/components/topToolbarBridge.js`
    - `frontend/src/components/topToolbarBridge.test.js`
    - `frontend/src/components/TopToolbar.capabilities.test.jsx`
    - `frontend/src/components/TopToolbar.exportFailure.test.jsx`
    - `frontend/src/components/TopToolbar.failureNotices.test.jsx`
    - `frontend/src/components/TopToolbar.formalSourceMode.test.jsx`
    - `frontend/src/components/TopToolbar.persistenceFailure.test.jsx`
  - Important consumers:
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
    - `frontend/src/router.js`
  - Child queue:
    - `frontend.strategy_workspace.monitor_research_source_tabs`
    - `frontend.strategy_workspace.version_experiment_collaboration_cards`
    - `frontend.strategy_workspace.layout_styles`
  - Closed child leaves:
    - `frontend.strategy_workspace.route_shell`
    - `frontend.strategy_workspace.shared_model_and_page_data`
    - `frontend.strategy_workspace.issue_queue_state`
    - `frontend.strategy_workspace.workspace_toolbar_bridge`
    - `frontend.strategy_workspace.code_mode_shell`
    - `frontend.strategy_workspace.dashboard_overview`

## Last Closed Parent

- `frontend.capabilities`
  - Status: parent closed.
  - Record: `markdown/00-frontend-refactor-governance/records/FE-0022-frontend-capabilities-baseline.md`
  - Closeout: `markdown/00-frontend-refactor-governance/records/FE-0038-frontend-capabilities-parent-closeout.md`
  - Current owned and split-target files:
    - `frontend/src/capabilities/supportMatrix.js`
    - `frontend/src/capabilities/supportMatrix.test.js`
    - `frontend/src/capabilities/capabilityActionBlocks.js`
    - `frontend/src/capabilities/capabilityActionBlocks.test.js`
    - `frontend/src/capabilities/capabilityCatalog.js`
    - `frontend/src/capabilities/capabilityCatalog.test.js`
    - `frontend/src/capabilities/capabilityBoundary.js`
    - `frontend/src/capabilities/capabilityBoundary.test.js`
    - `frontend/src/capabilities/capabilitySync.js`
    - `frontend/src/capabilities/capabilitySync.test.js`
    - `frontend/src/capabilities/capabilityProjection.js`
    - `frontend/src/capabilities/capabilityProjection.test.js`
    - `frontend/src/capabilities/capabilityGovernanceCore.js`
    - `frontend/src/capabilities/capabilityGovernanceCore.test.js`
    - `frontend/src/capabilities/capabilityGovernanceRegistry.js`
    - `frontend/src/capabilities/capabilityGovernanceRegistry.test.js`
    - `frontend/src/capabilities/capabilityGovernance.js`
    - `frontend/src/capabilities/capabilityGovernance.test.js`
    - `frontend/src/capabilities/builtinCapabilitySnapshot.js`
    - `frontend/src/capabilities/builtinCapabilitySnapshot.test.js`
    - `frontend/src/capabilities/capabilityNormalization.js`
    - `frontend/src/capabilities/capabilityNormalization.test.js`
    - `frontend/src/modules/moduleRegistryContracts.js`
    - `frontend/src/modules/moduleRegistryContracts.test.js`
    - `frontend/src/modules/moduleRegistryAssembly.js`
    - `frontend/src/modules/moduleRegistryAssembly.test.js`
    - `frontend/src/modules/moduleRegistry.js`
    - `frontend/src/modules/moduleRegistry.test.js`
    - `frontend/src/modules/builtinModules.js`
    - `frontend/src/store/graphStore.js`
    - `frontend/src/store/graphStoreCapabilityRefresh.js`
    - `frontend/src/store/graphStoreCapabilityRefresh.test.js`
    - `frontend/src/store/graphStore.capabilities.test.js`
  - Important consumers:
    - `frontend/src/store/graphStore.js`
    - `frontend/src/store/graphStorePersistenceHelpers.js`
    - `frontend/src/store/graphStoreCompileActions.js`
    - `frontend/src/store/graphStoreRuntimeSessionActions.js`
    - `frontend/src/components/ModuleSidebar.jsx`
    - `frontend/src/components/TopToolbar.jsx`
    - `frontend/src/pages/StrategyWorkspacePage.jsx`
    - `frontend/src/pages/StrategyWorkspaceExperimentCard.jsx`
    - `frontend/src/graph/compileGraph.js`
  - Child queue:
    - None.
  - Closed child leaves:
    - `frontend.capabilities.support_matrix_truth.sync_block_gate`
    - `frontend.capabilities.support_matrix_truth.catalog_maps`
    - `frontend.capabilities.support_matrix_truth.boundary_context`
    - `frontend.capabilities.support_matrix_truth.action_block_reason`
    - `frontend.capabilities.capability_projection`
    - `frontend.capabilities.governance_registry.core_contract`
    - `frontend.capabilities.governance_registry.registry_entries`
    - `frontend.capabilities.governance_registry.public_facade`
    - `frontend.capabilities.governance_registry`
    - `frontend.capabilities.builtin_capability_snapshot.default_snapshot`
    - `frontend.capabilities.builtin_capability_snapshot.normalization`
    - `frontend.capabilities.builtin_capability_snapshot`
    - `frontend.capabilities.module_registry_gate.contract_validation`
    - `frontend.capabilities.module_registry_gate.registry_assembly`
    - `frontend.capabilities.module_registry_gate.public_facade`
    - `frontend.capabilities.module_registry_gate`
    - `frontend.capabilities.store_capability_refresh.refresh_state_projection`
    - `frontend.capabilities.store_capability_refresh.public_action_facade`
    - `frontend.capabilities.store_capability_refresh`

## Previously Closed Parent

- `frontend.api_client`
  - Status: parent closed.
  - Record: `markdown/00-frontend-refactor-governance/records/FE-0017-frontend-api-client-baseline.md`
  - Closeout: `markdown/00-frontend-refactor-governance/records/FE-0021-frontend-api-client-parent-closeout.md`
  - Current owned files:
    - `frontend/src/api/apiBase.js`
    - `frontend/src/api/apiBase.test.js`
    - `frontend/src/api/apiTransport.js`
    - `frontend/src/api/apiTransport.test.js`
    - `frontend/src/api/client.js`
    - `frontend/src/api/fetchHelpers.js`
    - `frontend/src/api/fetchHelpers.test.js`
    - `frontend/src/utils/api.js`
  - Important consumers:
    - `frontend/src/components/DeployButton.jsx`
    - `frontend/src/pages/StrategyConfigCockpit.jsx`
    - `frontend/src/store/graphStorePersistenceHelpers.js`
    - `frontend/src/pages/AlertsPage.jsx`
    - `frontend/src/pages/ChaosPage.jsx`
    - `frontend/src/pages/RunbookPage.jsx`
    - `frontend/src/pages/SnapshotsPage.jsx`
    - `frontend/src/pages/StrategyWorkspaceSourceTab.jsx`
    - `frontend/src/components/TopToolbar.jsx`
    - `frontend/src/utils/runtimeApproval.js`
  - Child queue: closed.
  - Closed child leaves:
    - `frontend.api_client.base_resolution`
    - `frontend.api_client.request_transport`
    - `frontend.api_client.compat_fetch_helpers`

## Previously Closed Parent

- `frontend.routing`
  - Status: parent closed.
  - Record: `markdown/00-frontend-refactor-governance/records/FE-0012-frontend-routing-baseline.md`
  - Closeout: `markdown/00-frontend-refactor-governance/records/FE-0016-frontend-routing-parent-closeout.md`
  - Current owned files:
    - `frontend/src/router.js`
    - `frontend/src/router.test.js`
    - `frontend/src/routing/navigationDispatch.js`
    - `frontend/src/routing/navigationDispatch.test.js`
    - `frontend/src/routing/routeContract.js`
    - `frontend/src/routing/routeContract.test.js`
    - `frontend/src/routing/shellNavigation.js`
    - `frontend/src/routing/shellNavigation.test.js`
  - Important consumers:
    - `frontend/src/app/useAppRoute.js`
    - `frontend/src/App.jsx`
    - `frontend/src/components/LeftSidebar.jsx`
    - `frontend/src/components/CommandPalette.jsx`
    - `frontend/src/components/BacktestHistorySection.jsx`
    - `frontend/src/components/EventStreamPanel.jsx`
    - `frontend/src/hooks/useStrategyDirectoryModel.js`
    - `frontend/src/pages/*`
    - `frontend/src/utils/*Actions.js`
    - `frontend/src/test/testBridge.js`
  - Child queue: closed.
  - Closed child leaves:
    - `frontend.routing.route_contract`
    - `frontend.routing.navigation_dispatch`
    - `frontend.routing.shell_navigation`

- `frontend.app_shell`
  - Status: parent closed.
  - Record: `markdown/00-frontend-refactor-governance/records/FE-0002-frontend-app-shell-baseline.md`
  - Closeout: `markdown/00-frontend-refactor-governance/records/FE-0011-frontend-app-shell-parent-closeout.md`
  - Current owned files:
    - `frontend/src/main.jsx`
    - `frontend/src/App.jsx`
    - `frontend/src/app/AppGlobalOverlays.jsx`
    - `frontend/src/app/AppGlobalOverlays.test.jsx`
    - `frontend/src/app/DesktopTitleBar.jsx`
    - `frontend/src/app/DesktopTitleBar.test.jsx`
    - `frontend/src/app/AppRouteHost.jsx`
    - `frontend/src/app/AppRouteHost.test.jsx`
    - `frontend/src/app/AppRoot.jsx`
    - `frontend/src/app/AppRoot.test.jsx`
    - `frontend/src/app/AppShellFallback.jsx`
    - `frontend/src/app/AppShellFallback.test.jsx`
    - `frontend/src/app/installGlobalErrorHandlers.js`
    - `frontend/src/app/installGlobalErrorHandlers.test.js`
    - `frontend/src/app/useAppEnvironmentEvents.js`
    - `frontend/src/app/useAppEnvironmentEvents.test.jsx`
    - `frontend/src/app/useAppInitialization.js`
    - `frontend/src/app/useAppInitialization.test.jsx`
    - `frontend/src/app/useAppRoute.js`
    - `frontend/src/app/useAppRoute.test.jsx`
    - `frontend/src/app/useDesktopWindowChrome.js`
    - `frontend/src/app/useDesktopWindowChrome.test.jsx`
  - Child queue: closed.
  - Closed child leaves:
    - `frontend.app_shell.bootstrap_root`
    - `frontend.app_shell.startup_readiness`
    - `frontend.app_shell.environment_events`
    - `frontend.app_shell.desktop_window_chrome`
    - `frontend.app_shell.route_host`
    - `frontend.app_shell.global_overlays`

## Pending Parent Queue

- `frontend.strategy_hub`
- `frontend.graph_editor`
- `frontend.runtime_panels`
- `frontend.backtest_views`
- `frontend.store`
- `frontend.design_system_styles`
- `frontend.test_support`

## Closed Nodes

- `frontend.app_shell`
- `frontend.routing`
- `frontend.api_client`
- `frontend.capabilities`

## Deferred Merge Notes

Global module tree merge is deferred until frontend refactor closeout.
