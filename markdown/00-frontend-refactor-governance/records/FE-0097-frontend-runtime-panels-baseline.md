# FE-0097 Frontend Runtime Panels Baseline

Status: baseline established.

## Parent Node

`frontend.runtime_panels`

## Current Scope

This parent owns reusable frontend runtime panels and runtime-oriented projections used by the editor shell, strategy research console, workspace diagnostics tab, property panel diagnostics, and backtest detail surfaces.

The parent includes runtime event/history display, account/open-order summaries, run/backtest artifact management UI, runtime diagnostics projection and display, event replay, v4 runtime evidence summaries, timeline/report panels, mutation proposal controls, and the strategy research model that coordinates runtime panel filters and panel actions.

Runtime session store actions, runtime history persistence APIs, EventSource transport, backend API contracts, route parsing, approval/governance ops pages, and backtest detail page layout remain external parent inputs. Those should stay with `frontend.store`, `frontend.api_client`, `frontend.routing`, `frontend.governance_ops_pages`, or `frontend.backtest_views` unless a later parent explicitly owns them.

## Initial Child Queue

- `frontend.runtime_panels.event_stream_shell`
- `frontend.runtime_panels.strategy_research_model`
- `frontend.runtime_panels.history_sections`
- `frontend.runtime_panels.event_feed_and_candles`
- `frontend.runtime_panels.runtime_diagnostics_surface`
- `frontend.runtime_panels.evidence_timeline_reports`
- `frontend.runtime_panels.mutation_controls`
- `frontend.runtime_panels.replay_and_explanations`

## Current Owned And Split-Target Files

- `frontend/src/components/EventStreamPanel.jsx`
- `frontend/src/components/EventStreamPanel.backtestArtifacts.test.jsx`
- `frontend/src/components/EventStreamPanel.backtestHistory.test.jsx`
- `frontend/src/components/EventStreamPanel.dataQuality.test.jsx`
- `frontend/src/components/EventStreamPanel.executionExplanation.test.jsx`
- `frontend/src/components/EventStreamPanel.historyExplanation.test.jsx`
- `frontend/src/components/EventStreamPanel.layout.test.jsx`
- `frontend/src/components/EventStreamPanel.nodeFocus.test.jsx`
- `frontend/src/components/EventStreamPanel.refreshFeedback.test.jsx`
- `frontend/src/components/EventStreamPanel.runtimeArtifactActions.test.jsx`
- `frontend/src/components/BacktestHistorySection.jsx`
- `frontend/src/components/RunHistorySection.jsx`
- `frontend/src/components/StrategyBacktestsPanel.jsx`
- `frontend/src/components/StrategyEventsPanel.jsx`
- `frontend/src/components/StrategyRunsPanel.jsx`
- `frontend/src/components/StrategyResearchConsole.jsx`
- `frontend/src/components/StrategyResearchConsole.test.jsx`
- `frontend/src/components/AssetCandlesPanel.jsx`
- `frontend/src/components/AssetCandlesPanel.test.jsx`
- `frontend/src/components/EventReplaySection.jsx`
- `frontend/src/components/EventReplaySection.test.jsx`
- `frontend/src/components/RuntimeDiagnosticsPanel.jsx`
- `frontend/src/components/RuntimeDiagnosticsPanel.test.jsx`
- `frontend/src/components/RuntimeMutationPanel.jsx`
- `frontend/src/components/RuntimeMutationPanel.test.jsx`
- `frontend/src/components/RuntimeReportPanel.jsx`
- `frontend/src/components/RuntimeReportPanel.test.jsx`
- `frontend/src/components/GovernedTimelinePanel.jsx`
- `frontend/src/components/GovernedTimelinePanel.test.jsx`
- `frontend/src/components/V4RuntimeEvidencePanel.jsx`
- `frontend/src/components/V4RuntimeEvidencePanel.test.jsx`
- `frontend/src/components/EvidenceSummaryCards.jsx`
- `frontend/src/hooks/useStrategyResearchModel.js`
- `frontend/src/hooks/useStrategyResearchActions.js`
- `frontend/src/hooks/useStrategyResearchUiState.js`
- `frontend/src/hooks/strategyResearchSelectors.js`
- `frontend/src/hooks/strategyResearchSelectors.test.js`
- `frontend/src/hooks/useOrderAnimation.js`
- `frontend/src/utils/runtimeDiagnosticsProjection.js`
- `frontend/src/utils/runtimeDiagnosticsProjection.test.js`
- `frontend/src/utils/runtimeEvidenceSummary.js`
- `frontend/src/utils/runtimeEvidenceSummary.test.js`
- `frontend/src/utils/runtimeExplanation.js`
- `frontend/src/utils/runtimeExplanation.test.js`
- `frontend/src/utils/runtimeGovernance.js`
- `frontend/src/utils/runtimeGovernance.test.js`
- `frontend/src/utils/runtimeMutation.js`
- `frontend/src/utils/runtimeMutation.test.js`
- `frontend/src/utils/runtimeStatus.js`
- `frontend/src/utils/runtimeTimeline.js`
- `frontend/src/utils/runtimeTimeline.test.js`
- `frontend/src/utils/runtimeAiProposal.js`
- `frontend/src/utils/runtimeAiProposal.test.js`
- `frontend/src/utils/v4RuntimeEvidence.js`
- `frontend/src/utils/v4RuntimeEvidence.test.js`

## Important Consumers

- `frontend/src/pages/EditorPage.jsx`
- `frontend/src/pages/BacktestDetailPage.jsx`
- `frontend/src/pages/StrategyWorkspaceDiagnosticsTab.jsx`
- `frontend/src/pages/StrategyBacktestsPage.jsx`
- `frontend/src/components/propertyPanelSectionComposers.jsx`
- `frontend/src/store/graphStore.js`
- `frontend/src/store/graphStoreRuntimeHistoryActions.js`
- `frontend/src/store/graphStoreRuntimeSessionActions.js`
- `frontend/src/store/graphStoreRuntimeHistoryApi.js`
- `frontend/src/store/graphStoreRuntimeSelectionState.js`

## Whitebox Contract

### Public Inputs

- Graph store graph/runtime state, persisted run/backtest history, selected run/backtest ids, event lists, artifact persistence status, diagnostics, account/open-order snapshots, mutation proposals, and v4 runtime evidence envelopes.
- Store runtime actions for refreshing history, loading persisted detail, saving/discarding current runtime artifacts, toggling backtest compare selection, activating proposals, and rolling back proposals.
- Runtime report API helpers for creating, listing, opening, and exporting runtime reports.
- Router helpers for compare/detail navigation from runtime history surfaces.
- Capability context used by mutation controls.

### Public Outputs

- Event stream panel shell and exported section helpers used by editor/detail/research surfaces.
- Strategy research console runtime lanes for backtests, runs, and live event inspection.
- Run/backtest history sections with filters, pagination, comparison entry points, and runtime artifact actions.
- Runtime diagnostics panel, diagnostics projection rows, selected-node focus, and runtime evidence handoff.
- Evidence timeline, v4 evidence, runtime report, replay, mutation, candle, account, and event feed panels.
- Runtime explanation, status, timeline, governance, AI proposal, mutation, and evidence summary utilities.

## Equivalence Anchors

- `frontend/src/components/EventStreamPanel.layout.test.jsx`
- `frontend/src/components/EventStreamPanel.backtestArtifacts.test.jsx`
- `frontend/src/components/EventStreamPanel.backtestHistory.test.jsx`
- `frontend/src/components/EventStreamPanel.dataQuality.test.jsx`
- `frontend/src/components/EventStreamPanel.executionExplanation.test.jsx`
- `frontend/src/components/EventStreamPanel.historyExplanation.test.jsx`
- `frontend/src/components/EventStreamPanel.nodeFocus.test.jsx`
- `frontend/src/components/EventStreamPanel.refreshFeedback.test.jsx`
- `frontend/src/components/EventStreamPanel.runtimeArtifactActions.test.jsx`
- `frontend/src/components/StrategyResearchConsole.test.jsx`
- `frontend/src/components/RuntimeDiagnosticsPanel.test.jsx`
- `frontend/src/components/RuntimeReportPanel.test.jsx`
- `frontend/src/components/RuntimeMutationPanel.test.jsx`
- `frontend/src/components/GovernedTimelinePanel.test.jsx`
- `frontend/src/components/V4RuntimeEvidencePanel.test.jsx`
- `frontend/src/components/EventReplaySection.test.jsx`
- `frontend/src/components/AssetCandlesPanel.test.jsx`
- `frontend/src/hooks/strategyResearchSelectors.test.js`
- `frontend/src/utils/runtimeDiagnosticsProjection.test.js`
- `frontend/src/utils/runtimeEvidenceSummary.test.js`
- `frontend/src/utils/runtimeExplanation.test.js`
- `frontend/src/utils/runtimeGovernance.test.js`
- `frontend/src/utils/runtimeMutation.test.js`
- `frontend/src/utils/runtimeTimeline.test.js`
- `frontend/src/utils/runtimeAiProposal.test.js`
- `frontend/src/utils/v4RuntimeEvidence.test.js`
- Frontend build.

## Baseline Verification

- From `frontend/`, runtime panels anchor test set: passed, 26 files / 68 tests.
- From repo root, `git diff --check`: passed.
- From repo root, frontend recursive state JSON parse: passed.
- From repo root, `tools/check-full-feature-tree.ps1`: passed.
- From `frontend/`, `npm.cmd run build`: passed.

## Split Rules

- Keep runtime store state mutation, EventSource transport, runtime history persistence APIs, and session action lock behavior in `frontend.store` until that parent is active.
- Keep approval review queue UI and sandbox approval APIs out of this parent; they are ops/governance page candidates.
- Keep `BacktestDetailPage` and compare/detail page layout ownership for `frontend.backtest_views`, even when those pages consume runtime panels.
- Do not add cross-leaf shortcuts between runtime panels; route through shared utility exports, props, or the parent panel shell.
- Treat `EventStreamPanel.jsx` as the first high-value split candidate because it still contains exported helper sections, copied history section bodies, and orchestration glue.

## First Leaf

`frontend.runtime_panels.event_stream_shell`
