# Persistence / Replay Contract

This file is the active wording boundary for `CL-P1-004`.

## Goal

Keep persisted runtime detail, backtest detail, history cards, and replay views
on one stable payload family.

## Source of persisted truth

- run detail payload returned by `/api/runtime/history/:run_id`
- backtest detail payload returned by `/api/runtime/backtests/:backtest_id`
- replay payload returned by `/api/runtime/history/:run_id/replay` or
  `/api/runtime/backtests/:backtest_id/replay`
- persisted `backtest_artifacts.event_log.events`
- persisted `runtime_diagnostics`

The frontend must not rebuild these shapes from unrelated live-only state.

## Shared frontend state rules

- completed backtest selection now uses one shared builder in
  `frontend/src/store/graphStoreRuntimeSelectionState.js`
- live backtest completion and persisted backtest reload must project the same
  `runtime` selection shape
- persisted run detail and persisted backtest detail must clear the opposite
  selection id instead of leaving mixed `selectedHistoryRunId` /
  `selectedBacktestId` state behind
- replay widgets read the persisted record id plus sequence window directly from
  replay responses

## Reload rules

- detail pages should prefer persisted artifacts such as
  `backtest_artifacts.event_log.events`
- `runtime_diagnostics` should be reused directly after reload
- highlighted node ids should derive from persisted events, not from stale
  canvas selection memory
- `selectedNodeId` should follow the first highlighted persisted node when one
  exists

## Closeout rules

- live completion and persisted reload must agree on `runId`, `runKind`,
  `account`, `backtestArtifacts`, `diagnostics`, `events`, selected ids, and
  highlighted node ids when they describe the same record
- history cards, detail pages, and replay must stay readable after a full page
  reload without reconstructing missing explanation state
- no second reload-only DTO family may be introduced

## Current implementation entrypoints

- `frontend/src/store/graphStoreRuntimeSelectionState.js`
- `frontend/src/store/graphStoreRuntimeSessionState.js`
- `frontend/src/store/graphStoreRuntimeHistoryState.js`
- `frontend/src/store/graphStoreRuntimeHistoryProjection.js`
- `frontend/src/components/EventStreamPanel.jsx`
- `frontend/src/components/EventReplaySection.jsx`
- `frontend/src/pages/BacktestDetailPage.jsx`
