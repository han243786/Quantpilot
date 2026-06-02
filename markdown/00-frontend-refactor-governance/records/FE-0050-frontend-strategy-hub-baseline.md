# FE-0050 Frontend Strategy Hub Baseline

Status: baseline established.

## Parent Node

`frontend.strategy_hub`

## Current Scope

The strategy hub is the route-owned strategy management surface for browsing tracked strategy files, opening workspaces, applying templates, reviewing recent runs/backtests, managing compare selections, and inspecting the currently selected strategy. It should own hub page orchestration, hub-only projection/action helpers, strategy roster UI, inspector UI, recent activity UI, template-library UI, and hub layout CSS.

The graph store, router helpers, template definitions, and runtime/backtest detail pages remain external parent inputs. `useStrategyDirectoryModel` is a hub-owned split target because it is currently the strategy hub page model and is consumed only by `StrategyHubPage.jsx`, but its graph-store selectors and store actions must stay behind explicit parent boundary rules.

## Initial Child Queue

- `frontend.strategy_hub.route_shell`
- `frontend.strategy_hub.directory_model`
- `frontend.strategy_hub.hero_summary`
- `frontend.strategy_hub.roster_projection`
- `frontend.strategy_hub.roster_row_actions`
- `frontend.strategy_hub.inspector_projection`
- `frontend.strategy_hub.recent_activity_compare`
- `frontend.strategy_hub.template_library`
- `frontend.strategy_hub.shared_component_boundary`
- `frontend.strategy_hub.layout_styles`

## Current Owned And Split-Target Files

- `frontend/src/pages/StrategyHubPage.jsx`
- `frontend/src/pages/StrategyHubPage.test.jsx`
- `frontend/src/pages/StrategyHubPanelFallbacks.jsx`
- `frontend/src/pages/StrategyHubSectionFallbacks.jsx`
- `frontend/src/pages/StrategyHubHeroSection.jsx`
- `frontend/src/pages/StrategyHubInlineNote.jsx`
- `frontend/src/pages/StrategyHubBodySection.jsx`
- `frontend/src/pages/StrategyHubTemplateLibrarySection.jsx`
- `frontend/src/pages/StrategyHubTemplateLibrarySection.test.jsx`
- `frontend/src/pages/StrategyHubRosterSection.jsx`
- `frontend/src/pages/StrategyHubRosterDirectorySection.jsx`
- `frontend/src/pages/StrategyHubRosterToolbar.jsx`
- `frontend/src/pages/StrategyHubRosterTableSection.jsx`
- `frontend/src/pages/StrategyHubRosterTableSection.test.jsx`
- `frontend/src/pages/StrategyHubRosterTableRow.jsx`
- `frontend/src/pages/StrategyHubRosterRowActions.jsx`
- `frontend/src/pages/StrategyHubActivityPanelsSection.jsx`
- `frontend/src/pages/StrategyHubBacktestActivityCard.jsx`
- `frontend/src/pages/StrategyHubRunActivityCard.jsx`
- `frontend/src/pages/StrategyHubInspectorSection.jsx`
- `frontend/src/pages/StrategyHubInspectorOverviewSection.jsx`
- `frontend/src/pages/StrategyHubRecentBacktestsSection.jsx`
- `frontend/src/pages/StrategyHubRecentRunsSection.jsx`
- `frontend/src/pages/StrategyHubRecentRunItem.jsx`
- `frontend/src/pages/StrategyHubCompareQueueSection.jsx`
- `frontend/src/pages/strategy-hub.css`
- `frontend/src/hooks/useStrategyDirectoryModel.js`
- `frontend/src/hooks/useStrategyHubBodyData.js`
- `frontend/src/hooks/useStrategyHubRosterData.js`
- `frontend/src/hooks/useStrategyHubInspectorData.js`
- `frontend/src/components/strategySharedComponents.jsx`
- `frontend/src/utils/strategyHubStrategyIdentity.js`
- `frontend/src/utils/strategyFormatters.js`
- `frontend/src/utils/strategyHubRosterProjection.js`
- `frontend/src/utils/strategyHubRosterProjection.test.js`
- `frontend/src/utils/strategyHubRosterRowActions.js`
- `frontend/src/utils/strategyHubRosterRowActions.test.js`
- `frontend/src/utils/strategyHubInspectorProjection.js`
- `frontend/src/utils/strategyHubInspectorProjection.test.js`
- `frontend/src/utils/strategyHubInspectorActions.js`
- `frontend/src/utils/strategyHubInspectorActions.test.js`
- `frontend/src/utils/strategyHubRecentRunsView.js`
- `frontend/src/utils/strategyHubRecentRunsView.test.js`
- `frontend/src/utils/strategyHubRecentBacktestsActions.js`
- `frontend/src/utils/strategyHubRecentBacktestsActions.test.js`
- `frontend/src/utils/strategyHubCompareQueueActions.js`
- `frontend/src/utils/strategyHubCompareQueueActions.test.js`

## Important Consumers

- `frontend/src/app/AppRouteHost.jsx`
- `frontend/src/router.js`
- `frontend/src/store/graphStore.js`
- `frontend/src/templates/strategyTemplates.js`
- `frontend/src/pages/backtestAnalysisShared.jsx`
- `frontend/src/pages/StrategyWorkspaceCodeTab.jsx`
- `frontend/src/pages/StrategyWorkspaceIssueQueueCard.jsx`
- `frontend/src/pages/StrategyWorkspacePageSections.jsx`
- `frontend/src/components/ModuleSidebar.jsx`
- `frontend/src/components/RuntimeDiagnosticsPanel.jsx`
- `frontend/src/components/propertyPanelViews.jsx`
- `frontend/src/components/StrategyResearchConsole.jsx`

## Whitebox Contract

### Public Inputs

- Graph store graph metadata, graph index, runtime history, backtest history, compare selections, and refresh/load/delete/reveal actions.
- Router helpers for opening strategy workspaces, strategy backtests, backtest detail, and compare routes.
- Strategy template definitions and template load actions.
- Browser confirmation and tutorial open event.
- Existing hub CSS import through `StrategyHubPage.jsx`.

### Public Outputs

- Route-owned strategy hub page shell and lazy-loaded hub sections.
- Strategy directory model for tracked strategy files, selected strategy state, activity timeline, compare queue, and hub summary.
- Hero/status-strip metrics and workspace entry actions.
- Template library rendering and template load interaction.
- Roster toolbar, roster rows, row action groups, and row action dispatch.
- Inspector overview, next-move guidance, recent backtests, recent runs, and compare queue actions.
- Hub shared note/card/task/activity components until the shared component boundary leaf resolves cross-parent leakage.
- Hub layout CSS classes.

## Equivalence Anchors

- `frontend/src/pages/StrategyHubPage.test.jsx`
- `frontend/src/pages/StrategyHubRosterTableSection.test.jsx`
- `frontend/src/pages/StrategyHubTemplateLibrarySection.test.jsx`
- `frontend/src/utils/strategyHubRosterProjection.test.js`
- `frontend/src/utils/strategyHubRosterRowActions.test.js`
- `frontend/src/utils/strategyHubInspectorProjection.test.js`
- `frontend/src/utils/strategyHubInspectorActions.test.js`
- `frontend/src/utils/strategyHubRecentRunsView.test.js`
- `frontend/src/utils/strategyHubRecentBacktestsActions.test.js`
- `frontend/src/utils/strategyHubCompareQueueActions.test.js`
- Frontend build.

## Baseline Verification

- From `frontend/`, `npm.cmd test -- --run src/pages/StrategyHubPage.test.jsx src/pages/StrategyHubRosterTableSection.test.jsx src/pages/StrategyHubTemplateLibrarySection.test.jsx src/utils/strategyHubRosterProjection.test.js src/utils/strategyHubRosterRowActions.test.js src/utils/strategyHubInspectorProjection.test.js src/utils/strategyHubInspectorActions.test.js src/utils/strategyHubRecentRunsView.test.js src/utils/strategyHubRecentBacktestsActions.test.js src/utils/strategyHubCompareQueueActions.test.js`: passed, 10 files and 23 tests.
- From repo root, `git diff --check`: passed.
- From repo root, frontend recursive state JSON parse: passed.
- From repo root, `tools/check-full-feature-tree.ps1`: passed.
- From `frontend/`, `npm.cmd run build`: passed.

## Split Rules

- Keep graph-store state mutation and persistence behavior in store-owned APIs; hub leaves may wrap or project model data but must not rewrite store contracts.
- Keep router path builders owned by `frontend.routing`; hub leaves may call them but must not duplicate route construction.
- Keep template definitions owned by the templates module; hub leaves may render or apply templates through the existing model boundary.
- Shared component and formatter leakage is resolved through `frontend.strategy_hub.shared_component_boundary`; workspace and generic components consume neutral shared files instead of hub-named files.
- Split `strategy-hub.css` after the structural UI leaves so style regions can follow the final component boundaries.

## First Leaf

`frontend.strategy_hub.route_shell`
