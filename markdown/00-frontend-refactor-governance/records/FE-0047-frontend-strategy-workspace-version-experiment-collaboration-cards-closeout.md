# FE-0047 Frontend Strategy Workspace Version Experiment Collaboration Cards Closeout

Status: closed.

## Leaf Node

`frontend.strategy_workspace.version_experiment_collaboration_cards`

## Code Changes

- Added `frontend/src/pages/strategyWorkspaceGovernanceCardsShell.js`.
- Added `frontend/src/pages/strategyWorkspaceGovernanceCardsShell.test.js`.
- Updated `frontend/src/pages/StrategyWorkspaceVersionHistoryCard.jsx` to delegate version formatting, config-domain labels, count-change formatting, draft summary projection, compare-entry selection, evidence-option projection, and compare-selection toggling to the extracted shell module.
- Updated `frontend/src/pages/StrategyWorkspaceExperimentCard.jsx` to delegate experiment grid parsing, percent formatting, graph-scoped experiment filtering, active-experiment selection, and experiment-start payload construction to the extracted shell module.
- Updated `frontend/src/pages/StrategyWorkspaceCollaborationCard.jsx` to delegate actor formatting, collaboration row projection, audit-refresh eligibility, and audit actor-line formatting to the extracted shell module.

## Preserved Behavior

- Version history still saves label/note, previews/restores versions, keeps a two-version compare queue, binds optional A/B backtest evidence, and renders the same diff sections.
- Experiment card still parses comma-separated fee, slippage, and latency grids before calling `startBacktestExperiment`.
- Experiment results still render the same variant rows and percent formatting.
- Collaboration card still refreshes audit history for persisted graphs only and renders owner, editors, last saver, last runner, and audit entries.

## Public Inputs

- Current graph metadata, version list, compare selection, and backtest history.
- Experiment list, selected experiment, current graph id, and grid draft strings.
- Collaboration actors, last run/backtest actors, graph id, and audit entries.

## Public Outputs

- `formatWorkspaceGovernanceTime(value)`.
- `formatWorkspaceVersionList(items)`.
- `workspaceConfigDomainLabel(domainId)`.
- `workspaceConfigChangeLabels(change)`.
- `formatWorkspaceVersionCountChanges(changes)`.
- `buildWorkspaceVersionDraftSummary(currentGraph)`.
- `selectWorkspaceVersionCompareEntries(compareSelection, graphVersions)`.
- `buildWorkspaceVersionEvidenceOptions(backtestHistory, graphId)`.
- `toggleWorkspaceVersionCompareSelection(current, versionId)`.
- `parseWorkspaceExperimentNumberList(input, parser)`.
- `formatWorkspaceExperimentPercent(value)`.
- `selectWorkspaceGraphExperiments(experiments, graphId)`.
- `selectWorkspaceActiveExperiment(selectedExperiment, graphId)`.
- `buildWorkspaceExperimentStartPayload({ experimentName, feeGridDraft, slippageGridDraft, latencyGridDraft })`.
- `formatWorkspaceActor(actor, fallback)`.
- `buildWorkspaceCollaborationRows({ collaboration, lastRun, lastBacktest })`.
- `shouldRefreshWorkspaceAuditHistory(graphId)`.
- `formatWorkspaceAuditActorLine(entry)`.

## Verification

- From `frontend/`, `npm.cmd test -- --run src/pages/strategyWorkspaceGovernanceCardsShell.test.js src/pages/StrategyWorkspaceVersionHistoryCard.test.jsx src/pages/StrategyWorkspaceExperimentCard.test.jsx src/pages/StrategyWorkspaceCollaborationCard.test.jsx`: passed, 4 test files and 8 tests.

## Further-Split Decision

`frontend.strategy_workspace.version_experiment_collaboration_cards` does not need a deeper split yet. The reusable projection and parsing contracts are now separated, while each card keeps its own store subscription and UI side effects. A deeper split would mostly separate already tested visual subsections without reducing current risk.

## Residuals

- Continue with `frontend.strategy_workspace.layout_styles`.
